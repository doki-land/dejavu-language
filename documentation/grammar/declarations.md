# 推迟的声明能力

当前 Template Contract 不定义局部变量声明、宏、模板函数、namespace、using、class、trait 或模块系统。

历史实现中可能仍存在相关 token、AST 类型或占位代码。这些内容不构成正式语言表面，也不能出现在用户教程、IDE completion 或兼容性声明中。

将来引入声明能力前，必须先确定：

- 完整语法和合法位置；
- IR 节点及 wire version；
- 作用域、求值和错误语义；
- 与所有宿主的迁移策略；
- 合法、非法和跨宿主 conformance fixture。
