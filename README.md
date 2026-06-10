# Simple LR Parser

一个面向 `Simple` 左递归文法的 Rust 语法分析实验项目，当前已实现：

* 词法分析
* `FIRST` / `FOLLOW`
* LR(0) 项目集、DFA、分析表
* SLR(1) 分析表与表驱动分析
* Canonical LR(1) 项目集、DFA、分析表
* 基于 SLR(1) / LR(1) 的语法树构造

## 运行

在仓库根目录执行：

```bash
cargo run --
```

默认会运行内置演示，输出：

* 文法
* `FIRST` / `FOLLOW`
* LR(0) / LR(1) DFA
* LR(0) / SLR(1) / LR(1) 分析表
* 多组样例的词法分析和语法分析结果
* 以上内容现在按 Markdown 文档风格输出，便于重定向保存和跳转

常用命令：

```bash
cargo run -- grammar
cargo run -- first-follow
cargo run -- all-md [output-file]
cargo run -- lr0-dfa
cargo run -- lr1-dfa
cargo run -- lr0-table
cargo run -- slr-table
cargo run -- lr1-table
cargo run -- tokens <source-file>
cargo run -- parse-slr <source-file>
cargo run -- parse-lr1 <source-file>
```

可以直接配合仓库里的样例文件运行：

```bash
cargo run -- parse-slr examples/valid.simple
cargo run -- parse-lr1 examples/valid.simple
cargo run -- parse-slr examples/invalid.simple
```

## 项目结构

核心模块如下：

* `src/grammar.rs`：Simple 左递归文法与拓广开始符号
* `src/lexer.rs`：词法分析、注释跳过、词法错误处理
* `src/first_follow.rs`：`FIRST` / `FOLLOW` 与 `first_of_sequence`
* `src/item_lr0.rs` / `src/item_lr1.rs`：LR(0) / LR(1) 项目定义
* `src/dfa_lr0.rs` / `src/dfa_lr1.rs`：项目集闭包、`goto`、DFA 构造
* `src/lr_table.rs`：LR(0) / SLR(1) / LR(1) ACTION / GOTO 表构造
* `src/lr_parser.rs`：表驱动分析主循环、步骤记录、错误处理
* `src/syntax_tree.rs`：节点池语法树
* `src/item_set.rs` / `src/dfa.rs`：项目集和 DFA 的通用壳子

## 测试

```bash
cargo test
```

当前测试覆盖：

* lexer 成功路径与错误路径
* LR(0) / SLR(1) / LR(1) 构表
* LR(1) closure / DFA
* parser 成功路径、错误路径、步骤输出和语法树输出

## 文档

* [docs/SPEC.md](docs/SPEC.md)：当前实现说明
* [docs/TODO.md](docs/TODO.md)：当前已完成内容和后续可选工作

如果希望直接生成 Markdown 文件，可以执行：

```bash
cargo run -- all-md
```

默认写入 `demo_output.md`，也可以手动指定输出路径：

```bash
cargo run -- all-md report.md
```
