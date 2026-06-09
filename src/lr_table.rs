use std::collections::BTreeMap;
use std::fmt;

use crate::{
    dfa_lr0::DfaLR0,
    first_follow::{get_first_set, get_follow_set},
    grammar::Grammar,
    symbol::{NonTerminal, Symbol, Terminal},
};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Action {
    Shift(usize),
    Reduce(usize),
    Accept,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Shift(state_id) => write!(f, "s{state_id}"),
            Action::Reduce(production_id) => write!(f, "r{production_id}"),
            Action::Accept => write!(f, "acc"),
        }
    }
}

pub struct LRTable {
    pub action: BTreeMap<(usize, Terminal), Action>,
    // GOTO 只记录“状态 + 非终结符 -> 新状态”的归约后转移。
    pub goto: BTreeMap<(usize, NonTerminal), usize>,
    pub conflicts: Vec<Conflict>,
}

#[derive(Clone, Debug)]
pub struct Conflict {
    pub state_id: usize,
    pub terminal: Terminal,
    pub existing: Action,
    pub incoming: Action,
}

impl fmt::Display for Conflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "state {} / {}: {} vs {}",
            self.state_id, self.terminal, self.existing, self.incoming
        )
    }
}

impl LRTable {
    // 统一从这里写 ACTION 单元格，既负责正常插入，也负责收集冲突信息。
    pub fn insert_action(
        self: &mut Self,
        old_state: usize,
        term: Terminal,
        action: Action,
    ) -> bool {
        if !self.action.contains_key(&(old_state, term)) {
            self.action.insert((old_state, term), action);
            true
        } else {
            let existing = self.action.get(&(old_state, term)).unwrap();
            if *existing == action {
                true
            } else {
                self.conflicts.push(Conflict {
                    state_id: old_state,
                    terminal: term,
                    existing: existing.clone(),
                    incoming: action,
                });
                false
            }
        }
    }

    pub fn new() -> Self {
        Self {
            action: BTreeMap::new(),
            goto: BTreeMap::new(),
            conflicts: Vec::new(),
        }
    }

    // LR(0) 和 SLR(1) 共享同一套构表骨架，差异只在完成项目可落表的终结符集合。
    fn build_with_reduce<F>(grammar: &Grammar, dfa: &DfaLR0, reduce_terms: F) -> Result<Self, String>
    where
        F: Fn(usize, &Grammar) -> Result<Vec<Terminal>, String>,
    {
        let mut lr_table = Self::new();

        for ((old_state, symbol), new_state) in &dfa.transitions {
            match symbol {
                Symbol::NonTerminal(nt) => {
                    lr_table.goto.insert((*old_state, *nt), *new_state);
                }
                Symbol::Terminal(term) => {
                    lr_table.insert_action(*old_state, *term, Action::Shift(*new_state));
                }
                Symbol::Epsilon => {
                    return Err(
                        "internal lr table error: epsilon transition should not appear in DFA"
                            .to_string(),
                    );
                }
            }
        }

        for (state_id, state) in dfa.states.iter().enumerate() {
            for item in state.iter() {
                if !item.is_complete(grammar) {
                    continue;
                }

                if item.production_id == 0 {
                    lr_table.insert_action(state_id, Terminal::End, Action::Accept);
                } else {
                    for term in reduce_terms(item.production_id, grammar)? {
                        lr_table.insert_action(state_id, term, Action::Reduce(item.production_id));
                    }
                }
            }
        }

        Ok(lr_table)
    }

    pub fn build_lr0(grammar: &Grammar, dfa: &DfaLR0) -> Result<Self, String> {
        Self::build_with_reduce(grammar, dfa, |_production_id, _grammar| {
            Ok(Terminal::all().to_vec())
        })
    }

    pub fn build_slr1(grammar: &Grammar, dfa: &DfaLR0) -> Result<Self, String> {
        let first_set = get_first_set(grammar);
        let follow_set = get_follow_set(grammar, &first_set);

        // SLR(1) 只在 FOLLOW(lhs) 所在列填 reduce，其他部分和 LR(0) 共用。
        Self::build_with_reduce(grammar, dfa, |production_id, grammar| {
            let production = grammar
                .production(production_id)
                .ok_or_else(|| {
                    format!(
                        "internal slr table error: invalid production id {} while reading FOLLOW set",
                        production_id
                    )
                })?;
            let terminals = follow_set
                .get(&production.lhs)
                .into_iter()
                .flat_map(|set| set.iter())
                .filter_map(|symbol| match symbol {
                    Symbol::Terminal(term) => Some(*term),
                    Symbol::NonTerminal(_) | Symbol::Epsilon => None,
                })
                .collect();

            Ok(terminals)
        })
    }
}

pub fn format_lr0_table(table: &LRTable, grammar: &Grammar) -> String {
    // 为了便于人工核对，reduce 项直接展开成具体产生式，而不是只打印 rN。
    let mut lines = Vec::new();

    lines.push("ACTION:".to_string());
    for ((state_id, terminal), action) in &table.action {
        let action_text = match action {
            Action::Reduce(production_id) => match grammar.production(*production_id) {
                Some(production) => format!("reduce {}", production),
                None => action.to_string(),
            },
            _ => action.to_string(),
        };
        lines.push(format!(
            "  ACTION[{}, {}] = {}",
            state_id, terminal, action_text
        ));
    }

    lines.push(String::new());
    lines.push("GOTO:".to_string());
    for ((state_id, non_terminal), target_state) in &table.goto {
        lines.push(format!(
            "  GOTO[{}, {}] = {}",
            state_id, non_terminal, target_state
        ));
    }

    lines.push(String::new());
    lines.push(format!("conflicts: {}", table.conflicts.len()));
    for conflict in &table.conflicts {
        lines.push(format!("  {conflict}"));
    }

    lines.join("\n")
}

impl fmt::Display for LRTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ACTION:")?;
        for ((state_id, terminal), action) in &self.action {
            writeln!(f, "  ACTION[{}, {}] = {}", state_id, terminal, action)?;
        }

        writeln!(f)?;
        writeln!(f, "GOTO:")?;
        for ((state_id, non_terminal), target_state) in &self.goto {
            writeln!(
                f,
                "  GOTO[{}, {}] = {}",
                state_id, non_terminal, target_state
            )?;
        }

        writeln!(f)?;
        writeln!(f, "conflicts: {}", self.conflicts.len())?;
        for conflict in &self.conflicts {
            writeln!(f, "  {conflict}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{LRTable, format_lr0_table};
    use crate::{dfa_lr0::build_dfa_lr0, grammar::Grammar};

    #[test]
    fn lr0_and_slr1_conflict_counts_match_expectation() {
        let grammar = Grammar::simple_lr();
        let dfa = build_dfa_lr0(&grammar);
        let lr0_table = LRTable::build_lr0(&grammar, &dfa).expect("lr0 table should build");
        let slr_table = LRTable::build_slr1(&grammar, &dfa).expect("slr table should build");

        assert_eq!(lr0_table.conflicts.len(), 23);
        assert_eq!(slr_table.conflicts.len(), 0);
    }

    #[test]
    fn formats_lr_table_sections() {
        let grammar = Grammar::simple_lr();
        let dfa = build_dfa_lr0(&grammar);
        let slr_table = LRTable::build_slr1(&grammar, &dfa).expect("slr table should build");
        let formatted = format_lr0_table(&slr_table, &grammar);

        assert!(formatted.contains("ACTION:"));
        assert!(formatted.contains("GOTO:"));
        assert!(formatted.contains("conflicts: 0"));
    }
}
