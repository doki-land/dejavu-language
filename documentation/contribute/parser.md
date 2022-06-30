# 解析器笔记

Rust T1 语言前端位于 `projects/dejavu.rs/dejavu-language`：

- **Lexer**: 手写（定界符内表达式，`CodeToken` / `CodeLexer`）
- **Parser**: 手写递归下降（模板外扫描 + 表达式优先级）
- **Diagnostics**: [miette](https://docs.rs/miette)
- **产出**: `dejavu_ir::IrDocument`（经 `NativeTemplate` → `encode_native`）

从仓库根目录运行：

```bash
cargo test --manifest-path projects/dejavu.rs/Cargo.toml -p dejavu-language
```

TypeScript 解析在 `projects/dejavu.ts/dejavu-language`，产出与 Contract / IR 对齐的文档。

跨语言契约与金样见 `specifications/conformance/t1` 与 `specifications/ir/v1`。  
Oak（`oak-dejavu`）与 logos 已从仓库移除，不再作为解析实现。

参见 [改语言与解析](./change-parser.md)。
