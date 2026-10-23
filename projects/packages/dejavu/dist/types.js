/**
 * Public types for the `@doki-land/dejavu` facade.
 *
 * These types describe the author-facing contract documented in
 * `dejavu-typescript-ux.md`. N-API wire types live in `napi.ts`
 * and are translated here into stable camelCase shapes.
 */
// ---------------------------------------------------------------------------
// JIT Template (compile result)
// ---------------------------------------------------------------------------
/**
 * A JIT-prepared template returned by `compile()`.
 *
 * Templates are **process-local** objects; they are NOT serializable,
 * distributable, or safe to pass across Worker boundaries. Each `.render(ctx)`
 * is independent — no context/block state leaks between calls.
 */
export class Template {
    _irJson;
    _defaultOptions;
    _render;
    constructor(irJson, defaultOptions, render) {
        this._irJson = irJson;
        this._defaultOptions = defaultOptions;
        this._render = render;
    }
    /** The compiled IR as a JSON string. Intended for debugging. */
    get irJson() {
        return this._irJson;
    }
    /** Re-render with the stored JIT preparation. Source is NOT re-parsed. */
    render(ctx = {}, options = {}) {
        return this._render(this._irJson, ctx, { ...this._defaultOptions, ...options });
    }
}
