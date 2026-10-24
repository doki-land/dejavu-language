/**
 * Dejavu — shared release package / platform matrix.
 * Mirrors doki-engine/scripts/doki-release-packages.mjs exactly.
 *
 * Tree:
 *   projects/dejavu.rs/crates/dejavu-napi  — N-API engine surface (cdylib)
 *   projects/dejavu.ts/platforms/<plat>   — published platform packages carrying `.node`
 *   projects/dejavu.ts/packages/dejavu    — @doki-land/dejavu TypeScript CLI + facade
 *
 * Platform npm names: `@doki-land/dejavu-<os>-<cpu>`.
 * No `-cli` prefix. No `-gnu`/`-musl` suffix — matches @doki-land/nifty.
 *
 * Each platform package carries exactly one native addon, named
 * `dejavu.<nodeTriple>.node`, so the host loader can resolve `pkg/main`.
 */
export const SMOKE_VERSION = '0.0.0'

/**
 * @type {{
 *   host: string, npmDir: string, rustTarget: string, nodeTriple: string,
 *   name: string, os: string[], cpu: string[]
 * }[]}
 */
export const PLATFORM_PACKAGES = [
  {
    host: 'linux-x64',
    npmDir: 'dejavu-linux-x64',
    rustTarget: 'x86_64-unknown-linux-gnu',
    nodeTriple: 'linux-x64-gnu',
    name: '@doki-land/dejavu-linux-x64',
    os: ['linux'],
    cpu: ['x64'],
  },
  {
    host: 'linux-arm64',
    npmDir: 'dejavu-linux-arm64',
    rustTarget: 'aarch64-unknown-linux-gnu',
    nodeTriple: 'linux-arm64-gnu',
    name: '@doki-land/dejavu-linux-arm64',
    os: ['linux'],
    cpu: ['arm64'],
  },
  {
    host: 'darwin-x64',
    npmDir: 'dejavu-darwin-x64',
    rustTarget: 'x86_64-apple-darwin',
    nodeTriple: 'darwin-x64',
    name: '@doki-land/dejavu-darwin-x64',
    os: ['darwin'],
    cpu: ['x64'],
  },
  {
    host: 'darwin-arm64',
    npmDir: 'dejavu-darwin-arm64',
    rustTarget: 'aarch64-apple-darwin',
    nodeTriple: 'darwin-arm64',
    name: '@doki-land/dejavu-darwin-arm64',
    os: ['darwin'],
    cpu: ['arm64'],
  },
  {
    host: 'win32-x64',
    npmDir: 'dejavu-win32-x64',
    rustTarget: 'x86_64-pc-windows-msvc',
    nodeTriple: 'win32-x64-msvc',
    name: '@doki-land/dejavu-win32-x64',
    os: ['win32'],
    cpu: ['x64'],
  },
  {
    host: 'win32-arm64',
    npmDir: 'dejavu-win32-arm64',
    rustTarget: 'aarch64-pc-windows-msvc',
    nodeTriple: 'win32-arm64-msvc',
    name: '@doki-land/dejavu-win32-arm64',
    os: ['win32'],
    cpu: ['arm64'],
  },
]

/** Platform package root (CI stages native addons here) */
export const PLATFORM_NPM_ROOT = 'projects/dejavu.ts/platforms'

/** Native addon filename inside each platform package. */
export function nodeFileName(platform) {
  return `dejavu.${platform.nodeTriple}.node`
}

/** Publish order: platform addons first → runtime engine → @doki-land/dejavu meta-package last */
export const JS_PACKAGES = [
  {
    name: '@doki-land/dejavu-engine',
    path: 'projects/dejavu.ts/packages/dejavu-engine',
  },
  {
    name: '@doki-land/dejavu',
    path: 'projects/dejavu.ts/packages/dejavu',
  },
]

export function optionalDepsMap(version = SMOKE_VERSION) {
  /** @type {Record<string, string>} */
  const out = {}
  for (const p of PLATFORM_PACKAGES) out[p.name] = version
  return out
}

export function findPlatformByHost(host) {
  return PLATFORM_PACKAGES.find((p) => p.host === host)
}

export function findPlatformByNpmDir(npmDir) {
  return PLATFORM_PACKAGES.find((p) => p.npmDir === npmDir)
}

export function findPlatformByRustTarget(rustTarget) {
  return PLATFORM_PACKAGES.find((p) => p.rustTarget === rustTarget)
}
