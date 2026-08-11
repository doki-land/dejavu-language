# Dejavu

Dejavu parses templates into a shared intermediate representation and renders that IR with a JSON-compatible context. The current contract covers interpolation, conditions, loops, filters, HTML escaping, inheritance, includes, and template loading.

```dejavu
Hello, <% account.name %>!
```

Application developers using TypeScript import the public facade from `@doki-land/dejavu`. Engine-layer packages are for binding and core maintainers, not ordinary application code.

## Documentation

- [English user documentation](./projects/dejavu.ts/homepage/documents/en-us/index.md)
- [Simplified Chinese user documentation](./projects/dejavu.ts/homepage/documents/zh-hans/index.md)
- [Contributor and core developer documentation](./documentation/index.md)
- [Host-independent specifications](./specifications/)
- [Implementation compatibility](./documentation/compatibility.md)

The specification defines required behavior; the compatibility page records which host implementations have verified that behavior. Do not infer implementation completeness from the list of host directories.

## Repository layout

- `projects/dejavu.ts`, `.rs`, `.cs`, `.kt`, `.py`: host implementations and bindings
- `specifications/`: Template Contract, IR schema, and conformance fixtures
- `documentation/`: contributor workflows and implementation notes
- `projects/dejavu.ts/homepage/documents/`: external user documentation

## License

[MPL-2.0](./License.md)
