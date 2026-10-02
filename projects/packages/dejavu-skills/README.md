# `@doki-land/dejavu-skills`

Agent Skills for writing and reviewing Dejavu templates, choosing the right render path, and integrating a host without creating a second template language.

## Install

```bash
npx skills add @doki-land/dejavu-skills --skill dejavu-template -y
```

Preview or install globally:

```bash
npx skills add @doki-land/dejavu-skills --list
npx skills add @doki-land/dejavu-skills --skill dejavu-template -y -g
```

## Start with a prompt

```text
Turn this generated Rust module into a Dejavu template and define the smallest
generation model that keeps business semantics out of the template.

Review these templates for unsafe escaping, loader ambiguity, and behavior that
would diverge between AOT and runtime rendering.

Add Dejavu to this host and identify which shared conformance fixtures it must pass.
```

The installer requires Node.js 18+. This package supplies guidance; rendering and validation remain the responsibility of the Dejavu facade and tools used by the project.
