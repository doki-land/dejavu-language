/**
 * Host-supplied `filter` / `function` / `test` implementations for AOT output.
 *
 * Keys are canonical names only: `dejavu-codegen` resolves aliases at compile
 * time and emits the canonical spelling, so a registry never needs to know the
 * alias table. Implementations are synchronous and supplied per `render` call
 * (`render(ctx, { custom })`), keeping the runtime free of module-level state.
 */

export type CustomFilter = (value: unknown, args: unknown[]) => unknown;
export type CustomFunction = (args: unknown[]) => unknown;
export type CustomTest = (value: unknown, args: unknown[]) => boolean;

export interface CustomRegistry {
    filter?: Record<string, CustomFilter>;
    function?: Record<string, CustomFunction>;
    test?: Record<string, CustomTest>;
}
