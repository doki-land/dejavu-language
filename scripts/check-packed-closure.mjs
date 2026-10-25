#!/usr/bin/env node
/**
 * Packed-closure gate: consume the published Dejavu tarballs like an end user.
 *
 *   node scripts/check-packed-closure.mjs
 *   node scripts/check-packed-closure.mjs --host win32-x64 --version 0.0.1
 *   node scripts/check-packed-closure.mjs --keep
 *
 * Flow:
 *   1. pin the release versions with `scripts/set-release-version.mjs`
 *   2. `npm pack` the engine, the facade, the current platform addon (and the
 *      wasm bridge on Linux when it has been built)
 *   3. create a throwaway consumer project in the OS temp dir and install only
 *      those tarballs (never the registry copies)
 *   4. assert CLI version output, JIT render/escaping, AOT compile and execution
 *      of the generated ESM through `@doki-land/dejavu-engine`, and that the
 *      native platform package really landed in `node_modules/@doki-land`
 *
 * The version rewrite is limited to the publishable package manifests and is
 * restored on the way out, so the work tree never keeps release versions.
 *
 * Preconditions: the platform addon must already exist at
 * `projects/dejavu.ts/platforms/dejavu-<host>/dejavu.<triple>.node`, and the
 * TypeScript packages must be built (`dist/`). Both are produced by
 * `node scripts/build/napi.mjs --release` and `pnpm --filter ... build`.
 *
 * A cross-compiled target (e.g. `darwin-x64` on an Apple Silicon runner) is
 * skipped: npm refuses the `cpu` field and the addon cannot be loaded there, so
 * that matrix leg can only prove the build, not the consumer closure.
 */
import { spawnSync } from "node:child_process";
import {
    existsSync,
    mkdirSync,
    mkdtempSync,
    readFileSync,
    readdirSync,
    rmSync,
    writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
    JS_PACKAGES,
    PLATFORM_NPM_ROOT,
    PLATFORM_PACKAGES,
    SMOKE_VERSION,
    findPlatformByHost,
    nodeFileName,
} from "./dejavu-release-packages.mjs";

const ROOT = resolve(fileURLToPath(new URL("..", import.meta.url)));
const FACADE_DIR = "projects/dejavu.ts/packages/dejavu";
const ENGINE_DIR = "projects/dejavu.ts/packages/dejavu-engine";
const WASM_DIR = "projects/dejavu.ts/platforms/dejavu-unknown-wasm32";
const CONTRACT_PREFIX = "dejavu.napi/";
const VERSION_PATTERN = /^\d+\.\d+\.\d+([.-].*)?$/;
const FACADE_NAME = "@doki-land/dejavu";
const ENGINE_NAME = "@doki-land/dejavu-engine";

const RENDER_TEMPLATE = "<p><% name %></p>";
const RENDER_ESCAPED = "<p>&lt;b&gt;&amp;</p>";
const SAFE_TEMPLATE = "<% html |> safe %>";
const SAFE_OUTPUT = "<em>ok</em>";

function parseArgs(argv) {
    const options = { host: undefined, version: SMOKE_VERSION, keep: false };
    for (let i = 0; i < argv.length; i += 1) {
        const arg = argv[i];
        if (arg === "--keep") {
            options.keep = true;
            continue;
        }
        if (arg === "--host" || arg === "--version") {
            const value = argv[i + 1];
            if (!value) throw new Error(`${arg} requires a value`);
            options[arg.slice(2)] = value;
            i += 1;
            continue;
        }
        throw new Error(`unknown argument: ${arg}`);
    }
    if (!VERSION_PATTERN.test(options.version)) {
        throw new Error(`--version must be X.Y.Z (got ${options.version})`);
    }
    return options;
}

function defaultHost() {
    const host = `${process.platform}-${process.arch}`;
    if (!findPlatformByHost(host)) {
        const supported = PLATFORM_PACKAGES.map((platform) => platform.host).join(", ");
        throw new Error(`no platform package for ${host} (supported: ${supported})`);
    }
    return host;
}

/** `npm_cli.js` next to the running Node, so no shell quoting is involved. */
function npmCliPath() {
    const execPath = process.env.npm_execpath;
    if (execPath && execPath.endsWith(".js") && existsSync(execPath)) return execPath;
    const binDir = dirname(process.execPath);
    const candidates = [
        join(binDir, "node_modules", "npm", "bin", "npm-cli.js"),
        join(binDir, "..", "lib", "node_modules", "npm", "bin", "npm-cli.js"),
    ];
    return candidates.find((candidate) => existsSync(candidate));
}

function runNpm(args, cwd) {
    const cli = npmCliPath();
    if (cli) return spawnSync(process.execPath, [cli, ...args], { cwd, encoding: "utf8" });
    const command = process.platform === "win32" ? "npm.cmd" : "npm";
    const quoted = process.platform === "win32" ? args.map((arg) => `"${arg}"`) : args;
    return spawnSync(command, quoted, {
        cwd,
        encoding: "utf8",
        shell: process.platform === "win32",
    });
}

function runNode(args, cwd) {
    return spawnSync(process.execPath, args, { cwd, encoding: "utf8" });
}

function mustRun(result, label) {
    if (result.status === 0) return result;
    const output = `${result.stdout ?? ""}${result.stderr ?? ""}`.trim();
    throw new Error(`${label} failed with status ${result.status ?? "unknown"}${output ? `\n${output}` : ""}`);
}

function requireFile(path, hint) {
    if (!existsSync(path)) throw new Error(`missing ${rel(path)}${hint ? `\n  ${hint}` : ""}`);
    return path;
}

function rel(path) {
    return path.startsWith(ROOT) ? path.slice(ROOT.length + 1).replaceAll("\\", "/") : path;
}

/** Every manifest `scripts/set-release-version.mjs` rewrites. */
function releaseManifests() {
    return [
        ...JS_PACKAGES.map((pkg) => join(ROOT, pkg.path, "package.json")),
        ...PLATFORM_PACKAGES.map((pkg) => join(ROOT, PLATFORM_NPM_ROOT, pkg.npmDir, "package.json")),
    ].filter((file) => existsSync(file));
}

function snapshotManifests() {
    const files = releaseManifests();
    if (files.length !== JS_PACKAGES.length + PLATFORM_PACKAGES.length) {
        throw new Error("release manifests are missing, cannot pin and restore versions safely");
    }
    return new Map(files.map((file) => [file, readFileSync(file, "utf8")]));
}

function restoreManifests(snapshot) {
    let restored = 0;
    for (const [file, content] of snapshot) {
        if (readFileSync(file, "utf8") === content) continue;
        writeFileSync(file, content);
        restored += 1;
    }
    return restored;
}

function packTarball(packageDir, destination) {
    const result = mustRun(
        runNpm(["pack", "--pack-destination", destination, "--json"], packageDir),
        `npm pack ${rel(packageDir)}`,
    );
    let descriptor;
    try {
        [descriptor] = JSON.parse(result.stdout);
    } catch {
        throw new Error(`npm pack produced no JSON for ${rel(packageDir)}\n${result.stdout}`);
    }
    const tarball = join(destination, descriptor.filename);
    return requireFile(tarball, `npm pack ${rel(packageDir)} did not write ${descriptor.filename}`);
}

function readInstalledManifest(consumerDir, name) {
    const file = join(consumerDir, "node_modules", ...name.split("/"), "package.json");
    requireFile(file, `package ${name} is not installed`);
    return JSON.parse(readFileSync(file, "utf8"));
}

function main() {
    const options = parseArgs(process.argv.slice(2));
    const host = options.host ?? defaultHost();
    const platform = findPlatformByHost(host);
    if (!platform) throw new Error(`unknown host ${host} (see scripts/dejavu-release-packages.mjs)`);
    const addonName = nodeFileName(platform);
    const addonPath = join(ROOT, PLATFORM_NPM_ROOT, platform.npmDir, addonName);

    requireFile(
        addonPath,
        `build the ${host} addon first: node scripts/build/napi.mjs --release --target ${platform.rustTarget}`,
    );
    for (const [dir, entry] of [
        [FACADE_DIR, "dist/cli.js"],
        [ENGINE_DIR, "dist/runtime/index.js"],
    ]) {
        requireFile(
            join(ROOT, dir, entry),
            `build the TypeScript packages first: pnpm --filter ${dir === FACADE_DIR ? FACADE_NAME : ENGINE_NAME} build`,
        );
    }

    const targetArch = platform.nodeTriple.includes("arm64") ? "arm64" : "x64";
    if (process.arch !== targetArch) {
        // A cross-compiled addon cannot be installed (npm rejects the `cpu`
        // field) or loaded here, so there is no consumer closure to prove.
        console.log(
            `skip: ${platform.nodeTriple} cannot be consumed on ${process.platform}-${process.arch} (cross-compiled target)`,
        );
        return;
    }

    const tempRoot = mkdtempSync(join(tmpdir(), "dejavu-packed-closure-"));
    const tarballDir = join(tempRoot, "tarballs");
    const consumerDir = join(tempRoot, "consumer");
    mkdirSync(tarballDir, { recursive: true });
    mkdirSync(consumerDir, { recursive: true });

    const snapshot = snapshotManifests();
    let failure;
    try {
        console.log(`packed closure: ${platform.name}@${options.version} on ${process.platform}-${process.arch}`);

        mustRun(
            runNode([join(ROOT, "scripts", "set-release-version.mjs"), options.version], ROOT),
            "set-release-version",
        );

        const tarballs = [
            packTarball(join(ROOT, ENGINE_DIR), tarballDir),
            packTarball(join(ROOT, FACADE_DIR), tarballDir),
            packTarball(join(ROOT, PLATFORM_NPM_ROOT, platform.npmDir), tarballDir),
        ];
        const wasmDir = join(ROOT, WASM_DIR);
        const wasmBuilt = existsSync(join(wasmDir, "src", "lib", "dejavu_wasm.js"));
        if (host.startsWith("linux-") && wasmBuilt) {
            tarballs.push(packTarball(wasmDir, tarballDir));
        }
        for (const tarball of tarballs) console.log(`packed ${rel(tarball) || tarball}`);

        writeFileSync(
            join(consumerDir, "package.json"),
            `${JSON.stringify(
                {
                    name: "dejavu-packed-consumer",
                    private: true,
                    version: "0.0.0",
                    type: "module",
                },
                null,
                2,
            )}\n`,
        );

        mustRun(
            runNpm(["install", "--no-audit", "--no-fund", "--save-exact", ...tarballs], consumerDir),
            "npm install <local tarballs>",
        );

        const installed = readdirSync(join(consumerDir, "node_modules", "@doki-land")).sort();
        console.log(`installed ${installed.map((name) => `@doki-land/${name}`).join(", ")}`);
        if (!installed.includes(platform.npmDir)) {
            throw new Error(`${platform.name} was not installed (optional dependency closure broken)`);
        }
        requireFile(
            join(consumerDir, "node_modules", ...platform.name.split("/"), addonName),
            "installed platform package does not carry its native addon",
        );

        for (const name of [FACADE_NAME, ENGINE_NAME, platform.name]) {
            const manifest = readInstalledManifest(consumerDir, name);
            if (manifest.version !== options.version) {
                throw new Error(`${name}@${manifest.version} installed, expected ${options.version}`);
            }
        }

        const cliPath = join(consumerDir, "node_modules", ...FACADE_NAME.split("/"), "dist", "cli.js");

        const versionResult = mustRun(runNode([cliPath, "version"], consumerDir), "dejavu version");
        const versionLine = versionResult.stdout.trim();
        console.log(`version: ${versionLine}`);
        if (!versionLine.includes(FACADE_NAME)) {
            throw new Error(`dejavu version output does not name ${FACADE_NAME}: ${versionLine}`);
        }
        if (!versionLine.includes(CONTRACT_PREFIX)) {
            throw new Error(`dejavu version output has no binding contract: ${versionLine}`);
        }

        writeFileSync(
            join(consumerDir, "probe-render.mjs"),
            `import assert from "node:assert/strict";
import { render } from ${JSON.stringify(FACADE_NAME)};

const escaped = render(${JSON.stringify(RENDER_TEMPLATE)}, { name: "<b>&" });
assert.equal(escaped, ${JSON.stringify(RENDER_ESCAPED)}, "JIT render must HTML-escape interpolations");
const safe = render(${JSON.stringify(SAFE_TEMPLATE)}, { html: ${JSON.stringify(SAFE_OUTPUT)} });
assert.equal(safe, ${JSON.stringify(SAFE_OUTPUT)}, "the safe filter must bypass escaping");
console.log(\`render=\${escaped}\`);
console.log(\`safe=\${safe}\`);
`,
        );
        const renderResult = mustRun(runNode([join(consumerDir, "probe-render.mjs")], consumerDir), "render probe");
        for (const line of renderResult.stdout.trim().split("\n")) console.log(line);

        const templateFile = join(consumerDir, "hello.dejavu");
        const moduleFile = join(consumerDir, "hello.js");
        writeFileSync(templateFile, RENDER_TEMPLATE);
        mustRun(runNode([cliPath, "compile", templateFile, "--out", moduleFile], consumerDir), "dejavu compile");
        const generated = readFileSync(moduleFile, "utf8");
        if (!generated.includes(`"${ENGINE_NAME}/runtime"`)) {
            throw new Error("compiled ESM does not import the published AOT runtime");
        }
        if (!generated.includes("export function render")) {
            throw new Error("compiled file does not export a render() function");
        }
        console.log(`compile: wrote ${rel(moduleFile) || "hello.js"} (${generated.length} bytes)`);

        writeFileSync(
            join(consumerDir, "probe-aot.mjs"),
            `import assert from "node:assert/strict";
import { htmlEscape } from ${JSON.stringify(ENGINE_NAME)};
import { render } from "./hello.js";

assert.equal(htmlEscape("<b>"), "&lt;b&gt;");
const html = render({ name: "Ada" });
assert.equal(html, "<p>Ada</p>", "generated ESM must render through the engine runtime");
console.log(\`runtime=\${html}\`);
`,
        );
        const aotResult = mustRun(runNode([join(consumerDir, "probe-aot.mjs")], consumerDir), "AOT runtime probe");
        for (const line of aotResult.stdout.trim().split("\n")) console.log(line);

        console.log(`✔ packed closure verified for ${platform.name}@${options.version}`);
    } catch (error) {
        failure = error;
    } finally {
        const restored = restoreManifests(snapshot);
        if (restored > 0) console.log(`restored ${restored} release manifests`);
        if (options.keep) {
            console.log(`kept ${tempRoot}`);
        } else {
            rmSync(tempRoot, { recursive: true, force: true });
        }
    }
    if (failure) throw failure;
}

try {
    main();
} catch (error) {
    console.error(`✘ ${error instanceof Error ? error.message : String(error)}`);
    process.exit(1);
}
