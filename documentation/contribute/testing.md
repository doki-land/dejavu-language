# 测试与验收

## 适用场景

提交前、发布前或调查宿主漂移时，用本页选择验证范围。

## 仓库根命令

```bash
pnpm fmt:check
pnpm test
pnpm conformance
```

`pnpm test` 运行仓库脚本定义的 Rust 和 TypeScript 基线。其他宿主必须运行各自测试，不能由这个命令自动代表。

## 定向命令

```bash
pnpm --dir projects/dejavu.ts/dejavu-engine typecheck
pnpm --dir projects/dejavu.ts/dejavu-engine test
cargo test --manifest-path projects/dejavu.rs/Cargo.toml
```

路径形式不依赖 workspace 当前包名；包名和发布作用域仍由发布清单单独检查。

## 解释 conformance 输出

- PASS：宿主进程成功并与 `expected.out.txt` 字节一致。
- FAIL：宿主运行成功但输出不同，或进程明确失败。
- SKIP：没有获得该宿主的验证结果，不能算通过。

当前 runner 可能在一个案例被所有宿主跳过后仍返回 0。发布说明和兼容性文档必须按逐案例输出判断，而不是只看进程退出码。

## 完成验收

- [ ] 格式检查通过。
- [ ] 受影响模块的单元测试通过。
- [ ] 新行为有共享 fixture。
- [ ] 每个目标宿主的 PASS / FAIL / SKIP 已记录。
- [ ] 用户示例与实际 API、语法和输出一致。
