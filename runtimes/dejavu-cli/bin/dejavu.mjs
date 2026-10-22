#!/usr/bin/env node
/**
 * dejavu CLI shim — resolves the correct platform binary and execs it.
 * Install-time optionalDependencies ensure the right platform package is present.
 */
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));

/** os/cpu → platform package name + binary filename */
const PLATFORM_MAP = {
  'linux-x64-gnu':    ['@doki-land/dejavu-cli-linux-x64-gnu',    'dejavu'],
  'linux-arm64-gnu':  ['@doki-land/dejavu-cli-linux-arm64-gnu',  'dejavu'],
  'linux-x64-musl':   ['@doki-land/dejavu-cli-linux-x64-musl',   'dejavu'],
  'linux-arm64-musl': ['@doki-land/dejavu-cli-linux-arm64-musl', 'dejavu'],
  'darwin-x64':       ['@doki-land/dejavu-cli-darwin-x64',       'dejavu'],
  'darwin-arm64':     ['@doki-land/dejavu-cli-darwin-arm64',     'dejavu'],
  'win32-x64':        ['@doki-land/dejavu-cli-win32-x64',        'dejavu.exe'],
  'win32-arm64':      ['@doki-land/dejavu-cli-win32-arm64',      'dejavu.exe'],
};

function detectKey() {
  const os = process.platform;   // linux | darwin | win32
  const cpu = process.arch;       // x64 | arm64
  const libc = process.report?.getReport?.().header?.glibcVersionRuntime ? 'gnu'
             : (os === 'linux' ? 'gnu' : undefined);

  if (os === 'linux') {
    // Try musl first — Alpine users have only the musl package
    const musl = `${os}-${cpu}-musl`;
    if (PLATFORM_MAP[musl]) return musl;
    return `${os}-${cpu}-${libc || 'gnu'}`;
  }
  return `${os}-${cpu}`;
}

function resolveBinary() {
  const key = detectKey();
  const [pkg, bin] = PLATFORM_MAP[key] || [];
  if (!pkg) {
    console.error(`@doki-land/dejavu-cli: unsupported platform ${process.platform}-${process.arch}`);
    process.exit(1);
  }
  try {
    // require.resolve finds the installed optionalDependency
    const pkgPath = require.resolve(`${pkg}/package.json`);
    return join(dirname(pkgPath), bin);
  } catch {
    console.error(`@doki-land/dejavu-cli: platform package ${pkg} not installed.`);
    console.error(`Run: npm install ${pkg}@latest`);
    process.exit(1);
  }
}

const binPath = resolveBinary();
const result = spawnSync(binPath, process.argv.slice(2), {
  stdio: 'inherit',
  env: process.env,
});
if (result.error) {
  console.error(`dejavu: failed to spawn ${binPath}: ${result.error.message}`);
  process.exit(1);
}
process.exit(result.status ?? 0);
