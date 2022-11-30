# 增加 Conformance Fixture

## 适用场景

锁定用户可观察的 parse、render、inheritance 或 loader 行为时使用 fixture。单元测试只补充实现边角，不能替代共享契约样例。

## 修改前阅读

- [Template Contract](../../specifications/template-contract/v1.md)
- `specifications/conformance/` 中与目标行为最接近的现有案例

## Fixture 位置

| 行为 | 目录 |
| --- | --- |
| 插值、条件、循环、过滤器、转义 | `specifications/conformance/t1/` |
| extends、block、super、include | `specifications/conformance/inheritance/` |
| root、scheme、relative、priority、错误 | `specifications/conformance/loader/` |

典型 fixture 包含源码或模板树、`context.ctx.json`、`expected.ir.json`、`expected.out.txt`。Loader 案例使用 `roots.json`、`entry.txt` 以及期望 id 或期望诊断。

## 工作目录与命令

从仓库根目录执行：

```bash
pnpm fmt:check
pnpm --dir projects/dejavu.ts/dejavu-engine test
pnpm conformance
```

## 编写顺序

1. 先写规范句，明确合法输入、输出和错误。
2. 添加会失败的 fixture。
3. 修改实现。
4. 在每个受影响宿主运行 fixture。
5. 在 PR 中列出未运行或被跳过的宿主。

## 完成验收

- [ ] Fixture 最小且只验证一个行为。
- [ ] 输出精确到字节，空白差异有意为之。
- [ ] 非法输入验证诊断，不只验证“抛错”。
- [ ] 没有把 SKIP 记成 PASS。
