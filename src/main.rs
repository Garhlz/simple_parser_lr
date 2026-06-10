mod dfa;
mod dfa_lr0;
mod item_set;
mod first_follow;
mod grammar;
mod item_lr0;
mod lexer;
mod lr_parser;
mod lr_table;
mod symbol;
mod syntax_tree;
mod item_lr1;
mod dfa_lr1;

use crate::dfa_lr0::{build_dfa_lr0, format_dfa_lr0};
use crate::dfa_lr1::{build_dfa_lr1, format_dfa_lr1};
use crate::first_follow::{format_first_sets, format_follow_sets};
use crate::first_follow::get_first_set;
use crate::grammar::Grammar;
use crate::lexer::{format_tokens_verbose, tokenize};
use crate::lr_parser::{format_parse_steps, parse_lr};
use crate::lr_table::{LRTable, format_lr0_table};
use std::env;
use std::fs;
use std::fmt::Write as _;
use std::process;

const VALID_SAMPLES: &[&str] = &[
    "let x = 1;",
    "let y;",
    "x = 1 - 2 / 3;",
    "if ( not false and x >= 1 ) { } else if ( x == 10 or false ) { let z; }",
    "while ( x > 0 ) { x = x - 1; }",
    "{ let a = 1; let b; }",
    "{ }",
    "if ( x < 10 ) { let y = 1; } else if ( x == 10 ) { let z; }",
    "{ { } while ( x > 0 ) { if ( x == 1 ) { } } }",
    "let x = 1; // line comment\n/* block comment */ while ( x > 0 ) { x = x - 1; }",
];

const INVALID_SAMPLES: &[&str] = &[
    "let x = ;",
    "if ( x < 1 ) let y = 1;",
    "while ( x > 0 ) { x = ; }",
];

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args();
    let _program = args.next();

    // 命令行入口同时保留“全流程演示”和“按阶段单独查看”两种使用方式。
    match args.next().as_deref() {
        Some("all") | None => run_demo(),
        Some("all-md") => {
            let path = args
                .next()
                .unwrap_or_else(|| "demo_output.md".to_string());

            if args.next().is_some() {
                return Err("用法: cargo run -- all-md [output-file]".to_string());
            }

            let markdown = build_demo_markdown()?;
            fs::write(&path, markdown)
                .map_err(|err| format!("写入 Markdown 文件失败 `{path}`: {err}"))?;
            println!("已生成 Markdown 演示文档: {path}");
            Ok(())
        }
        Some("grammar") => {
            print!("{}", Grammar::simple_lr());
            Ok(())
        }
        Some("first-follow") => {
            let grammar = Grammar::simple_lr();
            println!("{}", format_first_sets(&grammar));
            println!();
            println!("{}", format_follow_sets(&grammar));
            Ok(())
        }
        Some("lr0-dfa") => {
            let grammar = Grammar::simple_lr();
            let dfa = build_dfa_lr0(&grammar);
            println!("{}", format_dfa_lr0(&dfa, &grammar));
            Ok(())
        }
        Some("lr1-dfa") => {
            let grammar = Grammar::simple_lr();
            let first = get_first_set(&grammar);
            let dfa = build_dfa_lr1(&grammar, &first);
            println!("{}", format_dfa_lr1(&dfa, &grammar));
            Ok(())
        }
        Some("lr0-table") => {
            let grammar = Grammar::simple_lr();
            let dfa = build_dfa_lr0(&grammar);
            let table = LRTable::build_lr0(&grammar, &dfa)?;
            println!("{}", format_lr0_table(&table, &grammar));
            Ok(())
        }
        Some("slr-table") => {
            let grammar = Grammar::simple_lr();
            let dfa = build_dfa_lr0(&grammar);
            let table = LRTable::build_slr1(&grammar, &dfa)?;
            println!("{}", format_lr0_table(&table, &grammar));
            Ok(())
        }
        Some("lr1-table") => {
            let grammar = Grammar::simple_lr();
            let first = get_first_set(&grammar);
            let dfa = build_dfa_lr1(&grammar, &first);
            let table = LRTable::build_lr1(&grammar, &dfa)?;
            println!("{}", format_lr0_table(&table, &grammar));
            Ok(())
        }
        Some("parse-slr") => {
            let path = args
                .next()
                .ok_or_else(|| "用法: cargo run -- parse-slr <source-file>".to_string())?;

            if args.next().is_some() {
                return Err("用法: cargo run -- parse-slr <source-file>".to_string());
            }

            let source = fs::read_to_string(&path)
                .map_err(|err| format!("读取源文件失败 `{path}`: {err}"))?;
            let tokens = tokenize(&source)?;
            let grammar = Grammar::simple_lr();
            let dfa = build_dfa_lr0(&grammar);
            let table = LRTable::build_slr1(&grammar, &dfa)?;
            let output = parse_lr(&grammar, &table, &tokens)
                .map_err(|err| format_parse_error(&err.to_string()))?;
            println!("{}", format_parse_steps(&output.steps));
            println!();
            println!("syntax tree:");
            println!("{}", output.tree);
            Ok(())
        }
        Some("parse-lr1") => {
            let path = args
                .next()
                .ok_or_else(|| "用法: cargo run -- parse-lr1 <source-file>".to_string())?;

            if args.next().is_some() {
                return Err("用法: cargo run -- parse-lr1 <source-file>".to_string());
            }

            let source = fs::read_to_string(&path)
                .map_err(|err| format!("读取源文件失败 `{path}`: {err}"))?;
            let tokens = tokenize(&source)?;
            let grammar = Grammar::simple_lr();
            let first = get_first_set(&grammar);
            let dfa = build_dfa_lr1(&grammar, &first);
            let table = LRTable::build_lr1(&grammar, &dfa)?;
            let output = parse_lr(&grammar, &table, &tokens)
                .map_err(|err| format_parse_error(&err.to_string()))?;
            println!("{}", format_parse_steps(&output.steps));
            println!();
            println!("syntax tree:");
            println!("{}", output.tree);
            Ok(())
        }
        Some("tokens") => {
            let path = args
                .next()
                .ok_or_else(|| "用法: cargo run -- tokens <source-file>".to_string())?;

            if args.next().is_some() {
                return Err("用法: cargo run -- tokens <source-file>".to_string());
            }

            let source = fs::read_to_string(&path)
                .map_err(|err| format!("读取源文件失败 `{path}`: {err}"))?;
            let tokens = tokenize(&source)?;
            println!("{}", format_tokens_verbose(&tokens));
            Ok(())
        }
        Some("help") => {
            print_usage();
            Ok(())
        }
        Some(other) => Err(format!("未知命令 `{other}`。\n\n{}", usage_text())),
    }
}

fn run_demo() -> Result<(), String> {
    print!("{}", build_demo_markdown()?);
    Ok(())
}

fn build_demo_markdown() -> Result<String, String> {
    let grammar = Grammar::simple_lr();
    let dfa = build_dfa_lr0(&grammar);
    let lr0_table = LRTable::build_lr0(&grammar, &dfa)?;
    let slr_table = LRTable::build_slr1(&grammar, &dfa)?;
    let first = get_first_set(&grammar);
    let dfa_lr1 = build_dfa_lr1(&grammar, &first);
    let lr1_table = LRTable::build_lr1(&grammar, &dfa_lr1)?;
    let mut out = String::new();

    // `all` / `all-md` 共用一份 Markdown 渲染，便于直接保存成实验附件或索引文档。
    writeln!(&mut out, "# Simple LR 演示输出\n").unwrap();
    writeln!(&mut out, "## 目录").unwrap();
    writeln!(&mut out, "- [LR 文法](#lr-文法)").unwrap();
    writeln!(&mut out, "- [FIRST 集](#first-集)").unwrap();
    writeln!(&mut out, "- [FOLLOW 集](#follow-集)").unwrap();
    writeln!(&mut out, "- [冲突统计](#冲突统计)").unwrap();
    writeln!(&mut out, "- [LR(0) DFA](#lr0-dfa)").unwrap();
    writeln!(&mut out, "- [LR(0) 分析表](#lr0-分析表)").unwrap();
    writeln!(&mut out, "- [SLR(1) 分析表](#slr1-分析表)").unwrap();
    writeln!(&mut out, "- [LR(1) DFA](#lr1-dfa)").unwrap();
    writeln!(&mut out, "- [LR(1) 分析表](#lr1-分析表)").unwrap();
    writeln!(&mut out, "- [词法分析](#词法分析)").unwrap();
    writeln!(&mut out, "- [SLR(1) 语法分析 - 合法样例](#slr1-语法分析---合法样例)").unwrap();
    writeln!(&mut out, "- [SLR(1) 语法分析 - 非法样例](#slr1-语法分析---非法样例)\n").unwrap();

    write_markdown_section(&mut out, "LR 文法", &grammar.to_string());
    write_markdown_section(&mut out, "FIRST 集", &format_first_sets(&grammar));
    write_markdown_section(&mut out, "FOLLOW 集", &format_follow_sets(&grammar));

    writeln!(&mut out, "## 冲突统计\n").unwrap();
    writeln!(&mut out, "- `LR(0)` conflicts: {}", lr0_table.conflicts.len()).unwrap();
    writeln!(&mut out, "- `SLR(1)` conflicts: {}", slr_table.conflicts.len()).unwrap();
    writeln!(&mut out, "- `LR(1)` conflicts: {}\n", lr1_table.conflicts.len()).unwrap();

    write_markdown_section(&mut out, "LR(0) DFA", &format_dfa_lr0(&dfa, &grammar));
    write_markdown_section(&mut out, "LR(0) 分析表", &format_lr0_table(&lr0_table, &grammar));
    write_markdown_section(&mut out, "SLR(1) 分析表", &format_lr0_table(&slr_table, &grammar));
    write_markdown_section(&mut out, "LR(1) DFA", &format_dfa_lr1(&dfa_lr1, &grammar));
    write_markdown_section(&mut out, "LR(1) 分析表", &format_lr0_table(&lr1_table, &grammar));

    writeln!(&mut out, "## 词法分析\n").unwrap();
    for (index, input) in VALID_SAMPLES.iter().enumerate() {
        writeln!(&mut out, "### 样例 {}\n", index + 1).unwrap();
        write_markdown_block(&mut out, "simple", input);
        match tokenize(input) {
            Ok(tokens) => write_markdown_block(&mut out, "text", &format_tokens_verbose(&tokens)),
            Err(err) => writeln!(&mut out, "- 词法错误: {err}\n").unwrap(),
        }
    }

    writeln!(&mut out, "## SLR(1) 语法分析 - 合法样例\n").unwrap();
    for (index, input) in VALID_SAMPLES.iter().enumerate() {
        writeln!(&mut out, "### 样例 {}\n", index + 1).unwrap();
        write_markdown_block(&mut out, "simple", input);
        match tokenize(input) {
            Ok(tokens) => match parse_lr(&grammar, &slr_table, &tokens) {
                Ok(output) => {
                    writeln!(&mut out, "- 分析结果: 接受\n").unwrap();
                    if index == 0 {
                        write_markdown_section(&mut out, "分析步骤", &format_parse_steps(&output.steps));
                    }
                    write_markdown_section(&mut out, "语法树", &output.tree.to_string());
                }
                Err(err) => {
                    writeln!(&mut out, "- 分析结果: 错误\n").unwrap();
                    writeln!(&mut out, "- {}\n", format_parse_error(&err.to_string())).unwrap();
                }
            },
            Err(err) => writeln!(&mut out, "- 词法错误: {err}\n").unwrap(),
        }
    }

    writeln!(&mut out, "## SLR(1) 语法分析 - 非法样例\n").unwrap();
    for (index, input) in INVALID_SAMPLES.iter().enumerate() {
        writeln!(&mut out, "### 样例 {}\n", index + 1).unwrap();
        write_markdown_block(&mut out, "simple", input);
        match tokenize(input) {
            Ok(tokens) => match parse_lr(&grammar, &slr_table, &tokens) {
                Ok(output) => {
                    writeln!(&mut out, "- 分析结果: 意外接受\n").unwrap();
                    write_markdown_section(&mut out, "分析步骤", &format_parse_steps(&output.steps));
                    write_markdown_section(&mut out, "语法树", &output.tree.to_string());
                }
                Err(err) => {
                    writeln!(&mut out, "- 分析结果: 错误\n").unwrap();
                    writeln!(&mut out, "- {}\n", format_parse_error(&err.to_string())).unwrap();
                }
            },
            Err(err) => writeln!(&mut out, "- 词法错误: {err}\n").unwrap(),
        }
    }

    Ok(out)
}

fn write_markdown_section(out: &mut String, title: &str, body: &str) {
    writeln!(out, "## {title}\n").unwrap();
    write_markdown_block(out, "text", body);
}

fn write_markdown_block(out: &mut String, language: &str, body: &str) {
    writeln!(out, "```{language}").unwrap();
    writeln!(out, "{body}").unwrap();
    writeln!(out, "```\n").unwrap();
}

fn print_usage() {
    println!("{}", usage_text());
}

fn format_parse_error(err: &str) -> String {
    // 这里把内部英文前缀翻译成统一的终端输出风格，避免 main 中重复分支判断。
    if err.starts_with("syntax error") {
        format!("语法错误: {err}")
    } else if err.starts_with("internal parser error") {
        format!("内部错误: {err}")
    } else {
        err.to_string()
    }
}

fn usage_text() -> &'static str {
    "\
Simple LR 实验入口

用法:
  cargo run --
  cargo run -- all
  cargo run -- all-md [output-file]
  cargo run -- grammar
  cargo run -- first-follow
  cargo run -- lr0-dfa
  cargo run -- lr1-dfa
  cargo run -- lr0-table
  cargo run -- slr-table
  cargo run -- lr1-table
  cargo run -- parse-slr <source-file>
  cargo run -- parse-lr1 <source-file>
  cargo run -- tokens <source-file>
  cargo run -- help
"
}
