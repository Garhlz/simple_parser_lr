# SPEC：Simple LR 实验当前实现说明

## 目标

本项目实现了一个面向 `Simple` 左递归文法的 LR 语法分析器，当前已覆盖：

* 词法分析
* `FIRST` / `FOLLOW`
* LR(0) 项目、闭包、`goto`、DFA
* LR(0) / SLR(1) ACTION / GOTO 表
* LR(1) 项目、闭包、`goto`、DFA
* LR(1) ACTION / GOTO 表
* 基于 SLR(1) / LR(1) 表的语法分析主循环
* 语法树构造与缩进输出

## 当前模块

```text
src/
  main.rs
  dfa.rs
  symbol.rs
  grammar.rs
  lexer.rs
  first_follow.rs
  item_set.rs
  item_lr0.rs
  item_lr1.rs
  dfa_lr0.rs
  dfa_lr1.rs
  lr_table.rs
  lr_parser.rs
  syntax_tree.rs
```

说明：

* `grammar.rs`：Simple 左递归文法与拓广开始符号
* `lexer.rs`：token 化、注释跳过、词法错误处理
* `first_follow.rs`：`FIRST` / `FOLLOW` 与 `first_of_sequence`
* `item_set.rs`：基于 `BTreeSet` 的通用项目集壳子
* `item_lr0.rs`：LR(0) 项目与项目集
* `item_lr1.rs`：LR(1) 项目与项目集
* `dfa.rs`：通用 DFA 外壳
* `dfa_lr0.rs` / `dfa_lr1.rs`：对应项目集的 `closure`、`goto` 与 DFA
* `lr_table.rs`：LR(0) / SLR(1) / LR(1) 表构造与冲突记录
* `lr_parser.rs`：表驱动分析主循环、错误处理、步骤记录
* `syntax_tree.rs`：节点池语法树

## 命令入口

```bash
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
```

说明：

* `all` 使用内置样例，输出完整文法、DFA、分析表和分析结果
* `parse-slr` / `parse-lr1` 保留文件输入，便于单独测试自定义源码
* 默认 `cargo run --` 等价于 `cargo run -- all`

## 输出约定

* `lr0-dfa`：打印全部项目集状态与转移边
* `lr1-dfa`：打印 Canonical LR(1) 项目集状态与转移边
* `lr0-table` / `slr-table` / `lr1-table`：打印完整 ACTION / GOTO 表与冲突统计
* `parse-slr` / `parse-lr1`：打印分析步骤表和语法树
* 语法错误与内部错误会用不同前缀区分

## 测试范围

当前测试覆盖：

* lexer 成功路径与错误路径
* parser 成功路径、错误路径、步骤输出与语法树输出
* LR(0) / SLR(1) / LR(1) 构表
* LR(1) closure / DFA
* LR 表冲突数量与表文本格式
