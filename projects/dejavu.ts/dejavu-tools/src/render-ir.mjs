#!/usr/bin/env node
/**
 * Tiny ESM bridge used by scripts/conformance-runner.mjs
 * Args: <ir.json> <ctx.json>
 */
import {readFileSync} from "node:fs";
import {pathToFileURL} from "node:url";
import {join, dirname} from "node:path";
import {fileURLToPath} from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const engineUrl = pathToFileURL(join(__dirname, "../../dejavu-engine/src/index.ts")).href;

const [{renderIr}] = await Promise.all([
    import(engineUrl).catch(async () => {
        // Node strip-types / tsx fallbacks handled by caller; try relative built path
        return import(pathToFileURL(join(__dirname, "../../dejavu-engine/src/index.ts")).href);
    }),
]);

const ir = JSON.parse(readFileSync(process.argv[2], "utf8"));
const ctx = JSON.parse(readFileSync(process.argv[3], "utf8"));
process.stdout.write(renderIr(ir, ctx));
