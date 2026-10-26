# 修改 AOT 代码生成与 Source Map

## 适用场景

修改 `dejavu generate` 输出、Rust `dejavu-codegen`、生成模块形状、`LineOrigin` / source map，或 `strictUndefined` 守卫插入时使用本流程。

## 修改前阅读

- [TypeScript 公开表面](../guide/typescript-usage.md)（`@doki-land/dejavu` 与 `@doki-land/dejavu-engine` 分工）
- [Template Contract](../../specifications/template-contract/v0.md) 中的 `strictUndefined` 与转义规则

## 仓库布局

| 层 | 路径 |
|----|------|
| Rust codegen | `projects/dejavu.rs/crates/dejavu-codegen/` |
| N-API 桥接 | `projects/dejavu.rs/crates/dejavu-napi/` |
| `dejavu generate` 编排与 source map JSON | `projects/dejavu.ts/packages/dejavu/src/generate.ts` |
| 生成模块运行时 | `projects/dejavu.ts/packages/dejavu-engine/`（`runtime` 子路径） |

## 工作目录与命令

从仓库根目录执行：

```bash
cargo test -p dejavu-codegen
pnpm --filter @doki-land/dejavu test
pnpm conformance
```

`generate.test.mjs` 中的 source map 用例会调用完整 `dejavu generate` 路径，修改 origins 或 `encodeMappings` 后必须跑通。

## Line origins 与 source map

Codegen 为每个模板构造记录一条 `LineOrigin`：`line` 是生成模块体内的 0-based 行号，`start` / `end` 是模板字节 span。`generate.ts` 在写出 `.js.map` 时为每个 origin 行在列 0 放一个 VLQ 段。

插值会生成三行主体代码：

```js
let __v = ctx.title;
if (__strict && __v === undefined) throw new Error("...");
__out.push(htmlEscape(valueToString(__v)));
```

**origin 落在 `let __v = ...` 这一行**，不是 `strictUndefined` 守卫行，也不是 `__out.push` 行。守卫是插值求值的实现细节，不单独映射回模板。

控制流构造（`if`、`for` 等）的 origin 仍落在对应语句的第一行；插值后的守卫会使后续 `if` 在生成代码中整体下移一行，Rust 测试 `origins_point_each_construct_at_its_template_span` 已锁定该偏移。

## Rust 测试中的 conformance 路径

crate 位于 `projects/dejavu.rs/crates/*` 时，金样目录为：

```text
../../../../specifications/conformance/t1
```

（四层 `..` 回到仓库根，再进入 `specifications/`。）

## 必须增加的测试

- Rust：`dejavu-codegen/tests/single_template.rs` 中的 origins 与 emit 形状。
- TypeScript：`projects/dejavu.ts/packages/dejavu/tests/generate.test.mjs` 中的 source map 与 generate 事务测试。
- 若改变可观察渲染语义，同步更新 `specifications/conformance/` 与 `pnpm conformance`。

## 完成验收

- [ ] `cargo test -p dejavu-codegen` 与 `pnpm --filter @doki-land/dejavu test` 通过。
- [ ] source map 测试按 binding 行（`let __v`）断言，不假设其在 `htmlEscape` push 的上一行。
- [ ] 生成模块仍只 import `@doki-land/dejavu-engine/runtime`。
- [ ] 未手改 `dist/` 或用户 `src/generated/` 来“修”映射。
