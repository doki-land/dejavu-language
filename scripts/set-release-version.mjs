#!/usr/bin/env node
/**
 * Align publishable Dejavu package versions.
 * Mirrors doki-engine/scripts/set-release-version.mjs shape.
 *
 * Usage: node scripts/set-release-version.mjs 0.0.1
 *
 * CI runs this right before `npm publish`, so the optional platform
 * dependencies must become concrete versions for the published tarball. Do not
 * commit that rewrite: the workspace needs `workspace:*` to link the platform
 * packages during install, otherwise every CI job loses the native addon.
 */
import { readFileSync, writeFileSync, existsSync } from 'node:fs'
import { join } from 'node:path'
import {
  PLATFORM_PACKAGES,
  PLATFORM_NPM_ROOT,
  JS_PACKAGES,
  optionalDepsMap,
} from './dejavu-release-packages.mjs'

const version = process.argv[2]
if (!version || !/^\d+\.\d+\.\d+([.-].*)?$/.test(version)) {
  console.error('usage: node scripts/set-release-version.mjs <X.Y.Z>')
  process.exit(1)
}

const root = process.cwd()

// Facade @doki-land/dejavu (TS API) — optionalDependencies pinned
for (const pkg of JS_PACKAGES) {
  const file = join(root, pkg.path, 'package.json')
  if (existsSync(file)) {
    const json = JSON.parse(readFileSync(file, 'utf8'))
    json.version = version
    if (json.optionalDependencies) {
      for (const k of Object.keys(json.optionalDependencies)) {
        json.optionalDependencies[k] = version
      }
    }
    writeFileSync(file, `${JSON.stringify(json, null, 2)}\n`)
    console.log(`set ${json.name}@${version}`)
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
