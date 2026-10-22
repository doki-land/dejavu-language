# dejavu-napi

Dejavu 的 N-API 引擎面：把 Rust 引擎的正交能力导出给 Node 宿主。

## 合同

- 绑定合同标识为 `dejavu.napi/1`，由 `bindingContractVersion()` 返回，宿主先校验再调用。
- 导出能力仅包括：`parseIr`、`normalizeIr`、`renderIr`、`renderSource`、`resolveTemplateRef`。
- 命令语义（`build` / `dev` / `check` 的参数、配置、stdio、退出码、交互）属于 TypeScript CLI，
  不在本 crate 导出，也不包装 clap。
- 诊断只有一种形态：`code` + `severity` + `stage` + `message` + `span` + `path` + `argsJson` + `related`。
- `context`、`catalog`、`options` 一律以 JSON 字符串传入，避免对象与 `serde_json::Value` 的类型分歧。

## catalog 规格

`renderIr` / `renderSource` 的 `catalog` 是 JSON 字符串：

```json
{
  "extensions": ["", ".dejavu"],
  "roots": [
    {
      "name": "pages",
      "scheme": "pages",
      "priority": 0,
      "basePath": "templates",
      "files": { "index.dejavu": { "irVersion": "1.0", "language": {}, "body": {} } }
    }
  ]
}
```

`files` 的值是已解析的 IR JSON，解析与外层 IR 一样由宿主通过 `parseIr` 得到。
