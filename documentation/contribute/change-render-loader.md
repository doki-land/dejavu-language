# 修改渲染器、过滤器或 Loader

## 适用场景

修改表达式求值、HTML 转义、过滤器、undefined、继承、include、路径解析、依赖回调或诊断码时使用本流程。

## 修改前阅读

- [Template Contract](../../specifications/template-contract/v1.md) 中的 Filters、Inheritance、Loader & Resolution
- [架构总览](../architecture/overview.md)

## 工作目录与命令

从仓库根目录执行：

```bash
pnpm --dir projects/dejavu.ts/dejavu-engine typecheck
pnpm --dir projects/dejavu.ts/dejavu-engine test
pnpm conformance
```

路径形式不依赖 workspace 当前包名；提交中的公开说明使用 `@doki-land/*`。

## 修改位置

- 渲染：`projects/dejavu.ts/dejavu-engine/src/render.ts`
- loader：`projects/dejavu.ts/dejavu-engine/src/loader.ts`
- 路径解析：`projects/dejavu.ts/dejavu-engine/src/resolve.ts`
- 共享 loader 类型：`projects/dejavu.ts/dejavu-types/src/`
- 其他宿主 renderer：各宿主对应 engine 模块

## 必须增加的测试

- 过滤器和转义：普通值、安全值、恶意 HTML、参数错误。
- undefined：默认空输出和 strict 模式。
- loader：bare、scheme、relative、priority、extension、root escape。
- 继承：block 覆盖、super、include、循环和缺失模板。
- 诊断：稳定 code、ref、from 和 searched roots。

## 完成验收

- [ ] 同一 IR + Context 的输出没有宿主漂移。
- [ ] canonical id 用于依赖与循环检测。
- [ ] 没有通过重复别名修补 loader 行为。
- [ ] 安全行为和用户文档同步。
