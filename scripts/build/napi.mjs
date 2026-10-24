/**
 * Build the Dejavu N-API addon and stage it into the matching platform package.
 *
 *   node scripts/build/napi.mjs                 # host debug build
 *   node scripts/build/napi.mjs --release       # host release build
 *   node scripts/build/napi.mjs --release --target aarch64-apple-darwin
 *
 * Output: `projects/dejavu.ts/platforms/dejavu-<host>/dejavu.<nodeTriple>.node`
 * That directory is what the release workflow uploads and publishes.
 */
import { spawnSync } from 'node:child_process'
import { copyFileSync, existsSync, mkdirSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  PLATFORM_NPM_ROOT,
  findPlatformByRustTarget,
  nodeFileName,
} from '../dejavu-release-packages.mjs'

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)))
const args = process.argv.slice(2)

const release = args.includes('--release')
const targetFlag = args.indexOf('--target')
const requestedTarget = targetFlag >= 0 ? args[targetFlag + 1] : undefined

if (targetFlag >= 0 && !requestedTarget) {
  console.error('napi: --target requires a Rust target triple')
  process.exit(2)
}

const rustTarget = requestedTarget ?? rustcHost()
const platform = findPlatformByRustTarget(rustTarget)
if (!platform) {
  console.error(
    `napi: unsupported Rust target ${rustTarget}. Add it to scripts/dejavu-release-packages.mjs first.`,
  )
  process.exit(2)
}

const profile = release ? 'release' : 'debug'
const cargoArgs = ['build', '-p', 'dejavu-napi']
if (release) cargoArgs.push('--release')
if (requestedTarget) cargoArgs.push('--target', rustTarget)

console.log(`napi: ${cargoArgs.join(' ')}`)
const cargo = spawnSync(process.env.CARGO ?? 'cargo', cargoArgs, { cwd: ROOT, stdio: 'inherit' })
if (cargo.status !== 0) {
  console.error(`napi: cargo build failed with status ${cargo.status ?? 'unknown'}`)
  process.exit(cargo.status ?? 1)
}

const libraryNames = ['dejavu_napi.dll', 'libdejavu_napi.so', 'libdejavu_napi.dylib']
const searchDirs = [join(ROOT, 'target', rustTarget, profile), join(ROOT, 'target', profile)]
const built = searchDirs
  .flatMap((dir) => libraryNames.map((name) => join(dir, name)))
  .find((candidate) => existsSync(candidate))

if (!built) {
  console.error(`napi: no dejavu_napi library found in ${searchDirs.join(' or ')}`)
  process.exit(1)
}

const packageDir = join(ROOT, PLATFORM_NPM_ROOT, platform.npmDir)
mkdirSync(packageDir, { recursive: true })
const outFile = join(packageDir, nodeFileName(platform))
copyFileSync(built, outFile)

console.log(`napi: ${built}`)
console.log(`napi: staged ${outFile}`)

function rustcHost() {
  const probe = spawnSync(process.env.RUSTC ?? 'rustc', ['-vV'], { encoding: 'utf8' })
  if (probe.status !== 0) {
    console.error('napi: rustc -vV failed; pass --target explicitly')
    process.exit(1)
  }
  const host = /^host:\s*(\S+)$/m.exec(probe.stdout)
  if (!host) {
    console.error('napi: could not parse rustc host triple')
    process.exit(1)
  }
  return host[1]
}
