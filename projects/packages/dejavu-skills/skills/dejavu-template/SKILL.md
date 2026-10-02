---
name: dejavu-template
description: Help users author, review, generate, and integrate Dejavu templates while preserving shared IR, escaping, loader, and cross-host conformance semantics.
---

# Dejavu Template

Use this skill for Dejavu template authoring, generation models, rendering, escaping, inheritance/includes, loader resolution, AOT/runtime choices, or a new host integration.

## Core boundary

- Dejavu owns template syntax, shared IR, rendering behavior, escaping, filters, inheritance, include resolution, diagnostics, and conformance.
- The caller owns domain semantics and supplies a stable generation model or render context.
- A framework adapter may connect a host to Dejavu, but must not copy or reinterpret the parser and renderer.

Templates render established meaning. Do not hide schema resolution, database planning, RPC identity, authorization policy, or other product semantics inside template expressions.

## Workflow

1. Identify the public Dejavu facade and existing template contract used by the project.
2. Define the smallest explicit context/generation model required by the output.
3. Choose AOT or runtime rendering based on the host's actual supported surface, not assumed performance claims.
4. Review escaping and loader roots before adding raw output, includes, or inheritance.
5. Run shared conformance when changing observable parse/render behavior or adding a host.

Do not claim parity from package names or placeholder implementations. When a host lacks the required conformance level, state the gap and keep the shared IR contract authoritative.
