# Dejavu 用户文档

使用 Dejavu，可以把模板和数据上下文渲染成文本或 HTML。语言目前包含插值、条件、循环、过滤器、可复用布局，并默认转义 HTML。

## 从这里开始

- 大约五分钟完成[第一次模板渲染](./start/quickstart.md)。
- 构建多文件模板前，先了解[核心概念](./start/concepts.md)。
- 在应用中[接入 TypeScript 包](./integrate/typescript.md)。

## 编写模板

- [插入数据](./templates/interpolation.md)、访问对象成员并处理缺失值。
- 使用 `if` 和 `loop` [选择和重复内容](./templates/control-flow.md)。
- 使用过滤器[转换并转义输出](./templates/filters.md)。
- 使用 `extends`、`block` 和 `include` [复用布局与局部模板](./templates/layouts.md)。

## 接入与维护

- [TypeScript 接入](./integrate/typescript.md)
- [其他宿主绑定](./integrate/other-hosts.md)
- [实现兼容性](./reference/compatibility.md)
- [故障排查](./troubleshoot.md)

语言参考描述可移植的模板行为。不同宿主绑定可能只实现其中一部分；选用前请先查看兼容性页面。
