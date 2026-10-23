# dejavu-napi

N-API bridge that exports the Dejavu Rust engine's orthogonal capabilities to Node hosts.

## Contract

- Binding contract id is `dejavu.napi/1`, returned by `bindingContractVersion()`. Hosts **must** verify before calling.
- Exported surface: `parseIr`, `normalizeIr`, `renderIr`, `renderSource`, `resolveTemplateRef`.
- Command semantics (`build`, `dev`, `check` flags, config files, stdio, exit codes, interactivity) belong to the TypeScript CLI, **not** this crate — no clap wrapper here.
- One diagnostic shape: `code` + `severity` + `stage` + `message` + `span` + `path` + `argsJson` + `related`.
- `context`, `catalog`, `options`, and `delimiters` are JSON strings at the N-API boundary — avoids object vs `serde_json::Value` type drift.

## Delimiters

`ParseRequest` and `RenderSourceRequest` accept an optional `delimiters` JSON string:

```json
{
  "syntaxMode": "template",
  "template": {
    "codeStart": "{%",
    "codeEnd": "%}",
    "commentStart": "{#",
    "commentEnd": "#}",
    "supportFilterPipe": true,
    "legacyFor": false
  }
}
```

Omitting it defaults to Dejavu delimiters (`<% %>` / `<# #>`). Doki and other hosts that need non-default delimiters **must** pass this field — the N-API layer otherwise falls through to the default parser.

## catalog schema

The `catalog` JSON string on `renderIr` / `renderSource`:

```json
{
  "extensions": ["", ".dejavu"],
  "roots": [
    {
      "name": "pages",
      "scheme": "pages",
      "priority": 0,
      "basePath": "templates",
      "files": {
        "index.dejavu": "{ \"irVersion\": \"1.0\", \"language\": { ... }, \"body\": { ... } }"
      }
    }
  ]
}
```

Values under `files` are pre-parsed IR JSON strings — the host parses each template with `parseIr` first, then passes the full catalog into `renderSource`.
