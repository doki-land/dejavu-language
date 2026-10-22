#!/usr/bin/env node
/**
 * Publish minimal 0.0.0 stubs so every Trusted-Publishing target exists on npm.
 * Mirrors doki-engine/scripts/publish-trusted-stubs.mjs exactly.
 *
 * Does not touch workspace package.json versions and does not build.
 * Already-published packages are skipped (not an error).
 *
 * Usage (from dejavu-language root):
 *   node scripts/publish-trusted-stubs.mjs --dry-run
 *   node scripts/publish-trusted-stubs.mjs
 *   node scripts/publish-trusted-stubs.mjs --otp 123456
 *   NPM_OTP=123456 node scripts/publish-trusted-stubs.mjs
 *   node scripts/publish-trusted-stubs.mjs --platforms-only
 */
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'
import { PLATFORM_PACKAGES, SMOKE_VERSION } from './dejavu-release-packages.mjs'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const dryRun = process.argv.includes('--dry-run')
const platformsOnly = process.argv.includes('--platforms-only')
const otpArgIdx = process.argv.indexOf('--otp')
const otpOverride =
  otpArgIdx >= 0 ? process.argv[otpArgIdx + 1] : process.env.NPM_OTP || null

/**
 * @typedef {{
 *   name: string, description: string,
 *   os?: string[], cpu?: string[],
 *   kind: 'js' | 'platform',
 * }} StubPkg
 */

/** @type {StubPkg[]} */
const JS_STUBS = [
  {
    name: '@doki-land/dejavu',
    description: 'Dejavu template engine API (Trusted Publishing placeholder)',
    kind: 'js',
  },
]

/** @returns {StubPkg[]} */
function platformStubs() {
  return PLATFORM_PACKAGES.map((p) => ({
    name: p.name,
    description: `Platform binary for @doki-land/dejavu (${p.npmDir}) — Trusted Publishing placeholder`,
    os: p.os,
    cpu: p.cpu,
    kind: 'platform',
  }))
}

function npmViewVersion(name) {
  const r = spawnSync('npm', ['view', name, 'version', '--json'], {
    encoding: 'utf8', shell: true, windowsHide: true,
  })
  if (r.status !== 0) return null
  try {
    const v = JSON.parse(r.stdout.trim())
    return typeof v === 'string' ? v : null
  } catch {
    return r.stdout.trim() || null
  }
}

/** @param {StubPkg} pkg */
function publishStub(pkg) {
  const dir = mkdtempSync(join(tmpdir(), 'dejavu-oidc-stub-'))
  try {
    /** @type {Record<string, unknown>} */
    const meta = {
      name: pkg.name,
      version: SMOKE_VERSION,
      description: pkg.description,
      license: 'MPL-2.0',
      private: false,
      main: 'index.js',
      files: ['index.js', 'README.md'],
      publishConfig: {
        access: 'public',
        registry: 'https://registry.npmjs.org/',
      },
      repository: {
        type: 'git',
        url: 'git+https://github.com/doki-land/dejavu-language.git',
      },
    }
    if (pkg.os) meta.os = pkg.os
    if (pkg.cpu) meta.cpu = pkg.cpu

    writeFileSync(join(dir, 'package.json'), JSON.stringify(meta, null, 2) + '\n')
    writeFileSync(
      join(dir, 'index.js'),
      `throw new Error(${JSON.stringify(
        `${pkg.name}@${SMOKE_VERSION} is an npm Trusted Publishing placeholder. Install a later version from release-npm.yml.`,
      )});\n`,
    )
    writeFileSync(
      join(dir, 'README.md'),
      `# ${pkg.name}\n\n` +
        `This **${SMOKE_VERSION}** release is a placeholder so ` +
        `[npm Trusted Publishing](https://docs.npmjs.com/trusted-publishers/) ` +
        `can be configured for \`doki-land/dejavu-language\` / \`release-npm.yml\`.\n\n` +
        `Do not depend on this version. Real artifacts ship on the next semver.\n`,
    )

    if (dryRun) {
      console.log(`[dry-run] would publish ${pkg.name}@${SMOKE_VERSION}`)
      return true
    }

    const publishArgs = ['publish', '--access', 'public']
    if (otpOverride) publishArgs.push('--otp', otpOverride)
    const r = spawnSync('npm', publishArgs, {
      cwd: dir, encoding: 'utf8', shell: true, stdio: 'inherit', windowsHide: false,
    })
    return r.status === 0
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
}

function main() {
  const packages = platformsOnly ? platformStubs() : [...JS_STUBS, ...platformStubs()]

  console.log(
    `trusted stubs @${SMOKE_VERSION} count=${packages.length}` +
      ` dryRun=${dryRun} platformsOnly=${platformsOnly} otp=${otpOverride ? 'yes' : 'no'}`,
  )

  if (!dryRun) {
    const who = spawnSync('npm', ['whoami'], { encoding: 'utf8', shell: true, windowsHide: true })
    if (who.status !== 0) {
      console.error('npm is not authenticated. Run: npm login')
      process.exit(1)
    }
    console.log(`npm user: ${who.stdout.trim()}`)
  }

  let published = 0, skipped = 0, failed = 0
  /** @type {string[]} */
  const failedNames = []

  for (const pkg of packages) {
    const existing = npmViewVersion(pkg.name)
    if (existing) {
      console.log(`skip  ${pkg.name} (already ${existing})`)
      skipped++
      continue
    }
    console.log(`\n=== publish ${pkg.name}@${SMOKE_VERSION} ===`)
    if (publishStub(pkg)) {
      published++
      console.log(`  → https://www.npmjs.com/package/${pkg.name}/access`)
    } else {
      failed++
      failedNames.push(pkg.name)
      console.error(`  FAILED ${pkg.name}`)
    }
  }

  console.log(`\ndone: published=${published} skipped=${skipped} failed=${failed}`)
  if (failedNames.length) {
    console.error('failed packages:')
    for (const n of failedNames) console.error(`  - ${n}`)
    console.error('\nIf npm asked for OTP, re-run with: node scripts/publish-trusted-stubs.mjs --otp <code>')
    process.exit(1)
  }
  if (!dryRun && published > 0) {
    console.log(
      '\nNext: on each new package page → Access → Trusted Publishers → add\n' +
        '  repository: doki-land/dejavu-language\n' +
        '  workflow: release-npm.yml\n' +
        'Then CI can publish 0.0.1+ without interactive OTP.',
    )
  }
}

main()
