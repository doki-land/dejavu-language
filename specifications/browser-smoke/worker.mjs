/**
 * Module worker for the browser smoke harness.
 *
 * Import maps live on the document realm, so a module worker may not resolve
 * the bare `@doki-land/dejavu-unknown-wasm32` import inside the host wrapper.
 * The worker measures that first: it imports the unmodified `dist/index.js`
 * directly, and only when resolution fails falls back to the harness route
 * `/__browser-bridge.js`, where the static server rewrites that single bare
 * specifier to the glue module URL in memory. Neither `dist/` nor `src/` is
 * modified on disk — the rewrite is a harness-only substitute for bundling.
 */
import { runFixtures } from "./runner.mjs";

const DIST_BRIDGE_URL = "/projects/dejavu.ts/packages/dejavu-language/dist/index.js";
const REWRITTEN_BRIDGE_URL = "/__browser-bridge.js";

let api;
let mode = "native";

async function loadBridge(wasmUrl) {
    if (api) return;
    try {
        api = await import(DIST_BRIDGE_URL);
    } catch {
        mode = "rewritten";
        api = await import(REWRITTEN_BRIDGE_URL);
    }
    await api.initWasm({ url: wasmUrl });
}

self.onmessage = async (event) => {
    const message = event.data;
    if (!message || message.type !== "run") return;
    try {
        await loadBridge(message.wasmUrl);
        const results = await runFixtures(message.fixtures, api);
        self.postMessage({ type: "done", mode, results });
    } catch (error) {
        self.postMessage({ type: "error", mode, error: String((error && error.stack) || error) });
    }
};
