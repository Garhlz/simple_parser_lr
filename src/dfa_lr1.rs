use crate::{
    dfa::Dfa,
    first_follow::first_of_sequence,
    grammar::Grammar,
    item_lr1::{ItemLR1, StateLR1},
    symbol::{NonTerminal, Symbol, Terminal},
};

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

pub fn closure_lr1(
    state: &StateLR1,
    grammar: &Grammar,
    first_set: &HashMap<NonTerminal, HashSet<Symbol>>,
) -> StateLR1 {
    let mut closure = state.clone();
    loop {
        let mut changed = false;
        let snapshot = closure.iter().cloned().collect::<Vec<ItemLR1>>();

        for item in &snapshot {
            let Some(Symbol::NonTerminal(nt)) = item.symbol_after_dot(grammar) else {
                continue;
            };

            let prod = grammar
                .production(item.production_id)
                .expect("valid production id");

            // beta 是圆点后该非终结符之后的符号串，后续会与当前 lookahead 拼接后求 FIRST(beta a)。
            let beta = &prod.rhs[item.dot + 1..];

            let mut beta_with_lookahead = beta.to_vec();
            beta_with_lookahead.push(Symbol::Terminal(item.lookahead));

            let lookahead_set = first_of_sequence(&beta_with_lookahead, first_set);

            for (prod_id, _) in grammar.productions_for(*nt) {
                for lookahead in lookahead_set.iter() {
                    let Symbol::Terminal(term) = lookahead else {
                        panic!(
                            "internal lr1 closure error: FIRST(beta a) should not contain ε or non-terminal"
                        );
                    };

                    let new_item = ItemLR1::new(prod_id, 0, *term);

                    if closure.insert(new_item) {
                        changed = true;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    closure
}

/// 把所有可在该符号上前移的项目推进一格，再对结果重新取闭包。
/// 即实际的状态转移计算过程
/// 这里需要输入first set
pub fn goto_lr1(
    state: &StateLR1,
    symbol: &Symbol,
    grammar: &Grammar,
    first_set: &HashMap<NonTerminal, HashSet<Symbol>>,
) -> Option<StateLR1> {
    let mut result = StateLR1::new();

    state
        .iter()
        .filter_map(|item| {
            // filter_map 允许把“筛选可前移项目”和“把圆点前移一格”合成一次遍历。
            if matches!(item.symbol_after_dot(grammar), Some(cur_symbol) if cur_symbol == symbol) {
                // advance_dot 本身返回 Option；filter_map 会自动跳过 None，只保留成功前移的项目。
                item.advance_dot(grammar)
            } else {
                None
            }
        })
        .for_each(|item| {
            result.insert(item);
        });

    let closure = closure_lr1(&result, grammar, first_set);

    if closure.is_empty() {
        None
    } else {
        Some(closure)
    }
}

pub type DfaLR1 = Dfa<StateLR1>;

/// 返回当前state的所有item的dot之后的symbol的集合
fn next_symbols_lr1(state: &StateLR1, grammar: &Grammar) -> BTreeSet<Symbol> {
    state
        .iter()
        // 这里直接把 Option<&Symbol> 转成 Option<Symbol>，避免后面长期携带借用。
        .filter_map(|item| item.symbol_after_dot(grammar).cloned())
        .collect()
}

/// 构建dfa的state数组和transition表，类似于构建有向图
pub fn build_dfa_lr1(
    grammar: &Grammar,
    first_set: &HashMap<NonTerminal, HashSet<Symbol>>,
) -> DfaLR1 {
    let mut start = StateLR1::new();

    // production 0: Program' -> Program
    // lookahead为 #
    start.insert(ItemLR1::new(0, 0, Terminal::End));

    let start_closure = closure_lr1(&start, grammar, first_set);

    let mut states = vec![start_closure];
    let mut transitions = BTreeMap::new();

    let mut index = 0;

    // 只有当遍历到自己的index之时才会进行当前状态节点的扩展
    while index < states.len() {
        let state = states[index].clone();

        for symbol in next_symbols_lr1(&state, grammar) {
            let Some(next_state) = goto_lr1(&state, &symbol, grammar, first_set) else {
                continue;
            };

            // position(...) 用值相等判断状态是否已存在；找不到时再分配新编号。
            let target_id = match states.iter().position(|s| *s == next_state) {
                Some(existing_id) => existing_id,
                None => {
                    states.push(next_state);
                    states.len() - 1
                }
            };

            transitions.insert((index, symbol), target_id);
        }

        index += 1;
    }

    DfaLR1 {
        states,
        transitions,
    }
}

pub fn format_dfa_lr1(dfa: &DfaLR1, grammar: &Grammar) -> String {
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

#[cfg(test)]
mod tests {
    use super::{build_dfa_lr1, closure_lr1};
    use crate::{
        first_follow::get_first_set,
        grammar::Grammar,
        item_lr1::{ItemLR1, StateLR1},
        symbol::Terminal,
    };

    #[test]
    fn lr1_start_closure_propagates_end_lookahead() {
        let grammar = Grammar::simple_lr();
        let first = get_first_set(&grammar);
        let mut start = StateLR1::new();
        start.insert(ItemLR1::new(0, 0, Terminal::End));

        let closure = closure_lr1(&start, &grammar, &first);

        assert!(closure.iter().any(|item| {
            item.production_id == 1 && item.dot == 0 && item.lookahead == Terminal::End
        }));
    }

    #[test]
    fn builds_non_empty_lr1_dfa() {
        let grammar = Grammar::simple_lr();
        let first = get_first_set(&grammar);
        let dfa = build_dfa_lr1(&grammar, &first);

        assert!(!dfa.states.is_empty());
        assert!(!dfa.transitions.is_empty());
        assert!(
            dfa.states[0]
                .iter()
                .any(|item| item.production_id == 0 && item.lookahead == Terminal::End)
        );
    }

    #[test]
    fn lr1_closure_uses_first_of_beta_and_lookahead() {
        let grammar = Grammar::simple_lr();
        let first = get_first_set(&grammar);
        let mut state = StateLR1::new();

        // Program -> · StmtList, # 触发 StmtList 的 closure，lookahead 应保持为 #
        state.insert(ItemLR1::new(1, 0, Terminal::End));

        let closure = closure_lr1(&state, &grammar, &first);

        assert!(closure.iter().any(|item| {
            item.production_id == 2 && item.dot == 0 && item.lookahead == Terminal::End
        }));
    }
}
