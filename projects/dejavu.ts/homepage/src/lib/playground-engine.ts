/**
 * Playground engine: loads the Dejavu WASM bridge artifact once, then renders
 * template source against a JSON context entirely in the browser.
 *
 * The artifact ships in `@doki-land/dejavu-unknown-wasm32`; `scripts/sync-wasm.mjs`
 * stages the glue module and the `.wasm` binary into `public/vendor/dejavu/`.
 * The wire payloads mirror the `dejavu.wasm/0` bridge contract.
 */

const GLUE_URL = "/vendor/dejavu/dejavu_wasm.js";
const WASM_URL = "/vendor/dejavu/dejavu_wasm_bg.wasm";
const BINDING_CONTRACT = "dejavu.wasm/0";

/** Diagnostic severity. */
export type Severity = "error" | "warning" | "info" | "debug";

/** Language-neutral diagnostic wire, mirrored from the bridge contract. */
export interface Diagnostic {
    code: string;
    severity: Severity;
    stage: string;
    message: string;
}

/** Render result wire. */
export interface RenderResult {
    output: string | undefined;
    diagnostics: Diagnostic[] | undefined;
}

interface WasmGlue {
    default: (input: { module_or_path: string }) => Promise<unknown>;
    bindingContractVersion: () => string;
    renderSource: (requestJson: string) => string;
}

let engineReady: Promise<WasmGlue> | null = null;

/** Instantiate the bridge exactly once; concurrent callers share the load. */
export function ensureEngine(): Promise<WasmGlue> {
    if (!engineReady) {
        engineReady = (async () => {
            const glue = (await import(GLUE_URL)) as unknown as WasmGlue;
            await glue.default({ module_or_path: WASM_URL });
            const contract = glue.bindingContractVersion();
            if (contract !== BINDING_CONTRACT) {
                throw new Error(`dejavu-wasm contract mismatch: expected ${BINDING_CONTRACT}, loaded ${contract}`);
            }
            return glue;
        })();
    }
    return engineReady;
}

export interface PlaygroundRender {
    output: string;
    diagnostics: Diagnostic[];
}

/** Render source with a parsed JSON context; the bridge never sees raw text. */
export function renderTemplate(glue: WasmGlue, source: string, context: unknown): PlaygroundRender {
    const raw = glue.renderSource(JSON.stringify({ source, context: JSON.stringify(context) }));
    const result = JSON.parse(raw) as RenderResult;
    return {
        output: result.output ?? "",
        diagnostics: result.diagnostics ?? [],
    };
}
