# 控制流实现状态

当前契约定义：

- `if`、`else if`、`else`、`end if`
- `loop item in values`、`end loop`
- 循环元数据：每次迭代在 body 作用域隐式绑定 `forloop` 对象，字段恰为
  `index`（1 基）、`index0`（0 基）、`first`、`last`、`length`、`depth`（1 基嵌套层）。
  嵌套循环内层遮蔽外层并在 body 结束后恢复；宏调用、partial 与 `only` include
  交换作用域，`depth` 从 1 重启。完整语义见 `specifications/ir/v0/nodes.md` 的
  `Stmt.For` 与 `specifications/conformance/loop-metadata`。

Parser 必须把这些结构 lower 到 IR 的条件和循环节点；renderer 必须保持分支顺序和集合迭代顺序。

范围循环、元组解构、`while` 和 `match` 未纳入当前 Contract。不要在 parser、IDE 语法或用户文档中把它们标为稳定能力。

证据：`specifications/conformance/t1/if_else`、`specifications/conformance/t1/loop_pipe` 和 `specifications/conformance/loop-metadata`。
