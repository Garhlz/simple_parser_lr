use crate::grammar::Grammar;
use crate::symbol::{NonTerminal, Symbol, Terminal};
use std::collections::{HashMap, HashSet};

pub fn first_of_sequence(
    symbols: &[Symbol],
    first_set_map: &HashMap<NonTerminal, HashSet<Symbol>>,
) -> HashSet<Symbol> {
    let mut result = HashSet::new();
    if symbols.is_empty() {
        result.insert(Symbol::Epsilon);
        return result;
    }
    for symbol in symbols {
        match symbol {
            Symbol::Terminal(term) => {
                result.insert(Symbol::Terminal(*term));
                return result;
            }
            Symbol::NonTerminal(non_terminal) => {
                let first_set = first_set_map
                    .get(non_terminal)
                    .expect("non-terminal missing from FIRST set map");
                result.extend(
                    first_set
                        .iter()
                        .filter(|symbol| **symbol != Symbol::Epsilon)
                        .cloned(),
                );
                // 如果当前非终结符可为空，继续考虑下一个
                if first_set.contains(&Symbol::Epsilon) {
                    continue;
                }
                return result;
            }
            Symbol::Epsilon => {
                // `rhs.is_empty()` 才是当前项目里约定的 ε 产生式表示；
                // 继续看后继符号，而不是立刻返回 FIRST = { ε }。
                continue;
            }
        }
    }
    result.insert(Symbol::Epsilon);
    result
}

pub fn get_first_set(grammar: &Grammar) -> HashMap<NonTerminal, HashSet<Symbol>> {
    let mut first_set_map: HashMap<NonTerminal, HashSet<Symbol>> = HashMap::new();
    // 初始化
    for production in &grammar.productions {
        first_set_map.entry(production.lhs).or_default();
    }

    loop {
        let mut changed = false;

        for production in &grammar.productions {
            // 这里直接用 first_of_sequence 求rhs整体的first集合
            let derived = first_of_sequence(&production.rhs, &first_set_map);
            let lhs_first = first_set_map.entry(production.lhs).or_default();
            let previous_len = lhs_first.len();
            lhs_first.extend(derived); // 尝试扩展，查看大小是否变化

            if lhs_first.len() != previous_len {
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    first_set_map
}

pub fn get_follow_set(
    grammar: &Grammar,
    first_set: &HashMap<NonTerminal, HashSet<Symbol>>,
) -> HashMap<NonTerminal, HashSet<Symbol>> {
    let mut follow_set: HashMap<NonTerminal, HashSet<Symbol>> = HashMap::new();
    // 初始化
    for production in &grammar.productions {
        follow_set.entry(production.lhs).or_default();
        for symbol in &production.rhs {
            if let Symbol::NonTerminal(non_terminal) = symbol {
                follow_set.entry(*non_terminal).or_default();
            }
        }
    }
    // 把输入结束符 # 加入开始符号的 Follow 集
    let start_follow: &mut HashSet<Symbol> = follow_set.get_mut(&grammar.start).unwrap();
    start_follow.insert(Symbol::Terminal(Terminal::End));

    loop {
        let mut changed = false;

        for production in &grammar.productions {
            let lhs_follow = follow_set.get(&production.lhs).cloned().unwrap_or_default();

            for index in 0..production.rhs.len() {
                // 当前求follow集的非终结符
                let Symbol::NonTerminal(non_terminal) = production.rhs[index] else {
                    continue;
                };
                // 同理，这里也直接用 first_of_sequence 直接求出后缀符号串整体的first集
                let suffix_first = first_of_sequence(&production.rhs[index + 1..], first_set);
                // 取出之前的 follow集，尝试进行扩展，然后和之前的长度进行对比
                let non_terminal_follow = follow_set.get_mut(&non_terminal).unwrap();
                let previous_len = non_terminal_follow.len();

                non_terminal_follow.extend(
                    suffix_first
                        .iter()
                        .filter(|symbol| **symbol != Symbol::Epsilon)
                        .cloned(),
                );

                if suffix_first.contains(&Symbol::Epsilon) {
                    non_terminal_follow.extend(lhs_follow.iter().cloned());
                }

                if non_terminal_follow.len() != previous_len {
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }

    follow_set
}

fn format_symbol_set(set: &HashSet<Symbol>) -> String {
    let mut items: Vec<String> = set.iter().map(ToString::to_string).collect();
    items.sort();
    format!("{{ {} }}", items.join(", "))
}

pub fn format_first_sets(grammar: &Grammar) -> String {
    let first = get_first_set(grammar);
    let mut lines = Vec::new();
    for &non_terminal in NonTerminal::all() {
        if let Some(set) = first.get(&non_terminal) {
            lines.push(format!(
                "FIRST({}) = {}",
                non_terminal,
                format_symbol_set(set)
            ));
        }
    }
    lines.join("\n")
}

pub fn format_follow_sets(grammar: &Grammar) -> String {
    let first = get_first_set(grammar);
    let follow = get_follow_set(grammar, &first);
    let mut lines = Vec::new();
    for &non_terminal in NonTerminal::all() {
        if let Some(set) = follow.get(&non_terminal) {
            lines.push(format!(
                "FOLLOW({}) = {}",
                non_terminal,
                format_symbol_set(set)
            ));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{first_of_sequence, get_first_set, get_follow_set};
    use crate::grammar::Grammar;
    use crate::symbol::{NonTerminal, Symbol, Terminal};
    use std::collections::HashSet;

    fn contains_all(set: &HashSet<Symbol>, expected: &[Symbol]) -> bool {
        expected.iter().all(|symbol| set.contains(symbol))
    }

    #[test]
    fn first_set_keeps_nullable_nonterminal() {
        let grammar = Grammar::simple_lr();
        let first = get_first_set(&grammar);

        let decl_init = first.get(&NonTerminal::DeclInit).unwrap();
        assert!(contains_all(
            decl_init,
            &[Symbol::Terminal(Terminal::Assign), Symbol::Epsilon]
        ));

        let else_part = first.get(&NonTerminal::ElsePart).unwrap();
        assert!(contains_all(
            else_part,
            &[Symbol::Terminal(Terminal::Else), Symbol::Epsilon]
        ));
    }

    #[test]
    fn first_of_sequence_propagates_through_nullable_prefix() {
        let grammar = Grammar::simple_lr();
        let first = get_first_set(&grammar);
        let sequence = vec![
            Symbol::NonTerminal(NonTerminal::DeclInit),
            Symbol::Terminal(Terminal::Semicolon),
        ];

        let derived = first_of_sequence(&sequence, &first);
        assert!(contains_all(
            &derived,
            &[
                Symbol::Terminal(Terminal::Assign),
                Symbol::Terminal(Terminal::Semicolon),
            ]
        ));
        assert!(!derived.contains(&Symbol::Epsilon));
    }

    #[test]
    fn follow_set_uses_nullable_suffix() {
        let grammar = Grammar::simple_lr();
        let first = get_first_set(&grammar);
        let follow = get_follow_set(&grammar, &first);

        let decl_init_follow = follow.get(&NonTerminal::DeclInit).unwrap();
        assert!(decl_init_follow.contains(&Symbol::Terminal(Terminal::Semicolon)));

        let block_follow = follow.get(&NonTerminal::Block).unwrap();
        assert!(contains_all(
            block_follow,
            &[
                Symbol::Terminal(Terminal::Else),
                Symbol::Terminal(Terminal::End),
            ]
        ));
    }
}
