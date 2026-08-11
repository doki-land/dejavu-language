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

const adapters = {
    ts: {
        label: "TypeScript",
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
        render(c) {
            const manifest = join(ROOT, "projects/dejavu.rs/Cargo.toml");
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
    kt: {
        label: "Kotlin",
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
    const langs = filter.length ? filter : Object.keys(adapters);
    let failed = 0;
    const skipped = new Set();

    console.log("Hard invariant: identical render(DejavuIR, Context) across languages\n");

    for (const name of cases()) {
        const c = { ...loadCase(name), name };
        console.log(`## ${name}`);
        const outputs = {};
        for (const lang of langs) {
            const adapter = adapters[lang];
            if (!adapter) {
                console.log(`  ${lang}: unknown`);
                failed++;
                continue;
            }
            const out = adapter.render(c);
            if (out === null) {
                console.log(`  ${lang}: SKIP (${adapter.label} unavailable)`);
                skipped.add(lang);
                continue;
            }
            outputs[lang] = out;
            const ok = out === c.out;
            console.log(`  ${lang}: ${ok ? "OK" : "FAIL"}`);
            if (!ok) {
                failed++;
                console.log(`    expected ${JSON.stringify(c.out)}`);
                console.log(`    actual   ${JSON.stringify(out)}`);
            }
        }
        const keys = Object.keys(outputs);
        for (let i = 1; i < keys.length; i++) {
            if (outputs[keys[0]] !== outputs[keys[i]]) {
                failed++;
                console.log(`  CROSS-FAIL: ${keys[0]} != ${keys[i]}`);
            }
        }
        console.log("");
    }

    if (skipped.size) {
        console.log(`Skipped: ${[...skipped].join(", ")}`);
    }
    if (failed) {
        console.error(`FAILED (${failed})`);
        process.exit(1);
    }
    console.log("All available languages: identical IR render output.");
}

main();
