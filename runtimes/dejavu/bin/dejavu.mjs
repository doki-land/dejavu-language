#!/usr/bin/env node
/**
 * @doki-land/dejavu — TypeScript CLI entry.
 * Command semantics live in `src/cli/`; the Rust engine is reached via N-API.
 */
import { runCli } from '../dist/cli/cli.js'

process.exitCode = await runCli(process.argv.slice(2))
