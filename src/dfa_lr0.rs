use crate::dfa::Dfa;
use crate::grammar::Grammar;
use crate::item_lr0::{ItemLR0, StateLR0};
use crate::symbol::Symbol;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// 输入state，输出其闭包state
/// 只要闭包中还存在圆点后是非终结符的项目，就继续把该非终结符的初始项目补进来。
pub fn closure_lr0(state: &StateLR0, grammar: &Grammar) -> StateLR0 {
    let mut closure = state.clone();
    loop {
        let mut changed = false;
        let tmp_vec = closure.iter().cloned().collect::<Vec<ItemLR0>>();
        for item in &tmp_vec {
            if let Some(symbol) = item.symbol_after_dot(grammar)
                && let Symbol::NonTerminal(nt) = symbol
            {
                grammar
                    .productions_for(*nt)
                    .map(|(prod_id, _)| ItemLR0::new(prod_id, 0))
                    .for_each(|item| {
                        changed |= closure.insert(item);
                        // 构建以当前非终结符为lhs，dot = 0的item，尝试插入closure
                        // 只要有一个成功插入，就算是修改了
                    });
            }
        }
        if !changed {
            break;
        }
    }
    closure
}

// GOTO 会先把所有可在该符号上前移的项目推进一格，再对结果重新取闭包。
pub fn goto_lr0(state: &StateLR0, symbol: &Symbol, grammar: &Grammar) -> Option<StateLR0> {
    let mut result = StateLR0::new();
    state
        .iter()
        .filter(
            // matches! 在这里直接把“圆点后是否正好等于目标符号”写成布尔条件。
            |item| {
                matches!(item.symbol_after_dot(grammar), 
                    Some(cur_symbol) if cur_symbol == symbol)
            },
        )
        .map(|item| item.advance_dot(grammar))
        .for_each(|item| {
            if let Some(item) = item {
                result.insert(item);
            };
        });

    let closure = closure_lr0(&result, grammar);
    if closure.is_empty() {
        None
    } else {
        Some(closure)
    }
}

// 获取当前state的所有的item的dot之后的symbol的去重集合
fn next_symbols<'a>(state: &StateLR0, grammar: &'a Grammar) -> BTreeSet<Symbol> {
    state
        .iter()
        .map(|item| item.symbol_after_dot(grammar))
        // symbol_after_dot 返回 Option<&Symbol>；flatten() 会自动跳过 None，并解开 Some。
        .flatten()
        .cloned()
        .collect::<BTreeSet<Symbol>>()
}

pub type DfaLR0 = Dfa<StateLR0>;

// 从初始项目集出发，按“圆点后可转移符号集合”逐步扩展完整的 LR(0) 项目集自动机。
pub fn build_dfa_lr0(grammar: &Grammar) -> DfaLR0 {
    let mut start = StateLR0::new();
    start.insert(ItemLR0::new(0, 0));
    let start_closure = closure_lr0(&start, grammar);

    let mut states = Vec::new();
    states.push(start_closure);

    let mut index = 0;
    let mut transitions: BTreeMap<(usize, Symbol), usize> = BTreeMap::new();

    while index < states.len() {
        // 每个状态只展开一次；新状态发现后追加到 states 末尾等待后续处理。
        let state = states.get(index).cloned().unwrap();
        for symbol in next_symbols(&state, grammar).into_iter() {
            let new_state = goto_lr0(&state, &symbol, grammar);
            if let Some(new_state) = new_state {
                // goto_lr0 返回的已经是闭包状态，这里只负责判重和编号分配。
                match states
                    .iter()
                    .enumerate()
                    .find(|(_, clo)| **clo == new_state)
                {
                    Some((target_id, _)) => {
                        // 新状态已经存在，只记录转移
                        transitions.insert((index, symbol.clone()), target_id);
                    }
                    None => {
                        states.push(new_state.clone());
                        let target_id = states.len() - 1;
                        transitions.insert((index, symbol.clone()), target_id);
                    }
                }
            }
        }
        index += 1
    }
    DfaLR0 {
        states,
        transitions,
    }
}

pub fn format_dfa_lr0(dfa: &DfaLR0, grammar: &Grammar) -> String {
    let mut lines = Vec::new();
    lines.push(format!("states: {}", dfa.states.len()));
    lines.push(String::new());

    for (index, state) in dfa.states.iter().enumerate() {
        lines.push(format!("I{index}:"));
        lines.push(state.format_with_grammar(grammar));
        lines.push(String::new());
    }

    lines.push("transitions:".to_string());
    for ((source, symbol), target) in &dfa.transitions {
        lines.push(format!("  I{source} -- {symbol} --> I{target}"));
    }

    lines.join("\n")
}

impl fmt::Display for DfaLR0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "states: {}", self.states.len())?;
        for (index, state) in self.states.iter().enumerate() {
            writeln!(f, "I{index}:")?;
            writeln!(f, "{state}")?;
        }
        writeln!(f, "transitions:")?;
        for ((source, symbol), target) in &self.transitions {
            writeln!(f, "I{source} -- {symbol} --> I{target}")?;
        }
        Ok(())
    }
}
