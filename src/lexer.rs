use crate::symbol::Terminal;

/// Token 结构：词法分析器产出的最小单元。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: Terminal,
    pub lexeme: String,
    pub offset: usize,
}

fn push_token(tokens: &mut Vec<Token>, kind: Terminal, lexeme: String, offset: usize) {
    tokens.push(Token {
        kind,
        lexeme,
        offset,
    });
}

/// 将源码字符串转换为 Token 序列，末尾自动添加 End 标记。
pub fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let mut chars = src.chars().peekable();
    let mut buf = String::new();
    let mut tokens = Vec::new();
    let mut offset: usize = 0;

    while let Some(ch) = chars.next() {
        let start = offset;

        match ch {
            ch if ch.is_whitespace() => {
                offset += ch.len_utf8();
            }

            ch if ch.is_ascii_alphabetic() || ch == '_' => {
                offset += ch.len_utf8();
                buf.push(ch);

                while let Some(next_ch) = chars.next_if(|c| c.is_ascii_alphanumeric() || *c == '_')
                {
                    offset += next_ch.len_utf8();
                    buf.push(next_ch);
                }

                let ident = std::mem::take(&mut buf);
                let kind = match ident.as_str() {
                    "let" => Terminal::Let,
                    "if" => Terminal::If,
                    "else" => Terminal::Else,
                    "while" => Terminal::While,
                    "true" => Terminal::True,
                    "false" => Terminal::False,
                    "and" => Terminal::And,
                    "or" => Terminal::Or,
                    "not" => Terminal::Not,
                    _ => Terminal::Id,
                };

                push_token(&mut tokens, kind, ident, start);
            }

            ch if ch.is_ascii_digit() => {
                offset += ch.len_utf8();
                buf.push(ch);

                while let Some(next_ch) = chars.next_if(|c| c.is_ascii_digit()) {
                    offset += next_ch.len_utf8();
                    buf.push(next_ch);
                }

                if let Some(next_ch) = chars.peek().copied() {
                    if next_ch.is_ascii_alphabetic() || next_ch == '_' {
                        return Err(format!(
                            "词法错误 (offset {offset}): 数字后面不能紧跟字母 '{}'",
                            next_ch
                        ));
                    }
                }

                let num_text = std::mem::take(&mut buf);
                push_token(&mut tokens, Terminal::Num, num_text, start);
            }

            '+' => {
                offset += ch.len_utf8();
                push_token(&mut tokens, Terminal::Plus, "+".into(), start);
            }

            '-' => {
                offset += ch.len_utf8();
                push_token(&mut tokens, Terminal::Minus, "-".into(), start);
            }

            '*' => {
                offset += ch.len_utf8();
                push_token(&mut tokens, Terminal::Star, "*".into(), start);
            }

            '/' => {
                offset += ch.len_utf8();

                if chars.next_if_eq(&'/').is_some() {
                    offset += '/'.len_utf8();

                    for next_ch in chars.by_ref() {
                        offset += next_ch.len_utf8();
                        if next_ch == '\n' {
                            break;
                        }
                    }
                } else if chars.next_if_eq(&'*').is_some() {
                    offset += '*'.len_utf8();

                    let mut prev = '\0';
                    let mut closed = false;

                    for next_ch in chars.by_ref() {
                        offset += next_ch.len_utf8();

                        if prev == '*' && next_ch == '/' {
                            closed = true;
                            break;
                        }

                        prev = next_ch;
                    }

                    if !closed {
                        return Err(format!("词法错误 (offset {start}): 多行注释未闭合"));
                    }
                } else {
                    push_token(&mut tokens, Terminal::Slash, "/".into(), start);
                }
            }

            '=' => {
                offset += ch.len_utf8();

                if chars.next_if_eq(&'=').is_some() {
                    offset += '='.len_utf8();
                    push_token(&mut tokens, Terminal::Eq, "==".into(), start);
                } else {
                    push_token(&mut tokens, Terminal::Assign, "=".into(), start);
                }
            }

            '!' => {
                offset += ch.len_utf8();

                if chars.next_if_eq(&'=').is_some() {
                    offset += '='.len_utf8();
                    push_token(&mut tokens, Terminal::Ne, "!=".into(), start);
                } else {
                    return Err(format!(
                        "词法错误 (offset {start}): 单独的 '!' 不合法，请使用 'not' 或 '!='"
                    ));
                }
            }

            '<' => {
                offset += ch.len_utf8();

                if chars.next_if_eq(&'=').is_some() {
                    offset += '='.len_utf8();
                    push_token(&mut tokens, Terminal::Le, "<=".into(), start);
                } else {
                    push_token(&mut tokens, Terminal::Lt, "<".into(), start);
                }
            }

            '>' => {
                offset += ch.len_utf8();

                if chars.next_if_eq(&'=').is_some() {
                    offset += '='.len_utf8();
                    push_token(&mut tokens, Terminal::Ge, ">=".into(), start);
                } else {
                    push_token(&mut tokens, Terminal::Gt, ">".into(), start);
                }
            }

            '(' => {
                offset += ch.len_utf8();
                push_token(&mut tokens, Terminal::LParen, "(".into(), start);
            }

            ')' => {
                offset += ch.len_utf8();
                push_token(&mut tokens, Terminal::RParen, ")".into(), start);
            }

            '{' => {
                offset += ch.len_utf8();
                push_token(&mut tokens, Terminal::LBrace, "{".into(), start);
            }

            '}' => {
                offset += ch.len_utf8();
                push_token(&mut tokens, Terminal::RBrace, "}".into(), start);
            }

            ';' => {
                offset += ch.len_utf8();
                push_token(&mut tokens, Terminal::Semicolon, ";".into(), start);
            }

            other => {
                return Err(format!("词法错误 (offset {start}): 非法字符 '{}'", other));
            }
        }
    }

    push_token(&mut tokens, Terminal::End, "#".into(), offset);

    Ok(tokens)
}

/// 将 token 序列格式化为包含词素的字符串，便于调试语法树。
///
/// 示例：
/// `id(x) = num(1) ; #`
pub fn format_tokens_verbose(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|token| {
            if token.kind == Terminal::Id || token.kind == Terminal::Num {
                format!("{}({})", token.kind, token.lexeme)
            } else {
                token.kind.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{format_tokens_verbose, tokenize};
    use crate::symbol::Terminal;

    #[test]
    fn tokenizes_simple_declaration() {
        let tokens = tokenize("let x = 1;").expect("tokenize should succeed");
        let kinds: Vec<Terminal> = tokens.iter().map(|t| t.kind).collect();

        assert_eq!(
            kinds,
            vec![
                Terminal::Let,
                Terminal::Id,
                Terminal::Assign,
                Terminal::Num,
                Terminal::Semicolon,
                Terminal::End,
            ]
        );
    }

    #[test]
    fn tokenizes_arithmetic_expression() {
        let tokens = tokenize("x = 1 - 2 / 3;").expect("tokenize should succeed");
        let kinds: Vec<Terminal> = tokens.iter().map(|t| t.kind).collect();

        assert_eq!(
            kinds,
            vec![
                Terminal::Id,
                Terminal::Assign,
                Terminal::Num,
                Terminal::Minus,
                Terminal::Num,
                Terminal::Slash,
                Terminal::Num,
                Terminal::Semicolon,
                Terminal::End,
            ]
        );
    }

    #[test]
    fn tokenizes_if_else_expression() {
        let tokens = tokenize("if ( x < 1 or false ) { let y = 1; } else if ( x == 2 ) { let z; }")
            .expect("tokenize should succeed");
        let kinds: Vec<Terminal> = tokens.iter().map(|t| t.kind).collect();

        assert_eq!(
            kinds,
            vec![
                Terminal::If,
                Terminal::LParen,
                Terminal::Id,
                Terminal::Lt,
                Terminal::Num,
                Terminal::Or,
                Terminal::False,
                Terminal::RParen,
                Terminal::LBrace,
                Terminal::Let,
                Terminal::Id,
                Terminal::Assign,
                Terminal::Num,
                Terminal::Semicolon,
                Terminal::RBrace,
                Terminal::Else,
                Terminal::If,
                Terminal::LParen,
                Terminal::Id,
                Terminal::Eq,
                Terminal::Num,
                Terminal::RParen,
                Terminal::LBrace,
                Terminal::Let,
                Terminal::Id,
                Terminal::Semicolon,
                Terminal::RBrace,
                Terminal::End,
            ]
        );
    }

    #[test]
    fn tokenizes_keywords() {
        let tokens =
            tokenize("let if else while true false and or not").expect("tokenize should succeed");
        let kinds: Vec<Terminal> = tokens.iter().map(|t| t.kind).collect();

        assert_eq!(
            kinds,
            vec![
                Terminal::Let,
                Terminal::If,
                Terminal::Else,
                Terminal::While,
                Terminal::True,
                Terminal::False,
                Terminal::And,
                Terminal::Or,
                Terminal::Not,
                Terminal::End,
            ]
        );
    }

    #[test]
    fn tokenizes_comments() {
        let tokens = tokenize(
            "let x = 1; // line comment\n/* block\ncomment */ while ( x > 0 ) { x = x - 1; }",
        )
        .expect("tokenize should succeed");
        let kinds: Vec<Terminal> = tokens.iter().map(|t| t.kind).collect();

        assert_eq!(
            kinds,
            vec![
                Terminal::Let,
                Terminal::Id,
                Terminal::Assign,
                Terminal::Num,
                Terminal::Semicolon,
                Terminal::While,
                Terminal::LParen,
                Terminal::Id,
                Terminal::Gt,
                Terminal::Num,
                Terminal::RParen,
                Terminal::LBrace,
                Terminal::Id,
                Terminal::Assign,
                Terminal::Id,
                Terminal::Minus,
                Terminal::Num,
                Terminal::Semicolon,
                Terminal::RBrace,
                Terminal::End,
            ]
        );
    }

    #[test]
    fn formats_tokens_with_lexemes() {
        let tokens = tokenize("let x = 1 >= 2;").expect("tokenize should succeed");
        assert_eq!(
            format_tokens_verbose(&tokens),
            "let id(x) = num(1) >= num(2) ; #"
        );
    }

    #[test]
    fn tokenizes_bool_ops_and_empty_block() {
        let tokens = tokenize("if ( not false and x >= 1 ) { }").expect("tokenize should succeed");
        let kinds: Vec<Terminal> = tokens.iter().map(|t| t.kind).collect();

        assert_eq!(
            kinds,
            vec![
                Terminal::If,
                Terminal::LParen,
                Terminal::Not,
                Terminal::False,
                Terminal::And,
                Terminal::Id,
                Terminal::Ge,
                Terminal::Num,
                Terminal::RParen,
                Terminal::LBrace,
                Terminal::RBrace,
                Terminal::End,
            ]
        );
    }

    #[test]
    fn tokenizes_underscore_identifier() {
        let tokens = tokenize("let _x1 = 42;").expect("tokenize should succeed");
        assert_eq!(tokens[1].kind, Terminal::Id);
        assert_eq!(tokens[1].lexeme, "_x1");
    }

    #[test]
    fn rejects_digits_followed_by_identifier_characters() {
        let err = tokenize("let x = 123abc;").expect_err("tokenize should fail");
        assert!(err.contains("数字后面不能紧跟字母"));
    }

    #[test]
    fn rejects_unclosed_block_comment() {
        let err = tokenize("let x = 1; /* unclosed").expect_err("tokenize should fail");
        assert!(err.contains("多行注释未闭合"));
    }

    #[test]
    fn rejects_illegal_character() {
        let err = tokenize("let x = @;").expect_err("tokenize should fail");
        assert!(err.contains("非法字符"));
    }

    #[test]
    fn rejects_bare_exclamation() {
        let err = tokenize("if ( !x ) { }").expect_err("tokenize should fail");
        assert!(err.contains("单独的 '!' 不合法"));
    }
}
