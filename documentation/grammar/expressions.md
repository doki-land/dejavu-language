# 表达式实现状态

当前契约包含字面量、标识符、成员/索引访问、二元/一元表达式和 `|>`
过滤器。具体节点形状见 [IR 节点](../../specifications/ir/v1/nodes.md)。

实现检查重点：

- parser 优先级在宿主间一致；
- 缺失标识符遵循默认空输出和 strict 模式；
- 过滤器从左到右求值；
- 单独的 `|` 被拒绝；
- JSON 兼容值在宿主间不引入私有类型语义。

函数调用和任意宿主函数执行未纳入当前 Contract。过滤器参数只按 Contract 定义的形式解析。

证据：`specifications/conformance/t1/{member,if_else,loop_pipe,safe_raw}`。
