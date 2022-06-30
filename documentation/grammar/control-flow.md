# 控制流实现状态

当前契约只定义：

- `if`、`else if`、`else`、`end if`
- `loop item in values`、`end loop`

Parser 必须把这些结构 lower 到 IR 的条件和循环节点；renderer 必须保持分支顺序和集合迭代顺序。

范围循环、元组解构、`break`、`continue`、`while` 和 `match` 未纳入当前 Contract。不要在 parser、IDE 语法或用户文档中把它们标为稳定能力。

证据：`specifications/conformance/t1/if_else` 和 `loop_pipe`。
