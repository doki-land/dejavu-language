# 架构总览

## 数据流

```text
应用或框架适配器
        ↓
公共 facade：@doki-land/dejavu  /  dejavu::*（Rust） / 各语言同名入口
        ↓
parser：source → IR
        ↓
renderer：IR + Context → output
        ↓
宿主输出（HTML、文本或框架响应）
```

宿主可以用自己的语言实现 parser、renderer 和绑定 API，但可观察语言行为由共享 Contract、IR schema 和 conformance fixture 定义。

TypeScript 应用入口固定为 `@doki-land/dejavu`；Rust 为 `dejavu` crate（`use dejavu::*`）。`@doki-land/dejavu-engine` 仅服务 Doki 产品宿主。

## 责任边界

| 层              | 负责                                               | 不负责                            |
|-----------------|----------------------------------------------------|-----------------------------------|
| 公共 facade     | 稳定的应用入口、parse、render、renderSource、check | 暴露实现层目录                    |
| language/parser | 定界符、词法、表达式和模板结构，产出 IR            | 读应用数据库、调用框架 UI         |
| IR/types        | 稳定节点形状、值和规范化                           | 绑定某一个宿主的对象模型          |
| engine/renderer | 表达式求值、转义、过滤器、继承和 include           | 定义另一套模板语义                |
| loader          | 逻辑路径、root、优先级、canonical id 和诊断        | 负责鉴权、HTTP 路由或业务文件发现 |
| 宿主适配器      | 请求/模型到 Context 的转换、宿主响应类型           | 修改 IR 语义或增加私有语法        |

## 关键不变量

- 相同 IR 和相同 Context 必须产生相同的规范输出。
- 正式执行路径使用 IR，不使用 legacy AST 快捷路径。
- loader 的依赖、循环检测和诊断使用 canonical id，而不是原始引用字符串。
- HTML 插值默认转义；绕过转义必须由显式 `safe` / `raw` 或宿主安全值完成。

实现细节见[同构分层](./isomorphic-layers.md)、[宿主表面与 LSP](./host-surface-and-lsp.md)
，规范行为见 [Template Contract](../../specifications/template-contract/v1.md)。
