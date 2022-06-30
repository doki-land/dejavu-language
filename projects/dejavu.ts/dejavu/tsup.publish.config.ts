import { defineConfig } from "tsup";

/** Bundle @dejavu/* into dist for npm publish (no workspace: deps). */
export default defineConfig({
    entry: ["src/index.ts"],
    format: ["esm"],
    dts: true,
    outDir: "dist",
    clean: true,
    target: "node20",
    tsconfig: "tsconfig.json",
    noExternal: [/@dejavu\//],
});
