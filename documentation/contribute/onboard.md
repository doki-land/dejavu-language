# 第一次进入仓库

## 适用场景

你刚克隆仓库，需要安装依赖、跑通基线，并定位 parser、IR、renderer 和 loader。

## 修改前阅读

- [架构总览](../architecture/overview.md)
- [Template Contract](../../specifications/template-contract/v1.md)
- [IR v1](../../specifications/ir/v1/readme.md)

## 工作目录与命令

以下命令都从 `dejavu-engine` 仓库根目录执行：

```bash
pnpm install
pnpm fmt:check
pnpm test
pnpm conformance
```

`pnpm conformance` 当前允许宿主跳过用例，不能单独作为完整跨宿主绿灯。检查输出中的每个宿主和用例，尤其是 `safe_raw`。

## 代码地图

| 任务                         | 位置                                                  |
|------------------------------|-------------------------------------------------------|
| TypeScript parser            | `projects/dejavu.ts/dejavu-language`                  |
| TypeScript renderer / loader | `projects/dejavu.ts/dejavu-engine`                    |
| 公共 TypeScript facade       | `projects/dejavu.ts/dejavu`                           |
| Rust parser / renderer       | `projects/dejavu.rs/dejavu-language`、`dejavu-engine` |
| IR schema 和金样             | `specifications/ir`、`specifications/conformance`     |

## Windows 已知问题

部分跨语言脚本在仓库绝对路径包含空格时可能把路径拆开，出现类似“不是内部或外部命令”的错误。先查看失败命令和
stderr；不要把宿主启动失败误判为模板语义失败。

## 完成验收

- [ ] 依赖安装成功。
- [ ] 格式检查和基础测试通过。
- [ ] 能指出 source → IR、IR → output 和 loader resolve 的入口。
- [ ] 能区分规范要求、fixture 证据和当前宿主状态。
