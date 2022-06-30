# 宿主绑定实现笔记

本目录给实现或维护宿主绑定的人看。应用接入教程位于 `/d/*`
用户文档，绑定设计原则见[增加宿主绑定](../contribute/add-host.md)。

- [TypeScript 公开表面](./typescript-usage.md)
- [Rust 公开表面](./rust-usage.md)
- [宿主实现状态](../compatibility.md)

所有绑定必须消费共享 IR 语义。框架适配器可以转换请求、Context 和响应类型，但不能定义私有模板语法。
