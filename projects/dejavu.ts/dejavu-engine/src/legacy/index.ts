/**
 * @deprecated Legacy AST engine. New hosts must use the IR path (`@dejavu/engine` / `dejavu`).
 * Kept temporarily for reference and emergency fallback; not used by Doki.
 * Will be removed after Template Contract 1.0 freezes and conformance is green.
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
