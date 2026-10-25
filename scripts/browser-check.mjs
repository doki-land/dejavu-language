#!/usr/bin/env node
/**
 * Browser smoke check for the Dejavu wasm bridge.
 *
 * Serves the repository over a minimal HTTP server and runs the unmodified
 * `dejavu-language` host wrapper in headless Chrome twice: on the document
 * thread through a native import map, and inside a module worker. Both must
 * reproduce `expected.out.txt` byte-for-byte with zero diagnostics.
 *
 * Requires a local Chrome/Edge. Set `DEJAVU_BROWSER` to point at another
 * Chromium-compatible executable.
 */
import { spawn } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { dirname, extname, join, sep } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const WASM_ENTRY = join(
    ROOT,
    "projects",
    "dejavu.ts",
    "platforms",
    "dejavu-unknown-wasm32",
    "src",
    "lib",
    "dejavu_wasm_bg.wasm",
);
const DIST_ENTRY = join(ROOT, "projects", "dejavu.ts", "packages", "dejavu-language", "dist", "index.js");

const WASM_URL = "/projects/dejavu.ts/platforms/dejavu-unknown-wasm32/src/lib/dejavu_wasm_bg.wasm";
const GLUE_URL = "/projects/dejavu.ts/platforms/dejavu-unknown-wasm32/src/lib/dejavu_wasm.js";
const BRIDGE_URL = "/projects/dejavu.ts/packages/dejavu-language/dist/index.js";
const WORKER_URL = "/specifications/browser-smoke/worker.mjs";
const PAGE_URL = "/specifications/browser-smoke/index.html";
const BARE_SPECIFIER = "@doki-land/dejavu-unknown-wasm32";
const RESULT_PREFIX = "DEJAVU_BROWSER_RESULT ";
const BROWSER_TIMEOUT_MS = 120000;
const GATE_TIMEOUT_MS = 45000;
const GATE_IMAGE = Buffer.from("R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==", "base64");

/**
 * Source fixtures parse their own template, so they prove the whole
 * `renderSource` path. The two IR fixtures are byte-identical through the
 * bridge and cover value semantics without re-parsing.
 */
const SOURCE_FIXTURES = ["comment", "hello", "if_else", "loop_pipe", "member", "safe_raw"];
const IR_FIXTURES = ["objects", "truthiness"];

const MIME = new Map([
    [".html", "text/html; charset=utf-8"],
    [".js", "text/javascript; charset=utf-8"],
    [".mjs", "text/javascript; charset=utf-8"],
    [".json", "application/json; charset=utf-8"],
    [".wasm", "application/wasm"],
    [".txt", "text/plain; charset=utf-8"],
    [".dejavu", "text/plain; charset=utf-8"],
]);

function requireFile(path, hint) {
    if (!existsSync(path)) throw new Error(`missing ${path}${hint ? ` (${hint})` : ""}`);
    return path;
}

function buildFixtures() {
    const fixtures = [];
    for (const name of SOURCE_FIXTURES) {
        const base = `/specifications/conformance/t1/${name}`;
        const dir = join(ROOT, base);
        for (const file of ["input.dejavu", "context.ctx.json", "expected.out.txt"]) {
            requireFile(join(dir, file), "run the repository build before the browser smoke");
        }
        fixtures.push({
            name: `t1/${name}`,
            kind: "source",
            sourceUrl: `${base}/input.dejavu`,
            ctxUrl: `${base}/context.ctx.json`,
            expectedUrl: `${base}/expected.out.txt`,
        });
    }
    for (const name of IR_FIXTURES) {
        const base = `/specifications/conformance/values/${name}`;
        const dir = join(ROOT, base);
        for (const file of ["expected.ir.json", "context.ctx.json", "expected.out.txt"]) {
            requireFile(join(dir, file), "run the repository build before the browser smoke");
        }
        fixtures.push({
            name: `values/${name}`,
            kind: "ir",
            irUrl: `${base}/expected.ir.json`,
            ctxUrl: `${base}/context.ctx.json`,
            expectedUrl: `${base}/expected.out.txt`,
        });
    }
    return fixtures;
}

/**
 * `dist/index.js` imports the platform package by its bare name so hosts can
 * resolve it however they like. A document import map does that natively, but
 * a module worker has no import map, so the harness serves this in-memory copy
 * with only that specifier rewritten to the glue module URL. Nothing on disk
 * in `dist/` or `src/` changes — this is the substitute for bundling.
 */
function buildWorkerBridge() {
    const source = readFileSync(requireFile(DIST_ENTRY), "utf8");
    const bare = JSON.stringify(BARE_SPECIFIER);
    if (!source.includes(bare)) throw new Error(`expected a bare import of ${BARE_SPECIFIER} in dist/index.js`);
    return source.replace(bare, JSON.stringify(GLUE_URL));
}

function send(response, contentType, body) {
    response.writeHead(200, { "content-type": contentType, "cache-control": "no-store" });
    response.end(body);
}

async function startServer(fixtures, workerBridge) {
    let resolveResult;
    const resultPromise = new Promise((resolve) => {
        resolveResult = resolve;
    });
    let resultBody = null;
    const pendingGates = [];
    const releaseGates = () => {
        for (const response of pendingGates.splice(0)) response.end(GATE_IMAGE);
    };
    const gateTimer = setTimeout(() => {
        if (resultBody === null) releaseGates();
    }, GATE_TIMEOUT_MS);

    const server = createServer((request, response) => {
        let pathname;
        try {
            pathname = decodeURIComponent(new URL(request.url ?? "/", "http://127.0.0.1").pathname);
        } catch {
            response.writeHead(400).end("bad request");
            return;
        }
        /**
         * The page keeps a parser-inserted image pending so the document `load`
         * event — and therefore `--dump-dom` — waits for the async wasm run.
         */
        if (pathname === "/__browser-gate") {
            response.writeHead(200, { "content-type": "image/gif", "cache-control": "no-store" });
            if (resultBody !== null) response.end(GATE_IMAGE);
            else pendingGates.push(response);
            return;
        }
        if (pathname === "/__browser-done" && request.method === "POST") {
            let body = "";
            request.setEncoding("utf8");
            request.on("data", (chunk) => {
                body += chunk;
            });
            request.on("end", () => {
                resultBody = body;
                response.writeHead(204).end();
                clearTimeout(gateTimer);
                releaseGates();
                resolveResult(body);
            });
            return;
        }
        if (pathname === "/__browser-fixtures.json") {
            send(
                response,
                "application/json; charset=utf-8",
                JSON.stringify({ fixtures, bridgeUrl: BRIDGE_URL, wasmUrl: WASM_URL, workerUrl: WORKER_URL }),
            );
            return;
        }
        if (pathname === "/__browser-bridge.js") {
            send(response, "text/javascript; charset=utf-8", workerBridge);
            return;
        }
        const filePath = join(ROOT, pathname);
        if (filePath !== ROOT && !filePath.startsWith(ROOT + sep)) {
            response.writeHead(403).end("forbidden");
            return;
        }
        if (!existsSync(filePath) || !statSync(filePath).isFile()) {
            response.writeHead(404).end("not found");
            return;
        }
        send(response, MIME.get(extname(filePath)) ?? "application/octet-stream", readFileSync(filePath));
    });
    await new Promise((resolve, reject) => {
        server.once("error", reject);
        server.listen(0, "127.0.0.1", resolve);
    });
    return {
        server,
        resultPromise,
        close() {
            clearTimeout(gateTimer);
            releaseGates();
            return new Promise((resolve) => server.close(resolve));
        },
    };
}

function runBrowser(browser, args) {
    return new Promise((resolve, reject) => {
        const child = spawn(browser, args, { stdio: ["ignore", "pipe", "pipe"] });
        let stdout = "";
        let stderr = "";
        child.stdout.setEncoding("utf8").on("data", (chunk) => {
            stdout += chunk;
        });
        child.stderr.setEncoding("utf8").on("data", (chunk) => {
            stderr += chunk;
        });
        const timer = setTimeout(() => child.kill(), BROWSER_TIMEOUT_MS);
        child.on("error", (error) => {
            clearTimeout(timer);
            reject(error);
        });
        child.on("close", (code) => {
            clearTimeout(timer);
            resolve({ code, stdout, stderr });
        });
    });
}

function findBrowser() {
    const fromEnv = process.env.DEJAVU_BROWSER;
    if (fromEnv) {
        if (!existsSync(fromEnv)) throw new Error(`DEJAVU_BROWSER points at a missing executable: ${fromEnv}`);
        return fromEnv;
    }
    const candidates = [
        "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
        "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
        "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
        "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
    ];
    const found = candidates.find((candidate) => existsSync(candidate));
    if (!found) {
        throw new Error("no Chromium browser found, set DEJAVU_BROWSER to a Chrome/Edge executable");
    }
    return found;
}

function unescapeHtml(text) {
    return text
        .replaceAll("&lt;", "<")
        .replaceAll("&gt;", ">")
        .replaceAll("&quot;", '"')
        .replaceAll("&#39;", "'")
        .replaceAll("&amp;", "&");
}

function extractPayload(dom) {
    const title = /<title>([\s\S]*?)<\/title>/i.exec(dom)?.[1] ?? "";
    if (title.startsWith(RESULT_PREFIX)) return JSON.parse(unescapeHtml(title.slice(RESULT_PREFIX.length)));
    const pre = /<pre id="result">([\s\S]*?)<\/pre>/i.exec(dom)?.[1] ?? "";
    if (pre && !pre.startsWith("pending")) return JSON.parse(unescapeHtml(pre));
    return null;
}

function reportSection(label, section) {
    console.log(`## ${label}`);
    if (section?.error) console.log(`  error: ${section.error}`);
    const results = section?.results ?? [];
    if (results.length === 0) console.log("  no fixture results");
    for (const result of results) {
        console.log(`  ${result.name}: ${result.ok ? "PASS" : "FAIL"}`);
        if (result.ok) continue;
        if (result.error) console.log(`    error: ${result.error}`);
        if (result.diagnostics?.length) console.log(`    diagnostics: ${JSON.stringify(result.diagnostics)}`);
        if (result.expected !== undefined) console.log(`    expected ${JSON.stringify(result.expected)}`);
        if (result.output !== undefined) console.log(`    actual   ${JSON.stringify(result.output)}`);
    }
    console.log("");
    return results.filter((result) => result.ok).length;
}

async function main() {
    const fixtures = buildFixtures();
    requireFile(WASM_ENTRY, "build the wasm platform package before the browser smoke");
    const workerBridge = buildWorkerBridge();
    const browser = findBrowser();

    let session;
    const userDataDir = mkdtempSync(join(tmpdir(), "dejavu-browser-"));
    try {
        session = await startServer(fixtures, workerBridge);
        const pageUrl = `http://127.0.0.1:${session.server.address().port}${PAGE_URL}`;
        const run = await runBrowser(browser, [
            "--headless=new",
            "--disable-gpu",
            "--no-first-run",
            "--no-default-browser-check",
            `--user-data-dir=${userDataDir}`,
            "--dump-dom",
            pageUrl,
        ]);
        const posted = await Promise.race([
            session.resultPromise,
            new Promise((resolve) => setTimeout(() => resolve(null), 2000)),
        ]);
        let payload = null;
        if (posted) payload = JSON.parse(posted);
        if (!payload) payload = extractPayload(run.stdout ?? "");
        if (!payload) {
            console.error(`browser produced no result payload (exit ${run.code})`);
            console.error((run.stderr ?? "").trim().slice(-2000));
            process.exitCode = 1;
            return;
        }

        const mainPass = reportSection("main thread (native import map, initWasm({ url }))", payload.mainThread);
        const workerPass = reportSection(`module worker (bridge mode: ${payload.worker?.mode ?? "unknown"})`, payload.worker);
        const total = fixtures.length;
        console.log(`Browser smoke: main-thread ${mainPass}/${total}, worker ${workerPass}/${total}`);
        if (mainPass !== total || workerPass !== total) process.exitCode = 1;
    } finally {
        if (session) await session.close();
        try {
            rmSync(userDataDir, { recursive: true, force: true });
        } catch (error) {
            console.warn(`could not remove temporary profile ${userDataDir}: ${error.message}`);
        }
    }
}

main().catch((error) => {
    console.error(String((error && error.stack) || error));
    process.exitCode = 1;
});
