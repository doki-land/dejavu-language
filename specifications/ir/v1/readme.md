# Dejavu IR v1

Dejavu 模板的跨语言中间表示。

| 文件                             | 作用                         |
|----------------------------------|------------------------------|
| [`schema.json`](./schema.json)   | JSON Schema（draft 2020-12） |
| [`nodes.md`](./nodes.md)         | 可读节点目录                 |
| [`normalize.md`](./normalize.md) | 语义相等前的规范化           |
| [`examples/`](./examples)        | 手写 IR 样例                 |

宿主遗留 AST dump **不是**本 IR；仅符合 `schema.json` 的文档算数。
