# Dejavu 核心开发者文档

本目录只服务贡献者、parser/IR/renderer 维护者和宿主绑定维护者。应用开发者请从 `/d/en-us/` 或 `/d/zh-hans/`
的用户文档开始；本目录不提供业务模板教程。

## 先选任务

- [第一次进入仓库](./contribute/onboard.md)：安装、检查、测试和代码地图。
- [修改模板语法](./contribute/change-parser.md)：parser、契约、fixture 和回归。
- [修改 IR](./contribute/change-ir.md)：schema、normalize、类型和跨宿主 fixture。
- [修改渲染器或过滤器](./contribute/change-render-loader.md)：render、escaping、undefined 和 loader。
- [增加宿主绑定](./contribute/add-host.md)：公共 facade、IR 消费和 conformance。
- [增加 conformance](./contribute/add-conformance.md)：夹具格式和验收命令。
- [发布变更](./contribute/release.md)：包名、导出面、兼容性和文档。

## 规范与实现

- [架构总览](./architecture/overview.md)
- [同构分层](./architecture/isomorphic-layers.md)
- [Template Contract](../specifications/template-contract/v1.md)
- [IR v1](../specifications/ir/v1/readme.md)
- [语法实现状态](./grammar/index.md)
- [宿主实现状态](./compatibility.md)

## 工作规则

1. 先修改或确认规范，再修改实现。
2. 可观察行为必须有 conformance 或单元测试。
3. 公共应用 API 使用 `@doki-land/dejavu`；实现包只供绑定维护者使用。
4. 规范描述宿主无关行为；宿主差异写入实现状态，不写进语言语义。

## 文档写作

请遵守[文档写作规范](./documentation-style.md)：每页只解决一个任务，能力声明必须有证据，不把规划写成交付能力。
