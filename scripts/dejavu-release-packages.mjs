/**
 * Dejavu — shared release package / platform matrix.
 * Mirrors doki-engine/scripts/doki-release-packages.mjs exactly.
 *
 * Tree:
 *   compilers/dejavu           — Rust CLI crate (dejavu-tools bin)
 *   compilers/dejavu/npm/<plat>— published platform packages (os/cpu constrained)
 *   projects/packages/dejavu   — @doki-land/dejavu (TS API)
 *
 * Platform npm names: `@doki-land/dejavu-<os>-<cpu>`.
 * No `-cli` prefix. No `-gnu`/`-musl` suffix — matches @doki-land/nifty.
 */
export const SMOKE_VERSION = '0.0.0'

/**
 * @type {{
 *   host: string, npmDir: string, rustTarget: string,
 *   name: string, os: string[], cpu: string[]
 * }[]}
 */
export const PLATFORM_PACKAGES = [
  {
    host: 'linux-x64',
    npmDir: 'linux-x64',
    rustTarget: 'x86_64-unknown-linux-gnu',
    name: '@doki-land/dejavu-linux-x64',
    os: ['linux'],
    cpu: ['x64'],
  },
  {
    host: 'linux-arm64',
    npmDir: 'linux-arm64',
    rustTarget: 'aarch64-unknown-linux-gnu',
    name: '@doki-land/dejavu-linux-arm64',
    os: ['linux'],
    cpu: ['arm64'],
  },
  {
    host: 'darwin-x64',
    npmDir: 'darwin-x64',
    rustTarget: 'x86_64-apple-darwin',
    name: '@doki-land/dejavu-darwin-x64',
    os: ['darwin'],
    cpu: ['x64'],
  },
  {
    host: 'darwin-arm64',
    npmDir: 'darwin-arm64',
    rustTarget: 'aarch64-apple-darwin',
    name: '@doki-land/dejavu-darwin-arm64',
    os: ['darwin'],
    cpu: ['arm64'],
  },
  {
    host: 'win32-x64',
    npmDir: 'win32-x64',
    rustTarget: 'x86_64-pc-windows-msvc',
    name: '@doki-land/dejavu-win32-x64',
    os: ['win32'],
    cpu: ['x64'],
  },
  {
    host: 'win32-arm64',
    npmDir: 'win32-arm64',
    rustTarget: 'aarch64-pc-windows-msvc',
    name: '@doki-land/dejavu-win32-arm64',
    os: ['win32'],
    cpu: ['arm64'],
  },
]

/** Platform package root (CI stages binaries here) */
export const PLATFORM_NPM_ROOT = 'compilers/dejavu/npm'

/** Publish order: platform binaries first → @doki-land/dejavu (TS API) last */
export const JS_PACKAGES = [
  {
    name: '@doki-land/dejavu',
    path: 'projects/packages/dejavu',
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
