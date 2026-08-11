/**
 * @deprecated Legacy AST engine. Isolated from the formal package surface:
 * - not in `tsconfig.json` include (use `pnpm typecheck:legacy` only)
 * - not exported from `@dejavu/engine` package.json
 *
 * New hosts must use the IR path (`@dejavu/engine` / `@doki-land/dejavu`).
 * Kept as in-tree reference until Template Contract 1.0 freezes and conformance is green.
 */
export { DejavuEngine, engine } from "./engine";
export {
    registerTemplate,
    renderTemplate,
    checkTemplate,
} from "./template";
export { DejavuRenderer, renderer } from "./renderer";
export type { RenderContext } from "./renderer";
export { FilterRegistry, filterRegistry } from "./filter";
export type { Filter } from "./filter";
export * from "./ast";
export * from "./token";
export * from "./lexer";
export * from "./parser";
export * from "./language";
export * from "./error-types";
