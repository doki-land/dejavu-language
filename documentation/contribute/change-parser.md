# 修改模板语法

## 适用场景

修改定界符、token、表达式优先级、控制流或 source → IR 行为时使用本流程。

## 修改前阅读

- [Template Contract](../../specifications/template-contract/v1.md)
- [IR 节点](../../specifications/ir/v1/nodes.md)
- [语法实现状态](../grammar/index.md)

当前 Contract 已冻结的行为不能静默改变。新增能力先定义语法、IR、诊断和版本策略；未进入 Contract 的能力不能只在单一宿主先变成正式语法。

## 工作目录与命令

从仓库根目录执行：

```bash
pnpm --dir projects/dejavu.ts/dejavu-language typecheck
pnpm --dir projects/dejavu.ts/dejavu-engine test
pnpm conformance
```

路径形式不依赖 workspace 当前包名；发布面仍统一使用 `@doki-land/*`。

## 修改位置

- TypeScript parser：`projects/dejavu.ts/dejavu-language/src/`
- Rust parser：`projects/dejavu.rs/dejavu-language/src/`
- IR 类型：对应宿主的 IR/types 模块
- 规范：`specifications/template-contract/`、`specifications/ir/`

## 必须增加的测试

- 一个合法输入 fixture：源码、期望 IR、上下文和期望输出。
- 每种非法形式的诊断测试：文件、span、原因和修复建议。
- 表达式变化的优先级和组合测试。
- 受影响宿主的 parser parity 测试。

## 完成验收

- [ ] Contract 和 IR schema 先于或随实现更新。
- [ ] 所有可观察行为都有 fixture。
- [ ] 未扩散 legacy AST。
- [ ] 用户文档只增加已经通过验收的语法。
