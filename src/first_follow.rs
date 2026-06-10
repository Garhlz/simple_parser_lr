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
                let Some(first_set) = first_set_map.get(non_terminal) else {
                    continue;
                };

                result.extend(
                    first_set
                        .iter()
                        .filter(|symbol| **symbol != Symbol::Epsilon)
                        .cloned(),
                );

                if first_set.contains(&Symbol::Epsilon) {
                    continue;
                }

                return result;
            }
            Symbol::Epsilon => {
                result.insert(Symbol::Epsilon);
                return result;
            }
        }
    }

    result.insert(Symbol::Epsilon);
    result
}

pub fn get_first_set(grammar: &Grammar) -> HashMap<NonTerminal, HashSet<Symbol>> {
    let mut first_set_map: HashMap<NonTerminal, HashSet<Symbol>> = HashMap::new();

    for production in &grammar.productions {
        first_set_map.entry(production.lhs).or_default();
    }

    loop {
        let mut changed = false;

        for production in &grammar.productions {
            let derived = first_of_sequence(&production.rhs, &first_set_map);
            let lhs_first = first_set_map.entry(production.lhs).or_default();
            let previous_len = lhs_first.len();
            lhs_first.extend(derived);

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
    let mut follow_set = HashMap::new();

    for production in &grammar.productions {
        follow_set.entry(production.lhs).or_default();
        for symbol in &production.rhs {
            if let Symbol::NonTerminal(non_terminal) = symbol {
                follow_set.entry(*non_terminal).or_default();
            }
        }
    }

    let start_follow: &mut HashSet<Symbol> = follow_set.get_mut(&grammar.start).unwrap();
    start_follow.insert(Symbol::Terminal(Terminal::End));

    loop {
        let mut changed = false;

        for production in &grammar.productions {
            let lhs_follow = follow_set.get(&production.lhs).cloned().unwrap_or_default();

            for index in 0..production.rhs.len() {
                let Symbol::NonTerminal(non_terminal) = production.rhs[index] else {
                    continue;
                };

                let suffix_first = first_of_sequence(&production.rhs[index + 1..], first_set);
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
