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
    let grammar = Grammar::simple_lr();
    let dfa = build_dfa_lr0(&grammar);
    let lr0_table = LRTable::build_lr0(&grammar, &dfa)?;
    let slr_table = LRTable::build_slr1(&grammar, &dfa)?;
    let first = get_first_set(&grammar);
    let dfa_lr1 = build_dfa_lr1(&grammar, &first);
    let lr1_table = LRTable::build_lr1(&grammar, &dfa_lr1)?;

    // all 模式按实验展示顺序串起文法、集合、DFA、分析表和样例分析。
    print_section("LR 文法");
    println!("{}\n", grammar);

    print_section("FIRST 集");
    println!("{}\n", format_first_sets(&grammar));

    print_section("FOLLOW 集");
    println!("{}\n", format_follow_sets(&grammar));

    print_section("冲突统计");
    println!("LR(0) conflicts: {}", lr0_table.conflicts.len());
    println!("SLR(1) conflicts: {}\n", slr_table.conflicts.len());
    println!("LR(1) conflicts: {}\n", lr1_table.conflicts.len());

    print_section("LR(0) DFA");
    println!("{}\n", format_dfa_lr0(&dfa, &grammar));

    print_section("LR(0) 分析表");
    println!("{}\n", format_lr0_table(&lr0_table, &grammar));

    print_section("SLR(1) 分析表");
    println!("{}\n", format_lr0_table(&slr_table, &grammar));

    print_section("LR(1) DFA");
    println!("{}\n", format_dfa_lr1(&dfa_lr1, &grammar));

    print_section("LR(1) 分析表");
    println!("{}\n", format_lr0_table(&lr1_table, &grammar));

    print_section("词法分析");
    for (index, input) in VALID_SAMPLES.iter().enumerate() {
        print_subsection(index + 1, input);
        match tokenize(input) {
            Ok(tokens) => println!("Token: {}\n", format_tokens_verbose(&tokens)),
            Err(err) => println!("词法错误: {err}\n"),
        }
    }

    print_section("SLR(1) 语法分析 - 合法样例");
    for (index, input) in VALID_SAMPLES.iter().enumerate() {
        print_subsection(index + 1, input);
        match tokenize(input) {
            Ok(tokens) => match parse_lr(&grammar, &slr_table, &tokens) {
                Ok(output) => {
                    println!("分析结果: 接受");
                    if index == 0 {
                        println!("分析步骤:\n{}", format_parse_steps(&output.steps));
                    }
                    println!("语法树:\n{}", output.tree);
                }
                Err(err) => {
                    println!("分析结果: 错误");
                    println!("{}\n", format_parse_error(&err.to_string()));
                }
            },
            Err(err) => println!("词法错误: {err}\n"),
        }
    }

    print_section("SLR(1) 语法分析 - 非法样例");
    for (index, input) in INVALID_SAMPLES.iter().enumerate() {
        print_subsection(index + 1, input);
        match tokenize(input) {
            Ok(tokens) => match parse_lr(&grammar, &slr_table, &tokens) {
                Ok(output) => {
                    println!("分析结果: 意外接受");
                    println!("分析步骤:\n{}", format_parse_steps(&output.steps));
                    println!("语法树:\n{}", output.tree);
                }
                Err(err) => {
                    println!("分析结果: 错误");
                    println!("{}\n", format_parse_error(&err.to_string()));
                }
            },
            Err(err) => println!("词法错误: {err}\n"),
        }
    }

    Ok(())
}

fn print_section(title: &str) {
    println!("\n=== {title} ===\n");
}

fn print_subsection(index: usize, input: &str) {
    println!("[样例 {index}] {input}");
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
