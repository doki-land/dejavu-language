# 语法实现状态

本目录是核心开发者的实现索引，不是用户语法教程。规范行为以 [Template Contract](../../specifications/template-contract/v1.md)
和 conformance fixture 为准。

| 能力                           | 当前契约  | 实现证据                                                         |
|--------------------------------|-----------|------------------------------------------------------------------|
| 文本、注释、插值               | 已纳入    | `specifications/conformance/t1/{comment,hello,member}`           |
| `if` / `else if` / `else`      | 已纳入    | `specifications/conformance/t1/if_else`                          |
| `loop item in values`          | 已纳入    | `specifications/conformance/t1/loop_pipe`                        |
| 表达式与 `                     | >` 过滤器 | 已纳入                                                           | [表达式状态](./expressions.md)和 T1 fixtures |
| 默认 HTML 转义、`safe` / `raw` | 已纳入    | `specifications/conformance/t1/safe_raw`；跨宿主 runner 仍有缺口 |
| extends、block、super、include | 已纳入    | `specifications/conformance/inheritance`                         |
| loader resolution              | 已纳入    | `specifications/conformance/loader`                              |
| 声明、宏、函数定义、模块       | 未纳入    | [推迟能力](./declarations.md)                                    |
| 静态类型系统                   | 未纳入    | [值模型状态](./types.md)                                         |

修改语法时走[修改模板语法](../contribute/change-parser.md)流程。
