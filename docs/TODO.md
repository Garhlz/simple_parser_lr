# TODO：Simple LR 实验收尾清单

## 当前已完成

* [x] 左递归 Simple 文法与拓广开始符号
* [x] 词法分析器与注释/错误处理
* [x] `FIRST` / `FOLLOW`
* [x] LR(0) 项目、`closure`、`goto`、DFA
* [x] LR(0) ACTION / GOTO 表与冲突输出
* [x] SLR(1) ACTION / GOTO 表
* [x] 基于表驱动的 LR 主循环
* [x] 语法树节点池与缩进输出
* [x] `cargo run -- all` 全流程演示
* [x] 单元测试覆盖 lexer / parser / lr_table

## 当前命令

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

* `all` 使用内置样例演示完整流程，不再依赖 `examples/`
* `parse-slr` 仍保留文件输入模式，便于手工测试自定义源码

## 后续可选工作

### LR(1)

* [ ] 定义 `LR1Item { production_id, dot, lookahead }`
* [ ] 实现 `closure_lr1`
* [ ] 实现 `goto_lr1`
* [ ] 构造 LR(1) DFA
* [ ] 构造 LR(1) ACTION / GOTO 表
* [ ] 增加 `lr1-dfa` / `lr1-table` / `parse-lr1` 入口

### 输出与报告

* [ ] 视实验报告需要，补一个“精简截图模式”命令
* [ ] 如果要提交源码说明，可再补一份简短 README
