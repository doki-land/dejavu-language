# 同构分层

Dejavu 的语言契约与宿主实现分离：每个宿主可以使用自己的 parser 和运行时，但最终消费同一份 IR 语义。

## 共享内容

- IR v1 的 JSON 形状和规范化规则。
- 表达式、控制流、过滤器、转义、继承和 loader 的可观察行为。
- 诊断的阶段、代码和可修复信息。
- conformance fixture 的输入、上下文、期望 IR 和期望输出。

## 可以不同的内容

- lexer、parser 和错误呈现方式。
- 包管理、绑定 API 的命名和宿主原生值类型。
- 解释执行、AOT 或其他内部优化，只要输出不改变。
- 框架适配器的请求、响应和文件系统集成。

## 公共 facade

应用依赖 `@doki-land/dejavu`。宿主绑定可以依赖 `@doki-land/dejavu-engine` 等实现包，但不能把这些内部模块当成应用 API。

```text
source.dejavu
   ↓
host parser → Dejavu IR
   ↓
host renderer(IR, Context)
   ↓
host output
```

## 一致性声明

规范要求所有符合契约的宿主在相同 IR 和 Context 下产生相同输出；当前哪些宿主已经通过哪些
fixture，见[宿主实现状态](../compatibility.md)。实现状态不能反向修改语言规范。
