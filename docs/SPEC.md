# SPEC：Simple LR 实验当前实现说明

## 目标

本项目实现了一个面向 Simple 左递归文法的 LR 语法分析器，当前已覆盖：

* 词法分析
* `FIRST` / `FOLLOW`
* LR(0) 项目、闭包、`goto`、DFA
* LR(0) / SLR(1) ACTION / GOTO 表
* 基于 SLR(1) 表的语法分析主循环
* 语法树构造与缩进输出

当前未实现：

* LR(1) 项目、DFA、分析表
* 基于 LR(1) 的分析入口

## 当前模块

```text
src/
  main.rs
  symbol.rs
  grammar.rs
  lexer.rs
  first_follow.rs
  item_lr0.rs
  dfa_lr0.rs
  lr_table.rs
  lr_parser.rs
  syntax_tree.rs
```

说明：

* `grammar.rs`：Simple 左递归文法与拓广开始符号
* `lexer.rs`：token 化、注释跳过、词法错误处理
* `first_follow.rs`：`FIRST` / `FOLLOW` 与 `first_of_sequence`
* `item_lr0.rs`：LR(0) 项目与项目集
* `dfa_lr0.rs`：`closure`、`goto`、LR(0) DFA
* `lr_table.rs`：LR(0) / SLR(1) 表构造与冲突记录
* `lr_parser.rs`：表驱动分析主循环、错误处理、步骤记录
* `syntax_tree.rs`：节点池语法树

## 命令入口

```bash
cargo run --
cargo run -- all
cargo run -- grammar
cargo run -- first-follow
cargo run -- lr0-dfa
cargo run -- lr0-table
cargo run -- slr-table
cargo run -- parse-slr <source-file>
cargo run -- tokens <source-file>
```

说明：

* `all` 使用内置样例，输出完整文法、DFA、分析表和分析结果
* `parse-slr` 保留文件输入，便于单独测试自定义源码

## 输出约定

* `lr0-dfa`：打印全部项目集状态与转移边
* `lr0-table` / `slr-table`：打印完整 ACTION / GOTO 表与冲突统计
* `parse-slr`：打印分析步骤表和语法树
* 语法错误与内部错误会用不同前缀区分

## 测试范围

当前测试覆盖：

* lexer 成功路径与错误路径
* parser 成功路径、错误路径、输出格式
* LR 表冲突数量与表文本格式
