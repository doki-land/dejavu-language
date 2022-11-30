# ✨ Dejavu

**One template surface, consistent across hosts.**

Dejavu separates template intent from the host runtime. Write a template once, then pick the binding that fits your app.
A shared IR keeps behavior aligned.

## 👀 At a glance

```dejavu
Hello, <% name %>!
```

Templates stay close to the output they describe. The intermediate representation (IR) carries shared semantics across
hosts.

## ✨ What you get

- One template language surface
- Shared IR and conformance fixtures
- AOT compilation where a host supports it; runtime rendering otherwise
- Host adapters kept separate from the public `dejavu` facade

## 🧩 Hosts

| Host       | Public surface  |
|------------|-----------------|
| TypeScript | `dejavu`        |
| Rust       | `dejavu`        |
| C#         | `Dejavu`        |
| Kotlin     | `dejavu`        |
| CLI        | `dejavu` binary |

Install and API details live under each host tree.

## 📚 Documentation

| Kind           | Start here                                                                                                                                                            |
|----------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Developer docs | [`documentation/index.md`](./documentation/index.md)                                                                                                                  |
| User docs      | [`documents/en-us/index.md`](./projects/dejavu.ts/homepage/documents/en-us/index.md) · [`zh-hans/index.md`](./projects/dejavu.ts/homepage/documents/zh-hans/index.md) |
| Specs          | [`specifications/`](./specifications/)                                                                                                                                |

Map: [`documentation/readme.md`](./documentation/readme.md).

## 📄 License

[MPL-2.0](./License.md)
