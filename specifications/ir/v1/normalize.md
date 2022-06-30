# Dejavu IR v1 规范化

在 conformance runner 做语义相等比较 **之前**应用以下规则。

## 规则

1. **剥除 span**  
   递归删除每一个 `span` 属性。

2. **稳定对象键序**  
   序列化比较时，每一层对象键按字典序排序。（排序后 `type` 不必仍在首位；相等是结构相等。）

3. **If / ElseIf 形状**
    - `Stmt.If.elseIfs` 必须是数组（空时用 `[]`）。
    - 永不把 `Stmt.ElseIf` 放进 `consequent` 或 `alternate`。
    - 无 `else` 分支时省略 `alternate`（不要发 `null` 或 `[]`，除非分支存在且 body 为空——优先省略）。

4. **删除空 Text**  
   丢弃 `value` 恰好为 `""` 的 `Text` 节点。  
   **不要**丢弃仅含空白的文本（空格/换行有意义）。

5. **保留 Comment**  
   IR 中保留 `Comment` 节点（渲染器忽略它们）。规范化时不要删除。

6. **数字**  
   字面量只用 JSON number。不要把数字编成字符串。v1 无独立的整数/浮点标签。

7. **默认标志**
    - `Interpolation.raw` 默认为 `false`；规范化后为 `false` 时省略。
    - `Stmt.For.index` 缺失时省略。

8. **Language 块**  
   始终存在。T1 金样使用默认定界符集，除非该用例测试自定义定界符。

9. **管道参数**  
   `Expr.Pipe.arguments` 始终为数组（过滤器无参时为 `[]`）。

## 伪代码

```text
function normalize(node):
  if object:
    delete node.span
    if node.type == "Text" and node.value == "": return null
    if node.type == "Interpolation" and node.raw == false: delete node.raw
    if node.type == "Stmt.For" and node.index is missing: ok
    for each key, value in node:
      node[key] = normalize(value)  # arrays filter nulls
    return node
  if array:
    return [normalize(x) for x in array if normalize(x) != null]
  return node
```

## 相等

规范化后，对 JSON 值做深度相等（dump 时键已排序）。不比较源码格式或 span 是否存在。
