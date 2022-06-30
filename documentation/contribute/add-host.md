# 增加宿主绑定

## 适用场景

为新的语言或运行时实现 Dejavu parser、IR renderer 或公共 facade 时使用本流程。

## 修改前阅读

- [同构分层](../architecture/isomorphic-layers.md)
- [Template Contract](../../specifications/template-contract/v1.md)
- [IR v1](../../specifications/ir/v1/readme.md)

## 实现要求

- 提供 parse、render、render source 和诊断的宿主原生入口。
- JSON Context 与 IR wire format 不产生宿主私有语义。
- 框架适配器建立在绑定之上，不在适配器里复制 parser 或 renderer。
- 公共包与内部模块分离，应用只接触稳定 facade。

## 必须增加的测试

- T1 IR render fixture。
- source → IR parser parity。
- safe/raw、undefined 和错误诊断。
- inheritance 与 loader fixture；未实现时在兼容性页面明确标记。

## 验收命令

从仓库根目录运行 `pnpm conformance <host-id>`，再运行宿主自己的完整测试套件。必须检查逐案例结果，不能把跳过当作通过。

## 完成验收

- [ ] 公共 API 有最小集成示例和预期输出。
- [ ] 至少完成 T1 IR render conformance。
- [ ] Parser、loader 和继承状态分别记录。
- [ ] 未引入宿主私有模板语法。
