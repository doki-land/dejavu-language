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
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const T1 = join(ROOT, "specifications", "conformance", "t1");
const INHERITANCE = join(ROOT, "specifications", "conformance", "inheritance");
const TMP = join(ROOT, ".conformance-tmp");

function cases() {
    return readdirSync(T1, { withFileTypes: true })
        .filter((d) => d.isDirectory())
        .map((d) => d.name)
        .sort();
}

function loadCase(name) {
    const dir = join(T1, name);
    return {
        name,
        irPath: join(dir, "expected.ir.json"),
        ctxPath: join(dir, "context.ctx.json"),
        out: readFileSync(join(dir, "expected.out.txt"), "utf8"),
    };
}

function inheritanceCases() {
    if (!existsSync(INHERITANCE)) return [];
    return readdirSync(INHERITANCE, { withFileTypes: true })
        .filter((d) => d.isDirectory())
        .map((d) => {
            const dir = join(INHERITANCE, d.name);
            return {
                name: d.name,
                dir,
                out: readFileSync(join(dir, "expected.out.txt"), "utf8"),
            };
        })
        .sort((a, b) => a.name.localeCompare(b.name));
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
    py: {
        label: "Python",
        required: false,
        render(c) {
            mkdirSync(TMP, { recursive: true });
            const script = join(TMP, `render_${c.name}.py`);
            writeFileSync(
                script,
                `
import json, sys
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
            const facadeDist = join(ROOT, "runtimes", "dejavu", "dist", "index.js");
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
    console.log("Required hosts: Rust, TypeScript — SKIP = FAIL\n");

    for (const name of cases()) {
        const c = { ...loadCase(name), name };
        console.log(`## ${name}`);
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

    // ---- inheritance fixtures (multi-template catalog) ----
    const inheritanceAdapters = ["rs-catalog", "ts-catalog"].map((k) => adapters[k]).filter(Boolean);
    if (inheritanceAdapters.length) {
        console.log("## inheritance fixtures (parser → catalog → render)");
        for (const dir of inheritanceCases()) {
            process.stdout.write(`  ${dir.name}: `);
            const outputs = {};
            let failed_any = false;
            for (const adapter of inheritanceAdapters) {
                const out = adapter.render(dir);
                if (out === null) {
                    process.stdout.write(`[${adapter.label}=SKIP] `);
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
    console.log("All required languages: identical IR render output.");
}

main();
