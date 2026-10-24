import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
    mkdirSync,
    mkdtempSync,
    readdirSync,
    readFileSync,
    renameSync,
    rmSync,
    unlinkSync,
    writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { buildGeneratePlan, writeOutput } from "../dist/generate.js";

const BASE64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/** Decode one Base64 VLQ segment into its delta fields. */
function decodeSegment(segment) {
    const fields = [];
    let index = 0;
    while (index < segment.length) {
        let result = 0;
        let shift = 0;
        let digit;
        do {
            digit = BASE64.indexOf(segment[index++]);
            result += (digit & 31) << shift;
            shift += 5;
        } while (digit & 32);
        const negative = result & 1;
        result >>= 1;
        fields.push(negative ? -result : result);
    }
    return fields;
}

/**
 * Reduce `mappings` to the first segment of every generated line.
 *
 * Codegen emits one origin per construct, so the first segment on a line is the
 * template position that line starts at.
 */
function firstMappingPerLine(mappings) {
    const positions = new Map();
    let source = 0;
    let line = 0;
    let column = 0;
    mappings.split(";").forEach((entry, generatedLine) => {
        if (!entry) return;
        const [generatedColumn, sourceDelta, lineDelta, columnDelta] = decodeSegment(entry.split(",")[0]);
        assert.equal(generatedColumn, 0, "generated lines must map from column 0");
        source += sourceDelta;
        line += lineDelta;
        column += columnDelta;
        positions.set(generatedLine, { source, line, column });
    });
    return positions;
}

/** Write a project config whose `entries` map goes to `./out`. */
function writeConfig(dir, entries) {
    const lines = Object.entries(entries).map(
        ([name, reference]) => `      ${JSON.stringify(name)}: ${JSON.stringify(reference)},`,
    );
    writeFileSync(
        join(dir, "dejavu.config.ts"),
        [
            "export default {",
            '  roots: [{ name: "app", dir: "./templates" }],',
            "  generate: {",
            '    outDir: "./out",',
            "    entries: {",
            ...lines,
            "    },",
            "  },",
            "};",
            "",
        ].join("\n"),
    );
}

/** Write a throwaway project with the given templates and a matching config. */
function scaffold(templates) {
    const dir = mkdtempSync(join(tmpdir(), "dejavu-generate-test-"));
    writeFileSync(
        join(dir, "package.json"),
        `${JSON.stringify({ name: "fixture", private: true, type: "module" }, null, 2)}\n`,
    );
    mkdirSync(join(dir, "templates"), { recursive: true });
    for (const [name, text] of Object.entries(templates)) {
        writeFileSync(join(dir, "templates", name), text);
    }
    writeConfig(dir, { home: "app:home.dejavu" });
    return dir;
}

/** Capture every file's exact bytes plus every directory path under `root`. */
function snapshotTree(root) {
    const files = new Map();
    const directories = [];
    visit(root, "");
    return { files, directories };

    function visit(current, prefix) {
        for (const entry of readdirSync(current, { withFileTypes: true }).sort((a, b) =>
            a.name < b.name ? -1 : a.name > b.name ? 1 : 0,
        )) {
            const rel = prefix ? `${prefix}/${entry.name}` : entry.name;
            if (entry.isDirectory()) {
                directories.push(rel);
                visit(join(current, entry.name), rel);
            } else {
                files.set(rel, readFileSync(join(current, entry.name)));
            }
        }
    }
}

/** Relative staging/backup paths that a finished publish must have removed. */
function transactionResidue(root) {
    const residue = [];
    visit(root, "");
    return residue.sort();

    function visit(current, prefix) {
        for (const entry of readdirSync(current, { withFileTypes: true })) {
            const rel = prefix ? `${prefix}/${entry.name}` : entry.name;
            if (
                entry.name.startsWith(".dejavu-tx-") ||
                entry.name.endsWith(".dejavu-tmp") ||
                entry.name === ".dejavu.lock" ||
                entry.name === ".dejavu-transaction.json"
            ) {
                residue.push(rel);
                continue;
            }
            if (entry.isDirectory()) visit(join(current, entry.name), rel);
        }
    }
}

test("source maps point generated lines at their template spans", async (t) => {
    const template = "<h1><% title %></h1>\n<% if ok %>\n<p><% ok %></p>\n<% end if %>\n";
    const dir = scaffold({ "home.dejavu": template });
    t.after(() => rmSync(dir, { recursive: true, force: true }));

    const plan = await buildGeneratePlan({ cwd: dir });
    writeOutput(plan);

    const code = readFileSync(join(dir, "out", "home.js"), "utf8");
    const map = JSON.parse(readFileSync(join(dir, "out", "home.js.map"), "utf8"));

    // Stable logical source ids: no machine paths, and the template text ships inline.
    assert.deepEqual(map.sources, ["app:home.dejavu"]);
    assert.deepEqual(map.sourcesContent, [template]);
    assert.equal(map.file, "home.js");

    const positions = firstMappingPerLine(map.mappings);
    const lines = code.split("\n");
    const lineOf = (needle) => {
        const index = lines.findIndex((line) => line.includes(needle));
        assert.notEqual(index, -1, `generated code has no line containing ${needle}`);
        return index;
    };
    const evalLine = (binding) => {
        const index = lines.findIndex((line) => line.includes(`let ${binding} =`));
        assert.notEqual(index, -1, `generated code has no binding line for ${binding}`);
        return index;
    };
    const escapes = lines
        .map((line, index) => (line.includes("__out.push(htmlEscape(valueToString(__v") ? index : -1))
        .filter((index) => index !== -1);
    assert.equal(escapes.length, 2, "one interpolated push per interpolation");

    assert.deepEqual(positions.get(lineOf('__out.push("<h1>")')), { source: 0, line: 0, column: 0 });
    // Interpolation origins attach to the evaluation line, before the strictUndefined guard.
    assert.deepEqual(positions.get(evalLine("__v")), { source: 0, line: 0, column: 4 });
    assert.deepEqual(positions.get(lineOf("if (isTruthy(ctx.ok))")), { source: 0, line: 1, column: 0 });
    assert.deepEqual(positions.get(evalLine("__v1")), { source: 0, line: 2, column: 3 });
});

test("source maps stay byte-stable across regenerations", async (t) => {
    const dir = scaffold({ "home.dejavu": "<p><% value %></p>\n" });
    t.after(() => rmSync(dir, { recursive: true, force: true }));

    const first = await buildGeneratePlan({ cwd: dir });
    writeOutput(first);
    const map = readFileSync(join(dir, "out", "home.js.map"), "utf8");

    const second = await buildGeneratePlan({ cwd: dir });
    writeOutput(second);

    assert.equal(readFileSync(join(dir, "out", "home.js.map"), "utf8"), map);
    assert.notEqual(JSON.parse(map).mappings, "");
});

test("first generate publishes the planned directory and leaves no transaction residue", async (t) => {
    const dir = scaffold({ "home.dejavu": "<p><% value %></p>\n" });
    t.after(() => rmSync(dir, { recursive: true, force: true }));

    const plan = await buildGeneratePlan({ cwd: dir });
    writeOutput(plan);

    const out = join(dir, "out");
    for (const file of plan.files) {
        assert.equal(
            readFileSync(join(out, ...file.path.split("/")), "utf8"),
            file.content,
            `${file.path} does not match the plan`,
        );
    }

    const manifest = JSON.parse(readFileSync(join(out, "manifest.json"), "utf8"));
    assert.equal(manifest.contract, "dejavu.generate/v0");
    assert.equal(manifest.files.length, plan.files.length);

    const snapshot = snapshotTree(out);
    assert.deepEqual(
        [...snapshot.files.keys()].sort(),
        [...plan.files.map((file) => file.path), "manifest.json"].sort(),
    );
    assert.deepEqual(transactionResidue(out), []);
});

test("a failed second generate restores the previous output byte for byte", async (t) => {
    const dir = scaffold({ "home.dejavu": "<p>home</p>\n" });
    t.after(() => rmSync(dir, { recursive: true, force: true }));

    const first = await buildGeneratePlan({ cwd: dir });
    writeOutput(first);

    const out = join(dir, "out");
    const manifestBefore = readFileSync(join(out, "manifest.json"));

    // Replace the entry set and block the new module path with a directory.
    // The commit therefore fails after stale files were moved aside and some
    // replacements were already in place, which is exactly the rollback case.
    writeFileSync(join(dir, "templates", "zebra.dejavu"), "<p>zebra</p>\n");
    writeConfig(dir, { zebra: "app:zebra.dejavu" });
    mkdirSync(join(out, "zebra.js"), { recursive: true });

    const before = snapshotTree(out);
    const second = await buildGeneratePlan({ cwd: dir });

    assert.throws(() => writeOutput(second), /is not a regular file/);

    const after = snapshotTree(out);
    assert.deepEqual(after.files, before.files);
    assert.deepEqual(after.directories, before.directories);
    assert.deepEqual(readFileSync(join(out, "manifest.json")), manifestBefore);
    assert.deepEqual(transactionResidue(out), []);
});

const LOCK_FILE = ".dejavu.lock";
const TRANSACTION_RECORD = ".dejavu-transaction.json";

/** sha256 of exact bytes, in the manifest's hash format. */
function hashBytes(buffer) {
    return `sha256:${createHash("sha256").update(buffer).digest("hex")}`;
}

/** Write a lock record into an output directory. */
function writeLock(out, overrides = {}) {
    writeFileSync(
        join(out, LOCK_FILE),
        `${JSON.stringify(
            {
                contract: "dejavu.generate.lock/v0",
                pid: process.pid,
                startedAt: new Date().toISOString(),
                version: "test",
                ...overrides,
            },
            null,
            2,
        )}\n`,
    );
}

test("an active writer lock blocks the second publish and leaves the output untouched", async (t) => {
    const dir = scaffold({ "home.dejavu": "<p>home</p>\n" });
    t.after(() => rmSync(dir, { recursive: true, force: true }));

    const out = join(dir, "out");
    const first = await buildGeneratePlan({ cwd: dir });
    writeOutput(first);
    const before = snapshotTree(out);

    // Simulate another live writer: the lock carries this (alive) pid.
    writeLock(out);
    const locked = snapshotTree(out);

    const blocked = await buildGeneratePlan({ cwd: dir });
    assert.throws(() => writeOutput(blocked), /locked by an active dejavu generate run/);

    const untouched = snapshotTree(out);
    assert.deepEqual(untouched.files, locked.files);
    assert.deepEqual(untouched.directories, locked.directories);

    // Releasing the lock lets the same plan publish normally again.
    unlinkSync(join(out, LOCK_FILE));
    const resumed = await buildGeneratePlan({ cwd: dir });
    writeOutput(resumed);

    assert.deepEqual(snapshotTree(out).files, before.files);
    assert.deepEqual(transactionResidue(out), []);
});

test("an interrupted publish is rolled back before the next publish", async (t) => {
    const dir = scaffold({ "home.dejavu": "<p>home</p>\n" });
    t.after(() => rmSync(dir, { recursive: true, force: true }));

    const out = join(dir, "out");
    const first = await buildGeneratePlan({ cwd: dir });
    writeOutput(first);
    const before = snapshotTree(out);

    // Reconstruct a commit that died halfway through: originals already moved
    // to the backup area, new bytes partly in place, manifest replaced, staging
    // still present and the journal on disk.
    const txRoot = join(out, ".dejavu-tx-crash");
    mkdirSync(join(txRoot, "stage"), { recursive: true });
    mkdirSync(join(txRoot, "backup"), { recursive: true });

    renameSync(join(out, "home.js"), join(txRoot, "backup", "home.js"));
    writeFileSync(join(out, "home.js"), "// interrupted commit\n");

    writeFileSync(join(out, "extra.js"), "// half published\n");
    writeFileSync(join(txRoot, "stage", "extra.js"), "// half published\n");

    mkdirSync(join(out, "_internal"), { recursive: true });
    writeFileSync(join(out, "_internal", "part.js"), "// half published\n");

    renameSync(join(out, "manifest.json"), join(txRoot, "backup", "manifest.json"));
    writeFileSync(join(out, "manifest.json"), "{}\n");

    writeFileSync(
        join(out, TRANSACTION_RECORD),
        `${JSON.stringify(
            {
                contract: "dejavu.generate.tx/v0",
                phase: "commit",
                pid: process.pid,
                startedAt: new Date().toISOString(),
                version: "test",
                stageDir: ".dejavu-tx-crash/stage",
                backupDir: ".dejavu-tx-crash/backup",
                manifestPath: "manifest.json",
                before: [...before.files.entries()].map(([path, bytes]) => ({
                    path,
                    hash: hashBytes(bytes),
                })),
                createdDirs: ["_internal"],
                planned: ["manifest.json", "home.js", "extra.js", "_internal/part.js"],
            },
            null,
            2,
        )}\n`,
    );

    const second = await buildGeneratePlan({ cwd: dir });
    writeOutput(second);

    assert.ok(
        second.warnings.some((warning) => warning.code === "dejavu.generate.recovered"),
        "the run reports that it recovered the previous output",
    );

    // Recovery restored the pre-commit bytes and removed every residue before
    // the fresh publish produced the same output again.
    const after = snapshotTree(out);
    assert.deepEqual(after.files, before.files);
    assert.deepEqual(after.directories, before.directories);
    assert.deepEqual(readFileSync(join(out, "manifest.json")), before.files.get("manifest.json"));
    assert.deepEqual(transactionResidue(out), []);
});

test("recovery never deletes files the interrupted publish did not plan", async (t) => {
    const dir = scaffold({ "home.dejavu": "<p>home</p>\n" });
    t.after(() => rmSync(dir, { recursive: true, force: true }));

    const out = join(dir, "out");
    const first = await buildGeneratePlan({ cwd: dir });
    writeOutput(first);
    const before = snapshotTree(out);

    const txRoot = join(out, ".dejavu-tx-crash");
    mkdirSync(join(txRoot, "stage"), { recursive: true });
    mkdirSync(join(txRoot, "backup"), { recursive: true });
    renameSync(join(out, "home.js"), join(txRoot, "backup", "home.js"));
    writeFileSync(join(out, "home.js"), "// interrupted commit\n");
    renameSync(join(out, "manifest.json"), join(txRoot, "backup", "manifest.json"));
    writeFileSync(join(out, "manifest.json"), "{}\n");
    writeFileSync(join(out, "notes.txt"), "user note written during the crash window\n");

    writeFileSync(
        join(out, TRANSACTION_RECORD),
        `${JSON.stringify(
            {
                contract: "dejavu.generate.tx/v0",
                phase: "commit",
                pid: process.pid,
                startedAt: new Date().toISOString(),
                version: "test",
                stageDir: ".dejavu-tx-crash/stage",
                backupDir: ".dejavu-tx-crash/backup",
                manifestPath: "manifest.json",
                before: [...before.files.entries()].map(([path, bytes]) => ({
                    path,
                    hash: hashBytes(bytes),
                })),
                createdDirs: [],
                planned: ["manifest.json", "home.js"],
            },
            null,
            2,
        )}\n`,
    );

    const second = await buildGeneratePlan({ cwd: dir });
    assert.throws(() => writeOutput(second), /unrecorded|not recorded|unknown/i);

    // Recovery restored the previous output and left the user file alone, so
    // the ownership check, not the recovery, reports the conflict.
    assert.equal(readFileSync(join(out, "home.js"), "utf8"), before.files.get("home.js").toString("utf8"));
    assert.equal(readFileSync(join(out, "notes.txt"), "utf8"), "user note written during the crash window\n");
    assert.deepEqual(transactionResidue(out), []);
});

test("a stale lock from a dead or expired run is taken over", async (t) => {
    const dir = scaffold({ "home.dejavu": "<p>home</p>\n" });
    t.after(() => rmSync(dir, { recursive: true, force: true }));

    const out = join(dir, "out");
    const first = await buildGeneratePlan({ cwd: dir });
    writeOutput(first);
    const before = snapshotTree(out);

    // Expired start time and a pid that cannot belong to a live writer.
    writeLock(out, { pid: 2147483646, startedAt: "2000-01-01T00:00:00.000Z", version: "0.0.0-stale" });

    const second = await buildGeneratePlan({ cwd: dir });
    writeOutput(second);

    assert.ok(
        second.warnings.some((warning) => warning.code === "dejavu.generate.stale-lock"),
        "the takeover is reported as a diagnostic",
    );
    assert.deepEqual(snapshotTree(out).files, before.files);
    assert.deepEqual(transactionResidue(out), []);
});
