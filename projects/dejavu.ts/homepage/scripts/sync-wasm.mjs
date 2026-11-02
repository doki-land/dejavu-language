/**
 * Copies the Dejavu WASM bridge artifact into `public/` so the playground can
 * fetch it as static assets. The artifact itself stays owned by the published
 * `@doki-land/dejavu-unknown-wasm32` package; this script only stages it.
 */
import { createRequire } from "node:module";
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const require = createRequire(join(root, "package.json"));

const glueEntry = require.resolve("@doki-land/dejavu-unknown-wasm32");
const glueDir = dirname(glueEntry);
const targetDir = join(root, "public", "vendor", "dejavu");

mkdirSync(targetDir, { recursive: true });
for (const file of ["dejavu_wasm.js", "dejavu_wasm_bg.wasm"]) {
    copyFileSync(join(glueDir, file), join(targetDir, file));
    console.log(`staged ${join(targetDir, file)}`);
}
