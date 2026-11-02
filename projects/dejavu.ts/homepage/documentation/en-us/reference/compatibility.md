# Implementation compatibility

This page separates the portable Dejavu specification from evidence about individual host implementations.

## How to read the status

- **Verified** means the named suite completed and checked its expected output.
- **Partially verified** means some cases completed but the full claimed surface did not.
- **Unverified** means the repository does not currently provide reliable evidence for the claimed surface.

## Current evidence

Status observed on 2026-08-11 with `pnpm conformance` from the repository root:

| Host       | T1 IR rendering    | Inheritance / loader                | Notes                                                                                    |
|------------|--------------------|-------------------------------------|------------------------------------------------------------------------------------------|
| TypeScript | Partially verified | Verified by TypeScript engine tests | Core T1 cases passed, but the `safe_raw` cross-host case did not complete in the runner. |
| Rust       | Partially verified | Unverified on the shared runner     | Core T1 IR cases passed; full loader parity is not established by this command.          |
| C#         | Partially verified | Unverified on the shared runner     | Core T1 IR cases passed; full loader parity is not established by this command.          |
| Python     | Partially verified | Unverified on the shared runner     | Core T1 IR cases passed; full loader parity is not established by this command.          |
| Kotlin     | Unverified         | Unverified                          | The runner skipped Kotlin in the observed environment.                                   |

The conformance command currently exits successfully when a case is skipped by every host. Until that runner behavior is
corrected, do not use its final success line as proof of complete cross-host conformance.

## Portable language surface

The normative contract currently covers interpolation, comments, `if`, `loop`, expressions, `|>` filters, default HTML
escaping, `safe` / `raw`, inheritance, includes, and loader resolution. Deferred features are listed in the Template
Contract rather than advertised as host capabilities.

Pin host package versions and review their release notes before shipping templates across multiple runtimes.
