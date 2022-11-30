import type {IrValue} from "./ir";

/** Normalize IR JSON for semantic equality (see specifications/ir/v1/normalize.md). */
export function normalizeValue(value: unknown): unknown {
    if (Array.isArray(value)) {
        return value.map(normalizeValue).filter((v) => v !== null);
    }
    if (value && typeof value === "object") {
        const obj = value as Record<string, unknown>;
        const out: Record<string, unknown> = {};
        for (const key of Object.keys(obj).sort()) {
            if (key === "span") continue;
            if (key === "raw" && obj[key] === false) continue;
            const nv = normalizeValue(obj[key]);
            out[key] = nv;
        }
        if (out.type === "Text" && out.value === "") {
            return null;
        }
        return out;
    }
    return value as IrValue;
}

export function normalizeIrJson(json: string): unknown {
    return normalizeValue(JSON.parse(json));
}
