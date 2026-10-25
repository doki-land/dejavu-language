/**
 * Host-supplied `filter` / `function` / `test` implementations for AOT output.
 *
 * Keys are canonical names only: `dejavu-codegen` resolves aliases at compile
 * time and emits the canonical spelling, so a registry never needs to know the
 * alias table. Implementations are synchronous and supplied per `render` call
 * (`render(ctx, { custom })`), keeping the runtime free of module-level state.
 */

import { runtimeError } from "./errors.js";

export type CustomFilter = (value: unknown, args: unknown[]) => unknown;
export type CustomFunction = (args: unknown[]) => unknown;
export type CustomTest = (value: unknown, args: unknown[]) => boolean;

export interface CustomRegistry {
    filter?: Record<string, CustomFilter>;
    function?: Record<string, CustomFunction>;
    test?: Record<string, CustomTest>;
}

/** Namespace a host-supplied implementation belongs to. */
export type CustomNamespace = "filter" | "function" | "test";

/**
 * Run one host implementation, normalising a thrown value into the shared
 * `dejavu.template.custom-invocation` code. Mirrors the Rust JIT, where the
 * host hands back `CustomError` instead of unwinding the render.
 */
export function invokeCustom<T>(
    namespace: CustomNamespace,
    name: string,
    call: () => T,
): T {
    try {
        return call();
    } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        throw runtimeError(
            "dejavu.template.custom-invocation",
            `${namespace} ${name}: ${message}`,
            { namespace, name, message },
        );
    }
}
