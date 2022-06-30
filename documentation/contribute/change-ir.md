# 修改 Dejavu IR

## 适用场景

新增或修改 IR 节点、字段、规范化规则、序列化形状或版本兼容行为时使用本流程。

## 修改前阅读

- [IR v1](../../specifications/ir/v1/readme.md)
- [节点定义](../../specifications/ir/v1/nodes.md)
- [规范化规则](../../specifications/ir/v1/normalize.md)

## 工作目录与命令

从仓库根目录执行：

```bash
pnpm fmt:check
pnpm conformance
cargo test --manifest-path projects/dejavu.rs/Cargo.toml -p dejavu-ir
```

随后运行每个受影响宿主自己的 IR 测试；不能仅验证 TypeScript 类型通过。

## 修改位置

- JSON Schema：`specifications/ir/v1/schema.json`
- 节点说明和规范化：`specifications/ir/v1/`
- 金样：`specifications/conformance/*/expected.ir.json`
- TypeScript 类型与 normalize：`projects/dejavu.ts/dejavu-types/src/`
- 各宿主 IR decoder / renderer：`projects/dejavu.{rs,cs,kt,py}/`

## 必须增加的测试

- schema 接受新形状、拒绝非法形状。
- normalize 前后语义等价。
- 旧 fixture 的兼容结果明确。
- 每个目标宿主从同一 JSON IR 渲染相同输出。

## 完成验收

- [ ] 版本和迁移策略已写入 IR 规范。
- [ ] Opaque/internal 字段没有泄漏进 wire format。
- [ ] 所有宿主都明确为通过、部分通过或未验证。
- [ ] 兼容性文档与真实证据同步。
