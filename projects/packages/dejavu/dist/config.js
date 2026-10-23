/**
 * Authoring surface for `dejavu.config.ts`.
 *
 * This subpath is intentionally dependency-free: it must never import the
 * N-API bridge, the CLI, or `node:fs`, so `import { defineConfig } from
 * "@doki-land/dejavu/config"` stays a pure type-level import.
 *
 * `defineConfig` only narrows the authoring type. Runtime validation happens
 * once in the config loading pipeline (`config-load.ts`) and cannot be
 * bypassed by using this helper.
 */
/**
 * Type helper for `dejavu.config.ts`. Returns the same object, unchanged.
 *
 * ```ts
 * import { defineConfig } from "@doki-land/dejavu/config";
 * export default defineConfig({ roots: [{ name: "app", dir: "./templates" }] });
 * ```
 */
export function defineConfig(config) {
    return config;
}
