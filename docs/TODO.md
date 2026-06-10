# TODO：Simple LR 实验收尾清单

## 当前已完成

* [x] 左递归 Simple 文法与拓广开始符号
* [x] 词法分析器与注释/错误处理
* [x] `FIRST` / `FOLLOW`
* [x] LR(0) 项目、`closure`、`goto`、DFA
* [x] LR(0) ACTION / GOTO 表与冲突输出
* [x] SLR(1) ACTION / GOTO 表
* [x] LR(1) 项目、`closure`、`goto`、DFA
* [x] LR(1) ACTION / GOTO 表
* [x] 基于表驱动的 SLR(1) / LR(1) 主循环
* [x] 语法树节点池与缩进输出
* [x] `cargo run -- all` 全流程演示
* [x] 单元测试覆盖 lexer / parser / lr_table / lr1 dfa

## 当前命令

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

* `all` 使用内置样例演示完整流程，不再依赖 `examples/`
* `parse-slr` / `parse-lr1` 保留文件输入模式，便于手工测试自定义源码

## 后续可选工作

### 输出与报告

* [ ] 视实验报告需要，补一个“精简截图模式”命令
* [x] 补一份简短 README

### 可选整理

* [ ] 视需要增加矩阵格式分析表输出，便于报告截图
* [ ] 如果继续演进代码结构，可进一步评估是否抽象更多共享逻辑
