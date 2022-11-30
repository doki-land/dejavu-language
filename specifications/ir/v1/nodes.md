# Dejavu IR v1 节点

机器权威 schema：[`schema.json`](./schema.json)。

## Document

| 字段        | 类型       | 说明                     |
|-------------|------------|--------------------------|
| `irVersion` | `"1.0"`    | 必须与 schema const 一致 |
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
- 可选 `index`
- `iterable`：表达式
- `body[]`

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
