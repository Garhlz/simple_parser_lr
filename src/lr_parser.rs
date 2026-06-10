use crate::{
    grammar::Grammar,
    lexer::{Token, format_tokens_verbose},
    lr_table::{Action, LRTable},
    symbol::{Symbol, Terminal},
    syntax_tree::SyntaxTree,
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    Syntax,
    Internal,
}

#[derive(Debug)]
pub struct ParseStep {
    pub step_number: usize,
    pub state_stack: Vec<usize>,
    pub symbol_stack: Vec<Symbol>,
    pub rest_input: Vec<Token>,
    pub action: Action,
}

#[derive(Debug)]
pub struct ParseOutput {
    pub steps: Vec<ParseStep>,
    pub tree: SyntaxTree,
}

#[derive(Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub state_id: usize,
    pub current_token: Token,
    pub action: Option<Action>,
    pub message: String,
}

fn parse_error(
    kind: ParseErrorKind,
    state_id: usize,
    current_token: &Token,
    action: Option<&Action>,
    message: impl Into<String>,
) -> ParseError {
    ParseError {
        kind,
        state_id,
        current_token: current_token.clone(),
        action: action.cloned(),
        message: message.into(),
    }
}

fn synthetic_end_token(offset: usize) -> Token {
    Token {
        kind: Terminal::End,
        lexeme: "#".to_string(),
        offset,
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let action_text = self
            .action
            .as_ref()
            .map(ToString::to_string)
            // 没有动作时用 none 占位，保持所有错误输出的文本结构一致。
            .unwrap_or_else(|| "none".to_string());
        let error_kind = match self.kind {
            ParseErrorKind::Syntax => "syntax error",
            ParseErrorKind::Internal => "internal parser error",
        };
        let token_text = if self.current_token.kind == Terminal::Id
            || self.current_token.kind == Terminal::Num
        {
            format!("{}({})", self.current_token.kind, self.current_token.lexeme)
        } else {
            self.current_token.kind.to_string()
        };

        write!(
            f,
            "{} at state {} with token `{}` (offset {}), action {}: {}",
            error_kind,
            self.state_id,
            token_text,
            self.current_token.offset,
            action_text,
            self.message
        )
    }
}

fn format_symbol_stack(symbols: &[Symbol]) -> String {
    if symbols.is_empty() {
        return "(empty)".to_string();
    }

    symbols
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_parse_steps(steps: &[ParseStep]) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "{:>4} | {:<24} | {:<24} | {:<32} | {}",
        "step", "states", "symbols", "input", "action"
    ));
    lines.push("-".repeat(108));

    for step in steps {
        let states = step
            .state_stack
            .iter()
            .map(|state| state.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let symbols = format_symbol_stack(&step.symbol_stack);
        let input = format_tokens_verbose(&step.rest_input);

        lines.push(format!(
            "{:>4} | {:<24} | {:<24} | {:<32} | {}",
            step.step_number, states, symbols, input, step.action
        ));
    }

    lines.join("\n")
}

/// 使用已构造好的 LR/SLR 分析表驱动语法分析，并在 shift/reduce 过程中同步构造语法树。
pub fn parse_lr(
    grammar: &Grammar,
    lr_table: &LRTable,
    tokens: &[Token],
) -> Result<ParseOutput, ParseError> {
    if tokens.is_empty() {
        let token = synthetic_end_token(0);
        return Err(parse_error(
            ParseErrorKind::Syntax,
            0,
            &token,
            None,
            "empty token stream; expected at least end marker",
        ));
    }

    let mut state_stack = vec![0];
    let mut symbol_stack: Vec<Symbol> = Vec::new();
    let mut index = 0;

    let mut steps = Vec::new();
    let mut step_number = 0;

    let mut tree = SyntaxTree::new();
    let mut node_stack: Vec<usize> = Vec::new();

    loop {
        let Some(state_id) = state_stack.last().copied() else {
            break;
        };
        let fallback_token = tokens
            .last()
            .cloned()
            // 这里兜底构造一个 #，保证后续错误路径始终有一个可打印的 token。
            .unwrap_or_else(|| synthetic_end_token(0));
        let Some(token) = tokens.get(index) else {
            return Err(parse_error(
                ParseErrorKind::Syntax,
                state_id,
                &fallback_token,
                None,
                "input exhausted before accept",
            ));
        };
        let kind = token.kind;

        let Some(action) = lr_table.action.get(&(state_id, kind)) else {
            return Err(parse_error(
                ParseErrorKind::Syntax,
                state_id,
                token,
                None,
                format!("missing ACTION[{}, {}]", state_id, kind),
            ));
        };

        match action {
            Action::Shift(new_state) => {
                // Shift 会消费当前 lookahead，并同步推进状态栈、符号栈和节点栈。
                state_stack.push(*new_state);
                symbol_stack.push(Symbol::Terminal(kind));

                let node_id = tree.add_terminal(kind, &token.lexeme);
                node_stack.push(node_id);

                index += 1;
                steps.push(ParseStep {
                    step_number,
                    state_stack: state_stack.clone(),
                    symbol_stack: symbol_stack.clone(),
                    rest_input: tokens[index..].to_vec(),
                    action: action.clone(),
                });
            }
            Action::Reduce(prod_id) => {
                let Some(prod) = grammar.production(*prod_id) else {
                    return Err(parse_error(
                        ParseErrorKind::Internal,
                        state_id,
                        token,
                        Some(action),
                        format!("invalid production id {}", prod_id),
                    ));
                };

                let rhs_len = prod.rhs_len();
                let mut popped_symbols = Vec::new();
                let mut popped_nodes = Vec::new();

                // Reduce 按产生式右部长度回退，再通过 GOTO 决定归约后的目标状态。
                for _ in 0..rhs_len {
                    if state_stack.pop().is_none() {
                        return Err(parse_error(
                            ParseErrorKind::Internal,
                            state_id,
                            token,
                            Some(action),
                            "state stack underflow during reduce",
                        ));
                    }

                    let Some(symbol) = symbol_stack.pop() else {
                        return Err(parse_error(
                            ParseErrorKind::Internal,
                            state_id,
                            token,
                            Some(action),
                            "symbol stack underflow during reduce",
                        ));
                    };
                    popped_symbols.push(symbol);

                    let Some(node_id) = node_stack.pop() else {
                        return Err(parse_error(
                            ParseErrorKind::Internal,
                            state_id,
                            token,
                            Some(action),
                            "node stack underflow during reduce",
                        ));
                    };
                    popped_nodes.push(node_id);
                }

                if prod.rhs != popped_symbols.into_iter().rev().collect::<Vec<_>>() {
                    // reduce 弹栈是从右往左弹出的，这里 reverse 后才能和产生式右部做顺序一致的比较。
                    return Err(parse_error(
                        ParseErrorKind::Internal,
                        state_id,
                        token,
                        Some(action),
                        format!("reduced symbols do not match production {}", prod),
                    ));
                }

                let parent_id = if prod.rhs.is_empty() {
                    let epsilon_id = tree.add_epsilon();
                    tree.add_non_terminal(prod.lhs, vec![epsilon_id])
                } else {
                    // 语法树孩子节点同样需要 reverse，才能恢复成文法右部原本的从左到右顺序。
                    tree.add_non_terminal(prod.lhs, popped_nodes.into_iter().rev().collect())
                };

                symbol_stack.push(Symbol::NonTerminal(prod.lhs));
                node_stack.push(parent_id);

                let Some(stack_top) = state_stack.last().copied() else {
                    return Err(parse_error(
                        ParseErrorKind::Internal,
                        state_id,
                        token,
                        Some(action),
                        "state stack is empty after reduce",
                    ));
                };

                let Some(new_state) = lr_table.goto.get(&(stack_top, prod.lhs)) else {
                    return Err(parse_error(
                        ParseErrorKind::Internal,
                        state_id,
                        token,
                        Some(action),
                        format!("missing GOTO[{}, {}]", stack_top, prod.lhs),
                    ));
                };

                state_stack.push(*new_state);
                steps.push(ParseStep {
                    step_number,
                    state_stack: state_stack.clone(),
                    symbol_stack: symbol_stack.clone(),
                    rest_input: tokens[index..].to_vec(),
                    action: action.clone(),
                });
            }
            Action::Accept => {
                // Accept 时，节点栈顶应已经对应完整语法树的根。
                if node_stack.len() != 1 {
                    return Err(parse_error(
                        ParseErrorKind::Internal,
                        state_id,
                        token,
                        Some(action),
                        format!(
                            "node stack should contain exactly one root at accept, found {}",
                            node_stack.len()
                        ),
                    ));
                }

                let Some(root) = node_stack.last().copied() else {
                    return Err(parse_error(
                        ParseErrorKind::Internal,
                        state_id,
                        token,
                        Some(action),
                        "node stack is empty at accept",
                    ));
                };

                tree.set_root(root);
                steps.push(ParseStep {
                    step_number,
                    state_stack: state_stack.clone(),
                    symbol_stack: symbol_stack.clone(),
                    rest_input: tokens[index..].to_vec(),
                    action: action.clone(),
                });

                return Ok(ParseOutput { steps, tree });
            }
        }

        step_number += 1;
    }

    Err(parse_error(
        ParseErrorKind::Internal,
        0,
        &tokens
            .last()
            .cloned()
            .unwrap_or_else(|| synthetic_end_token(0)),
        None,
        "parser stopped before accept",
    ))
}

#[cfg(test)]
mod tests {
    use super::{ParseErrorKind, format_parse_steps, parse_lr};
    use crate::first_follow::get_first_set;
    use crate::grammar::Grammar;
    use crate::lexer::{Token, tokenize};
    use crate::lr_table::{Action, LRTable};
    use crate::symbol::{NonTerminal, Terminal};
    use crate::{dfa_lr0::build_dfa_lr0, dfa_lr1::build_dfa_lr1};

    fn build_slr_table() -> LRTable {
        let grammar = Grammar::simple_lr();
        let dfa = build_dfa_lr0(&grammar);
        LRTable::build_slr1(&grammar, &dfa).expect("slr table should build")
    }

    fn parse_ok(input: &str) -> super::ParseOutput {
        let grammar = Grammar::simple_lr();
        let table = build_slr_table();
        let tokens = tokenize(input).expect("tokenize should succeed");
        parse_lr(&grammar, &table, &tokens).expect("parse should succeed")
    }

    fn build_lr1_table() -> LRTable {
        let grammar = Grammar::simple_lr();
        let first = get_first_set(&grammar);
        let dfa = build_dfa_lr1(&grammar, &first);
        LRTable::build_lr1(&grammar, &dfa).expect("lr1 table should build")
    }

    fn parse_ok_tree(input: &str) -> String {
        parse_ok(input).tree.to_string()
    }

    #[test]
    fn parses_valid_declaration() {
        let output = parse_ok("let x = 1;");

        assert!(!output.steps.is_empty());
        assert!(matches!(
            output.steps.last().map(|step| &step.action),
            Some(Action::Accept)
        ));
        assert!(output.tree.root.is_some());
    }

    #[test]
    fn formats_parse_steps_table() {
        let output = parse_ok("let x = 1;");
        let formatted = format_parse_steps(&output.steps);

        assert!(formatted.contains("step"));
        assert!(formatted.contains("states"));
        assert!(formatted.contains("s2"));
        assert!(formatted.contains("acc"));
    }

    #[test]
    fn parses_declaration_with_epsilon_initializer() {
        let tree_text = parse_ok_tree("let x;");

        assert!(tree_text.contains("DeclInit"));
        assert!(tree_text.contains("ε"));
    }

    #[test]
    fn parses_assignment_with_operator_precedence() {
        let tree_text = parse_ok_tree("x = 1 - 2 / 3;");

        assert!(tree_text.contains("AssignStmt"));
        assert!(tree_text.contains("-"));
        assert!(tree_text.contains("/"));
    }

    #[test]
    fn parses_if_else_if_statement() {
        let tree_text =
            parse_ok_tree("if ( x < 10 ) { let y = 1; } else if ( x == 10 ) { let z; }");

        assert!(tree_text.contains("IfStmt"));
        assert!(tree_text.contains("ElsePart"));
        assert!(tree_text.contains("=="));
    }

    #[test]
    fn parses_while_statement() {
        let tree_text = parse_ok_tree("while ( x > 0 ) { x = x - 1; }");

        assert!(tree_text.contains("WhileStmt"));
        assert!(tree_text.contains(">"));
    }

    #[test]
    fn parses_block_statement() {
        let tree_text = parse_ok_tree("{ let a = 1; let b; }");

        assert!(tree_text.contains("Block"));
        assert!(tree_text.contains("StmtList"));
    }

    #[test]
    fn parses_empty_block() {
        let tree_text = parse_ok_tree("{ }");

        assert!(tree_text.contains("Block"));
        assert!(tree_text.contains("{"));
        assert!(tree_text.contains("}"));
    }

    #[test]
    fn parses_nested_if_while_with_bool_precedence() {
        let tree_text =
            parse_ok_tree("if ( not false and x >= 1 ) { while ( x > 0 ) { if ( x == 1 ) { } } }");

        assert!(tree_text.contains("IfStmt"));
        assert!(tree_text.contains("WhileStmt"));
        assert!(tree_text.contains("not"));
        assert!(tree_text.contains("and"));
        assert!(tree_text.contains(">="));
    }

    #[test]
    fn parses_multi_statement_program_with_comments() {
        let tree_text = parse_ok_tree(
            "let x = 1; // line comment\n/* block comment */ while ( x > 0 ) { x = x - 1; }",
        );

        assert!(tree_text.contains("DeclStmt"));
        assert!(tree_text.contains("WhileStmt"));
        assert!(tree_text.contains("StmtList"));
    }

    #[test]
    fn parses_valid_input_with_lr1_table() {
        let grammar = Grammar::simple_lr();
        let table = build_lr1_table();
        let tokens = tokenize("let x = 1;").expect("tokenize should succeed");

        let output = parse_lr(&grammar, &table, &tokens).expect("parse should succeed");

        assert!(matches!(
            output.steps.last().map(|step| &step.action),
            Some(Action::Accept)
        ));
        assert!(output.tree.to_string().contains("DeclStmt"));
    }

    #[test]
    fn rejects_invalid_statement() {
        let grammar = Grammar::simple_lr();
        let table = build_slr_table();
        let tokens = tokenize("let x = ;").expect("tokenize should succeed");

        let err = parse_lr(&grammar, &table, &tokens).expect_err("parse should fail");

        assert_eq!(err.kind, ParseErrorKind::Syntax);
        assert!(err.message.contains("missing ACTION"));
        assert_eq!(err.current_token.lexeme, ";");
    }

    #[test]
    fn rejects_missing_block_after_if() {
        let grammar = Grammar::simple_lr();
        let table = build_slr_table();
        let tokens = tokenize("if ( x < 1 ) let y = 1;").expect("tokenize should succeed");

        let err = parse_lr(&grammar, &table, &tokens).expect_err("parse should fail");

        assert_eq!(err.kind, ParseErrorKind::Syntax);
        assert!(err.message.contains("missing ACTION"));
    }

    #[test]
    fn reports_missing_goto_with_action() {
        let grammar = Grammar::simple_lr();
        let mut table = build_slr_table();
        table.goto.remove(&(0, NonTerminal::Statement));
        let tokens = tokenize("let x = 1;").expect("tokenize should succeed");

        let err = parse_lr(&grammar, &table, &tokens).expect_err("parse should fail");
        let display = err.to_string();

        assert_eq!(err.kind, ParseErrorKind::Internal);
        assert!(err.message.contains("missing GOTO"));
        assert!(display.contains("action r4"));
        assert!(display.contains("internal parser error"));
        assert!(display.contains("token `"));
    }

    #[test]
    fn rejects_empty_token_stream() {
        let grammar = Grammar::simple_lr();
        let table = build_slr_table();

        let err = parse_lr(&grammar, &table, &[]).expect_err("parse should fail");

        assert_eq!(err.kind, ParseErrorKind::Syntax);
        assert!(err.message.contains("empty token stream"));
    }

    #[test]
    fn rejects_input_without_end_marker() {
        let grammar = Grammar::simple_lr();
        let table = build_slr_table();
        let tokens = vec![
            Token {
                kind: Terminal::Let,
                lexeme: "let".to_string(),
                offset: 0,
            },
            Token {
                kind: Terminal::Id,
                lexeme: "x".to_string(),
                offset: 4,
            },
        ];

        let err = parse_lr(&grammar, &table, &tokens).expect_err("parse should fail");

        assert_eq!(err.kind, ParseErrorKind::Syntax);
        assert!(err.message.contains("input exhausted before accept"));
    }

    #[test]
    fn formats_parse_error_text() {
        let grammar = Grammar::simple_lr();
        let table = build_slr_table();
        let tokens = tokenize("let x = ;").expect("tokenize should succeed");

        let err = parse_lr(&grammar, &table, &tokens).expect_err("parse should fail");
        let display = err.to_string();

        assert!(display.contains("syntax error"));
        assert!(display.contains("state"));
        assert!(display.contains("token `;`"));
        assert!(display.contains("action none"));
        assert!(display.contains("missing ACTION"));
    }
}
