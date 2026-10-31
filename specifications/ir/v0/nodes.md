# Dejavu IR v0 节点

机器权威 schema：[`schema.json`](./schema.json)。

## Document

| 字段        | 类型       | 说明                     |
|-------------|------------|--------------------------|
| `irVersion` | `"v0"`     | 必须与 schema const 一致，未知版本显式拒绝 |
| `language`  | `Language` | 回显解析时配置           |
| `body`      | `Template` | 根节点                   |

默认定界符：代码 `<%` `%>`，注释 `<#` `#>`。

## Trim 模式

编码在引入代码的开标签上：

| `trim`  | 源码开标签 | 效果                          |
|---------|------------|-------------------------------|
| `none`  | `<%.`      | 保留周围空白                  |
| `ws`    | `<%_`      | 去掉空格/制表，保留换行       |
| `nl`    | `<%-`      | 去掉空格与最近换行            |
| `ws_nl` | `<%~`      | 去掉空格/换行，但保留最远换行 |
| `all`   | `<%=`      | 去掉所有相邻空白              |

无修饰符的纯 `<%` 在 T1 下序列化为 `trim: "none"`，除非某语言文档另有默认。

## 结构节点

### `Template`

根容器。`children` 为有序内容节点列表。

### `Text`

字面模板文本。`value` 为定界符切分后的精确源码文本（未做 HTML 转义）。

### `Comment`

不含定界符的模板注释体。渲染时忽略。

### `Interpolation`

将 `expression` 渲染为字符串。可选 `raw: true` 跳过 HTML 自动转义。

## 语句（`Stmt.*`）

### `Stmt.If`

- `test`、`consequent[]`
- `elseIfs[]`，元素为 `Stmt.ElseIf`（永不混入 `consequent`）
- 可选 `alternate[]` 表示 `else`

### `Stmt.For`

`loop item in iterable`（当 `language.template.legacyFor` 时也可用遗留 `for`）。

- `item`：循环变量名
- `iterable`：表达式（渲染时必须是数组，否则 `dejavu.template.invalid-iterable`）
- `body[]`

节点形状**不含**索引字段。循环元数据以隐式绑定 `forloop` 出现在每次迭代的作用域中：

| 字段     | 类型    | 含义                                   |
|----------|---------|----------------------------------------|
| `index`  | number  | 1 基序号                               |
| `index0` | number  | 0 基序号                               |
| `first`  | boolean | 是否首项                               |
| `last`   | boolean | 是否末项                               |
| `length` | number  | 总项数                                 |
| `depth`  | number  | 1 基嵌套层                             |

- `forloop` 只在 body 内可见；循环外不绑定（缺失时按普通标识符规则：非 strict 为 `null`，strict 报 `dejavu.template.undefined-variable`）。
- 嵌套循环：内层迭代的 `forloop` 遮蔽外层，body 结束后恢复外层当前迭代值。
- `depth` 入口规则：循环入口时，若当前可见 `forloop` 是对象且其 `depth` 为不小于 1 的整数，则新 `depth = depth + 1`，否则 `depth = 1`。宏调用、partial 与 `only` include 交换整个作用域，`depth` 从 1 重启。
- 同名上下文变量（含上下文自带的 `forloop`，以及字面名为 `forloop` 的循环变量）在 body 内被遮蔽，循环后按原值恢复。恢复由**键是否存在**决定：键存在时写回原值（包括 JSON `null`），仅键缺失时删除。上下文 `forloop` 非对象或 `depth` 非法时 `depth` 从 1 重启，原值仍按同一规则恢复。
- 空数组零次迭代：body 不执行，`forloop` 不绑定。

### `Stmt.Break` / `Stmt.Continue`

`break` / `continue` 语句节点，无附加字段（仅 `trim` 与可选 `span`）。

- 绑定对象是**词法上最近的外层 `Stmt.For`**，且只在同一**词法单元**内成立。
  词法单元 = 模板顶层 body、每个 `macro` body、每个 `call` caller 块 body、
  每个 `partial` body、每个 `block` body。
- `break` 终止所绑定的循环：跳过剩余迭代，循环正常收尾（循环变量与 `forloop`
  按普通规则恢复），循环之后的模板内容继续渲染。
- `continue` 跳过所绑定循环当前迭代的剩余 body，进入下一次迭代；最后一次迭代
  上的 `continue` 等价于循环正常结束。
- 嵌套循环中信号只被最内层绑定的循环消费一次，不影响外层。
- `break` 不会把被中止的迭代改写成 `forloop.last == true`；元数据仍按原始
  `length` 计算。
- 词法单元内没有外层 `loop` 时（含跨 macro / caller / partial / block 边界逃逸），
  parser 报 `dejavu.parse.control-outside-loop`（args `kind` 为 `break` 或
  `continue`）。手写 IR 必须满足同一绑定规则；渲染器与 AOT 编译器不依赖 parser
  时按各自合同拒绝或产生无效产物。
- AOT 与 JIT 控制流语义一致：AOT 直接生成宿主语言的循环内 `break` / `continue`。

### `Stmt.Block` / `Stmt.Extends` / `Stmt.Include` / `Stmt.Super` / `Stmt.Raw`

T1 的继承与包含表面。`Stmt.Extends.parent` 与 `Stmt.Include.path` 为表达式（通常是字符串字面量）。

## 表达式（`Expr.*`）

| 类型              | 作用                             |
|-------------------|----------------------------------|
| `Expr.Literal`    | `null` / bool / number / string  |
| `Expr.Identifier` | 在上下文中按名查找               |
| `Expr.Member`     | `object.property`                |
| `Expr.Index`      | `object[index]`                  |
| `Expr.Call`       | `callee(args...)`                |
| `Expr.Binary`     | 算术 / 比较 / 逻辑 / `in`        |
| `Expr.Unary`      | `!` `-` `+`                      |
| `Expr.Pipe`       | `expression \|> filter(args...)` |

过滤器 **不是**独立语句节点；一律以 `Expr.Pipe` 出现。

## Span

`span: { start, end }` 为相对原始源码的 UTF-8 字节偏移。手写金样可省略 span，且 **在语义相等前必须剥除**（见 [
`normalize.md`](./normalize.md)）。

## Value / Context JSON

运行时值使用普通 JSON：

- `null`、`boolean`、`number`、`string`
- 值数组
- 字符串键 → 值 的对象

Conformance 的 `*.ctx.json` 是单个 JSON 对象（根上下文），不加外层包装。
