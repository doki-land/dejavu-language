# Use another host binding

The Dejavu language and IR contracts are host-independent. A host binding decides how applications install the library,
construct context values, load templates, and surface diagnostics.

## Available implementations

The repository contains implementations or integration work for TypeScript, Rust, C#, Kotlin, and Python. Their feature
coverage is not identical. Do not infer support from the presence of a directory or package alone.

Before adopting a binding, check:

- whether it parses source or only renders existing IR;
- which conformance suites it runs;
- whether inheritance and loader resolution are covered;
- whether `safe` / `raw` and strict missing-value behavior are tested;
- whether its public package has been released for your ecosystem.

The current evidence is recorded in [implementation compatibility](../reference/compatibility.md).

## Binding contract

A conforming binding should expose equivalent operations for:

- parsing source into Dejavu IR;
- rendering IR with a JSON-compatible context;
- rendering source as a convenience operation;
- reporting parse, resolution, and render diagnostics;
- loading referenced templates when inheritance or includes are enabled.

Framework adapters should call a host binding rather than define new template syntax. Adapter-specific routing, request
handling, authentication, and file discovery remain outside the Dejavu language contract.

If a binding is missing a feature, treat that as an implementation limitation rather than changing the template language
for that host.
