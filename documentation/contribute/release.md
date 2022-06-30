# 发布变更

## 适用场景

发布公共 facade、实现包、宿主绑定或语言契约版本时使用本清单。

## 发布前检查

- 公共 npm 包使用 `@doki-land/*` 作用域；应用入口是 `@doki-land/dejavu`。
- package exports 指向实际构建产物，不把 workspace 源文件当成发布产物。
- 内部 engine/language/types 包没有被用户教程误写成应用依赖。
- Contract、IR schema、fixture 和实现版本关系已记录。

## 验收命令

从仓库根目录执行：

```bash
pnpm fmt:check
pnpm test
pnpm conformance
```

随后运行每个发布宿主的包构建和测试，并人工核对 conformance 的 SKIP。

## 文档检查

- 英文和中文用户文档文件树、标题层级和代码块一致。
- 安装命令、导入路径和 API 与发布包一致。
- 兼容性页写明日期、命令和 fixture 范围。
- 未完成能力没有进入用户导航或发布说明。

## 完成验收

- [ ] 包名、版本、exports 和产物可从干净项目安装。
- [ ] 用户 quickstart 在干净项目运行并产生文档中的输出。
- [ ] 所有目标宿主状态有证据。
- [ ] 发行说明明确兼容性变化和迁移要求。
