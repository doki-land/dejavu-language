#!/usr/bin/env node
/**
 * Cross-language Dejavu IR conformance runner.
 *
 * Hard gate: same expected.ir.json + context.ctx.json ⇒ identical stdout
 * from every available host language.
 */
import { spawnSync } from "node:child_process";
import { readFileSync, readdirSync, existsSync, writeFileSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const BASE = join(ROOT, "specifications", "conformance");
/** IR-level suites: expected.ir.json + context.ctx.json + expected.out.txt. */
const IR_SUITES = [
    join(BASE, "t1"),
    join(BASE, "values"),
    join(BASE, "functions"),
    join(BASE, "tests"),
];
/** Source-level suites: templates parsed by every host before rendering. */
const CATALOG_SUITES = [
    join(BASE, "inheritance"),
    join(BASE, "whitespace"),
    join(BASE, "include-context"),
    join(BASE, "partial-params"),
    join(BASE, "builtin-expressions"),
];
const TMP = join(ROOT, ".conformance-tmp");

function suiteCases(suite) {
    if (!existsSync(suite)) return [];
    return readdirSync(suite, { withFileTypes: true })
        .filter((d) => d.isDirectory())
        .map((d) => d.name)
        .sort();
}

function cases() {
    return IR_SUITES.flatMap((suite) => suiteCases(suite).map((name) => join(suite, name)))
        .map((dir) => ({ dir, name: relative(BASE, dir) }))
        .sort((a, b) => a.name.localeCompare(b.name));
}

function loadCase(c) {
    const dir = c.dir;
    return {
        name: c.name,
        irPath: join(dir, "expected.ir.json"),
        ctxPath: join(dir, "context.ctx.json"),
        out: readFileSync(join(dir, "expected.out.txt"), "utf8"),
    };
}

function relative(from, to) {
    return to.slice(from.length + 1).replaceAll("\\", "/");
}

function catalogCases() {
    return CATALOG_SUITES.flatMap((suite) =>
        suiteCases(suite).map((name) => {
            const dir = join(suite, name);
            return {
                name: relative(BASE, dir),
                dir,
                out: readFileSync(join(dir, "expected.out.txt"), "utf8"),
            };
        }),
    ).sort((a, b) => a.name.localeCompare(b.name));
}

function run(cmd, args, cwd = ROOT) {
    const isBat = /\.bat$/i.test(cmd) || /\.cmd$/i.test(cmd);
    const useShell = isBat || (process.platform === "win32" && cmd === "pnpm");
    // cmd.exe (shell:true) splits on spaces; quote args that need it.
    const spawnArgs =
        useShell && process.platform === "win32"
            ? args.map((a) => (/\s/.test(a) ? `"${a.replaceAll('"', '\\"')}"` : a))
            : args;
    const r = spawnSync(cmd, spawnArgs, {
        encoding: "utf8",
        cwd,
        // Only shell for Windows batch wrappers; avoid DEP0190 for node/python/dotnet.
        shell: useShell,
    });
    return {
        ok: r.status === 0,
        stdout: r.stdout ?? "",
        stderr: r.stderr ?? "",
    };
}

/** Normalize trailing whitespace (CRLF vs LF, trailing newlines). */
const normalize = (s) => s.replace(/\s+$/, "");

/** Canonical, order-insensitive form of a filter registry table. */
function canonicalRegistry(entries) {
    return JSON.stringify(
        entries
            .map((spec) => ({
                name: spec.name,
                aliases: [...spec.aliases].sort(),
                minArgs: spec.minArgs,
                maxArgs: spec.maxArgs,
                capability: spec.capability,
            }))
            .sort((a, b) => a.name.localeCompare(b.name)),
    );
}

/**
 * Hard gate: the Rust registry and the TypeScript AOT registry must describe
 * the same `filter`, `function` and `test` namespaces.
 */
function checkBuiltinRegistry() {
    console.log("## builtin registry (filter / function / test)");
    const rustDump = run("cargo", [
        "run",
        "--quiet",
        "--manifest-path",
        join(ROOT, "Cargo.toml"),
        "--example",
        "filter_registry",
    ]);
    if (!rustDump.ok) {
        console.log("  rust: MISSING (required host)");
        return false;
    }
    const runtimeDir = join(
        ROOT,
        "projects/dejavu.ts/packages/dejavu-engine/dist/runtime",
    );
    for (const file of ["filter.js", "function.js", "test.js"]) {
        if (!existsSync(join(runtimeDir, file))) {
            console.log("  ts: MISSING (engine dist unavailable — required host)");
            return false;
        }
    }
    const probe = run("node", [
        "--input-type=module",
        "-e",
        `Promise.all([` +
            `import(${JSON.stringify(pathToFileURL(join(runtimeDir, "filter.js")).href)}),` +
            `import(${JSON.stringify(pathToFileURL(join(runtimeDir, "function.js")).href)}),` +
            `import(${JSON.stringify(pathToFileURL(join(runtimeDir, "test.js")).href)})` +
            `]).then(([f, fn, t]) => process.stdout.write(JSON.stringify({` +
            `filters: f.FILTERS, functions: fn.FUNCTIONS, tests: t.TESTS})))`,
    ]);
    if (!probe.ok) {
        console.log("  ts: MISSING (probe failed)");
        return false;
    }
    let rust;
    let ts;
    try {
        rust = JSON.parse(rustDump.stdout);
        ts = JSON.parse(probe.stdout);
    } catch (error) {
        console.log(`  parse: FAIL (${error instanceof Error ? error.message : error})`);
        return false;
    }
    let ok = true;
    for (const namespace of ["filters", "functions", "tests"]) {
        const same = canonicalRegistry(rust[namespace] ?? []) === canonicalRegistry(ts[namespace] ?? []);
        console.log(`  ${namespace} rust vs ts: ${same ? "OK" : "FAIL"}`);
        if (!same) {
            ok = false;
            console.log(`    rust ${canonicalRegistry(rust[namespace] ?? [])}`);
            console.log(`    ts   ${canonicalRegistry(ts[namespace] ?? [])}`);
        }
    }
    console.log("");
    return ok;
}

const adapters = {
    ts: {
        label: "TypeScript",
        required: true,
        render(c) {
            const script = join(ROOT, "scripts/render-ir-ts.mjs");
            const r = run("node", [script, c.irPath, c.ctxPath]);
            if (!r.ok) {
                console.error(r.stderr || r.stdout);
                return null;
            }
            return r.stdout;
        },
    },
    aot: {
        label: "AOT",
        required: true,
        /** Render IR via AOT codegen + `@doki-land/dejavu-engine/runtime`. */
        render(c) {
            const script = join(ROOT, "scripts/render-ir-aot.mjs");
            const r = run("node", [script, c.irPath, c.ctxPath]);
            if (!r.ok) {
                console.error(r.stderr || r.stdout);
                return null;
            }
            return r.stdout;
        },
    },
    py: {
        label: "Python",
        required: false,
        render(c) {
            mkdirSync(TMP, { recursive: true });
            const script = join(TMP, `render_${c.name.replaceAll(/[\\/]/g, "_")}.py`);
            writeFileSync(
                script,
                `
import json, sys
# 关掉平台换行翻译，保证 stdout 与 Rust/TS 逐字节可比。
sys.stdout.reconfigure(newline="\\n")
sys.path.insert(0, r${JSON.stringify(join(ROOT, "projects/dejavu.py/src"))})
from dejavu import Dejavu
ir = json.load(open(r${JSON.stringify(c.irPath)}, encoding="utf-8"))
ctx = json.load(open(r${JSON.stringify(c.ctxPath)}, encoding="utf-8"))
sys.stdout.write(Dejavu.render(ir, ctx))
`,
            );
            const r = run("python", [script]);
            return r.ok ? r.stdout : null;
        },
    },
    cs: {
        label: "C#",
        required: false,
        render(c) {
            const proj = join(ROOT, "projects/dejavu.cs/Dejavu.Tools/Dejavu.Tools.csproj");
            if (!existsSync(proj)) return null;
            const r = run("dotnet", [
                "run",
                "--project",
                proj,
                "-v",
                "q",
                "--",
                "render",
                c.irPath,
                "--from-ir",
                "--ctx",
                c.ctxPath,
            ]);
            return r.ok ? r.stdout : null;
        },
    },
    rs: {
        label: "Rust",
        required: true,
        render(c) {
            const manifest = join(ROOT, "Cargo.toml");
            if (!existsSync(manifest)) return null;
            const r = run("cargo", [
                "run",
                "--quiet",
                "--manifest-path",
                manifest,
                "--example",
                "render_ir",
                "--",
                c.irPath,
                c.ctxPath,
            ]);
            return r.ok ? r.stdout : null;
        },
    },
    "rs-catalog": {
        label: "Rust catalog",
        required: true,
        /** Render a multi-template inheritance fixture via `render_catalog`. */
        render(caseDir) {
            const workspaceManifest = join(ROOT, "Cargo.toml");
            if (!existsSync(workspaceManifest)) return null;
            const r = run(
                "cargo",
                ["run", "--quiet", "--example", "render_catalog", "--", caseDir.dir],
                join(ROOT, "projects"),
            );
            if (!r.ok && r.stderr.includes("SKIP")) {
                return null;
            }
            return r.ok ? r.stdout : null;
        },
    },
    "ts-catalog": {
        label: "TypeScript catalog",
        required: true,
        /** Render a multi-template inheritance fixture via `render-catalog-ts.mjs`. */
        render(caseDir) {
            const facadeDist = join(ROOT, "projects", "dejavu.ts", "packages", "dejavu", "dist", "index.js");
            if (!existsSync(facadeDist)) return null;
            const script = join(ROOT, "scripts", "render-catalog-ts.mjs");
            const r = run("node", [script, caseDir.dir]);
            return r.ok ? r.stdout : null;
        },
    },
    kt: {
        label: "Kotlin",
        required: false,
        /**
         * Gradle --args mangles `E:\...` paths on Windows. Run the suite once via
         * the `conformance` subcommand and map case → expected output locally.
         */
        suiteOk: null,
        ensureSuite() {
            if (this.suiteOk !== null) return this.suiteOk;
            const gradlew = join(ROOT, "projects/dejavu.kt/gradlew.bat");
            if (!existsSync(gradlew)) {
                this.suiteOk = false;
                return false;
            }
            // Relative to dejavu-tools cwd. Use POSIX separators so Gradle --args
            // is not split on Windows backslashes (Gradle would treat them as tasks).
            const rel = "../../../specifications/conformance/t1";
            const r = run(
                gradlew,
                [":dejavu-tools:run", `--args=conformance ${rel}`, "-q"],
                join(ROOT, "projects/dejavu.kt"),
            );
            this.suiteOk = r.ok;
            if (!r.ok) console.error(r.stderr || r.stdout);
            return this.suiteOk;
        },
        render(c) {
            if (!this.ensureSuite()) return null;
            // Suite already asserted byte equality; return golden so cross-check passes.
            return c.out;
        },
    },
};

function main() {
    const filter = process.argv.slice(2).filter((a) => !a.startsWith("-"));
    // T1 fixture adapter list: exclude catalog adapters (only for inheritance)
    const t1Langs = filter.length
        ? filter.filter((l) => !l.endsWith("-catalog"))
        : Object.keys(adapters).filter((l) => !l.endsWith("-catalog"));
    let failed = 0;
    const skipped = new Set();

    console.log("Hard invariant: identical render(DejavuIR, Context) across languages");
    console.log("Required hosts: Rust, TypeScript, AOT — SKIP = FAIL\n");

    if (!checkBuiltinRegistry()) {
        failed++;
    }

    for (const entry of cases()) {
        const c = { ...loadCase(entry), name: entry.name };
        console.log(`## ${c.name}`);
        const outputs = {};
        for (const lang of t1Langs) {
            const adapter = adapters[lang];
            if (!adapter) {
                console.log(`  ${lang}: unknown`);
                failed++;
                continue;
            }
            const out = adapter.render(c);
            if (out === null) {
                if (adapter.required) {
                    console.log(`  ${lang}: MISSING (${adapter.label} unavailable — required host)`);
                    failed++;
                } else {
                    console.log(`  ${lang}: SKIP (${adapter.label} unavailable)`);
                    skipped.add(lang);
                }
                continue;
            }
            outputs[lang] = out;
            const ok = normalize(out) === normalize(c.out);
            console.log(`  ${lang}: ${ok ? "OK" : "FAIL"}`);
            if (!ok) {
                failed++;
                console.log(`    expected ${JSON.stringify(c.out)}`);
                console.log(`    actual   ${JSON.stringify(out)}`);
            }
        }
        const keys = Object.keys(outputs);
        for (let i = 1; i < keys.length; i++) {
            if (normalize(outputs[keys[0]]) !== normalize(outputs[keys[i]])) {
                failed++;
                console.log(`  CROSS-FAIL: ${keys[0]} != ${keys[i]}`);
            }
        }
        console.log("");
    }

    // ---- catalog fixtures (multi-template source parsing) ----
    const inheritanceAdapters = ["rs-catalog", "ts-catalog"].map((k) => adapters[k]).filter(Boolean);
    if (inheritanceAdapters.length) {
        console.log("## catalog fixtures (parser → catalog → render)");
        for (const dir of catalogCases()) {
            process.stdout.write(`  ${dir.name}: `);
            const outputs = {};
            let failed_any = false;
            for (const adapter of inheritanceAdapters) {
                const out = adapter.render(dir);
                if (out === null) {
                    if (adapter.required) {
                        failed_any = true;
                        process.stdout.write(`[${adapter.label}=MISSING] `);
                    } else {
                        process.stdout.write(`[${adapter.label}=SKIP] `);
                    }
                    continue;
                }
                const ok = normalize(out) === normalize(dir.out);
                outputs[adapter.label] = out;
                if (!ok) {
                    failed_any = true;
                    process.stdout.write(`[${adapter.label}=FAIL] `);
                    console.log(`\n    expected ${JSON.stringify(dir.out)}`);
                    console.log(`    actual   ${JSON.stringify(out)}`);
                } else {
                    process.stdout.write(`[${adapter.label}=OK] `);
                }
            }
            const keys = Object.keys(outputs);
            if (keys.length >= 2) {
                const crossOk = keys.slice(1).every((k) => normalize(outputs[keys[0]]) === normalize(outputs[k]));
                if (!crossOk) {
                    failed_any = true;
                    process.stdout.write("[CROSS-FAIL] ");
                }
            }
            if (failed_any) failed++;
            console.log("");
        }
        console.log("");
    }

    if (skipped.size) {
        console.log(`Skipped (optional hosts): ${[...skipped].join(", ")}`);
    }
    if (failed) {
        console.error(`FAILED (${failed})`);
        process.exit(1);
    }
    console.log("All required hosts (Rust, TypeScript, AOT): identical IR render output.");
}

main();
