#!/usr/bin/env node
/**
 * Align publishable Dejavu package versions.
 * Usage: node scripts/set-release-version.mjs 0.0.1
 */
import { readFileSync, writeFileSync, existsSync } from 'node:fs'
import { join } from 'node:path'
import {
  PLATFORM_PACKAGES,
  PLATFORM_NPM_ROOT,
  CLI_META_PACKAGE,
  JS_PACKAGE,
  optionalDepsMap,
} from './dejavu-release-packages.mjs'

const version = process.argv[2]
if (!version || !/^\d+\.\d+\.\d+([.-].*)?$/.test(version)) {
  console.error('usage: node scripts/set-release-version.mjs <X.Y.Z>')
  process.exit(1)
}

const root = process.cwd()

function rewriteDeps(deps, known) {
  if (!deps || typeof deps !== 'object') return deps
  const out = { ...deps }
  for (const [k, v] of Object.entries(out)) {
    if (!known.has(k) || typeof v !== 'string') continue
    if (v.startsWith('workspace:') || v.startsWith('file:')) out[k] = version
  }
  return out
}

const known = new Set([
  CLI_META_PACKAGE.name,
  JS_PACKAGE.name,
  ...PLATFORM_PACKAGES.map((p) => p.name),
])

// Meta-package @doki-land/dejavu-cli
{
  const file = join(root, CLI_META_PACKAGE.path, 'package.json')
  if (existsSync(file)) {
    const json = JSON.parse(readFileSync(file, 'utf8'))
    json.version = version
    json.optionalDependencies = optionalDepsMap(version)
    writeFileSync(file, `${JSON.stringify(json, null, 2)}\n`)
    console.log(`set ${json.name}@${version}`)
  }
}

// TS engine @doki-land/dejavu (only rewrites version — deps stay as-is)
{
  const file = join(root, JS_PACKAGE.path, 'package.json')
  if (existsSync(file)) {
    const json = JSON.parse(readFileSync(file, 'utf8'))
    json.version = version
    writeFileSync(file, `${JSON.stringify(json, null, 2)}\n`)
    console.log(`set ${json.name}@${version} (TS engine)`)
  }
}

// Platform package stubs
for (const plat of PLATFORM_PACKAGES) {
  const file = join(root, PLATFORM_NPM_ROOT, plat.npmDir, 'package.json')
  if (!existsSync(file)) {
    console.warn(`skip missing platform stub ${file}`)
    continue
  }
  const json = JSON.parse(readFileSync(file, 'utf8'))
  json.version = version
  writeFileSync(file, `${JSON.stringify(json, null, 2)}\n`)
  console.log(`set ${plat.name}@${version}`)
}

console.log(`\nAll packages pinned to ${version}.`)
