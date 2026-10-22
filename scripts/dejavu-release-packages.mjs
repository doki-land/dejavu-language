/**
 * Dejavu CLI — shared release package / platform matrix.
 *
 * Platform npm names: `@doki-land/dejavu-cli-<os>-<cpu>-<libc?>`.
 * Windows: no `-msvc` suffix (`dejavu-cli-win32-x64`).
 * Binary filename: `dejavu` on Unix, `dejavu.exe` on Windows.
 */

/** @type {{ host: string, npmDir: string, rustTarget: string, name: string, os: string[], cpu: string[], libc?: string, binName: string }[]} */
export const PLATFORM_PACKAGES = [
  {
    host: 'linux-x64-gnu',
    npmDir: 'linux-x64-gnu',
    rustTarget: 'x86_64-unknown-linux-gnu',
    name: '@doki-land/dejavu-cli-linux-x64-gnu',
    os: ['linux'],
    cpu: ['x64'],
    libc: 'glibc',
    binName: 'dejavu',
  },
  {
    host: 'linux-arm64-gnu',
    npmDir: 'linux-arm64-gnu',
    rustTarget: 'aarch64-unknown-linux-gnu',
    name: '@doki-land/dejavu-cli-linux-arm64-gnu',
    os: ['linux'],
    cpu: ['arm64'],
    libc: 'glibc',
    binName: 'dejavu',
  },
  {
    host: 'linux-x64-musl',
    npmDir: 'linux-x64-musl',
    rustTarget: 'x86_64-unknown-linux-musl',
    name: '@doki-land/dejavu-cli-linux-x64-musl',
    os: ['linux'],
    cpu: ['x64'],
    libc: 'musl',
    binName: 'dejavu',
  },
  {
    host: 'linux-arm64-musl',
    npmDir: 'linux-arm64-musl',
    rustTarget: 'aarch64-unknown-linux-musl',
    name: '@doki-land/dejavu-cli-linux-arm64-musl',
    os: ['linux'],
    cpu: ['arm64'],
    libc: 'musl',
    binName: 'dejavu',
  },
  {
    host: 'darwin-x64',
    npmDir: 'darwin-x64',
    rustTarget: 'x86_64-apple-darwin',
    name: '@doki-land/dejavu-cli-darwin-x64',
    os: ['darwin'],
    cpu: ['x64'],
    binName: 'dejavu',
  },
  {
    host: 'darwin-arm64',
    npmDir: 'darwin-arm64',
    rustTarget: 'aarch64-apple-darwin',
    name: '@doki-land/dejavu-cli-darwin-arm64',
    os: ['darwin'],
    cpu: ['arm64'],
    binName: 'dejavu',
  },
  {
    host: 'win32-x64',
    npmDir: 'win32-x64',
    rustTarget: 'x86_64-pc-windows-msvc',
    name: '@doki-land/dejavu-cli-win32-x64',
    os: ['win32'],
    cpu: ['x64'],
    binName: 'dejavu.exe',
  },
  {
    host: 'win32-arm64',
    npmDir: 'win32-arm64',
    rustTarget: 'aarch64-pc-windows-msvc',
    name: '@doki-land/dejavu-cli-win32-arm64',
    os: ['win32'],
    cpu: ['arm64'],
    binName: 'dejavu.exe',
  },
]

/** Platform npm package root (CI stages binaries here) */
export const PLATFORM_NPM_ROOT = 'compilers/dejavu-cli/npm'

/** Meta-package that pulls the right platform binary */
export const CLI_META_PACKAGE = {
  name: '@doki-land/dejavu-cli',
  path: 'runtimes/dejavu-cli',
}

/** Existing @doki-land/dejavu (TS engine — separate from CLI) */
export const JS_PACKAGE = {
  name: '@doki-land/dejavu',
  path: 'projects/packages/dejavu',
}

export function optionalDepsMap(version) {
  const out = {}
  for (const p of PLATFORM_PACKAGES) out[p.name] = version
  return out
}

export function findPlatformByHost(host) {
  return PLATFORM_PACKAGES.find((p) => p.host === host)
}
