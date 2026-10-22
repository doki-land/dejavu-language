#!/usr/bin/env node
/**
 * @doki-land/dejavu — CLI shim.
 * Resolves the platform binary from optionalDependencies and execs it.
 * Naming matches @doki-land/nifty: @doki-land/dejavu-<os>-<cpu>.
 */
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const PLATFORM_MAP = {
  'linux-x64':   '@doki-land/dejavu-linux-x64',
  'linux-arm64': '@doki-land/dejavu-linux-arm64',
  'darwin-x64':  '@doki-land/dejavu-darwin-x64',
  'darwin-arm64':'@doki-land/dejavu-darwin-arm64',
  'win32-x64':   '@doki-land/dejavu-win32-x64',
  'win32-arm64': '@doki-land/dejavu-win32-arm64',
}

const key = `${process.platform}-${process.arch}`
const pkg = PLATFORM_MAP[key]
if (!pkg) {
  console.error(`@doki-land/dejavu: unsupported platform ${key}`)
  process.exit(1)
}

const binName = process.platform === 'win32' ? 'dejavu.exe' : 'dejavu'
let binPath
try {
  const pkgPath = require.resolve(`${pkg}/package.json`)
  binPath = join(dirname(pkgPath), binName)
} catch {
  console.error(`@doki-land/dejavu: platform package ${pkg} not installed.`)
  process.exit(1)
}

const result = spawnSync(binPath, process.argv.slice(2), { stdio: 'inherit', env: process.env })
if (result.error) { console.error(result.error.message); process.exit(1) }
process.exit(result.status ?? 0)
