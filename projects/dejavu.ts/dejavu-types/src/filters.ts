import type {IrValue} from "./ir";

export type FilterFn = (value: IrValue, args: IrValue[]) => IrValue;

/** Sentinel object: interpolation must not HTML-escape this payload. */
export const SAFE_HTML_KEY = "__dejavuSafeHtml";

export type SafeHtmlValue = {
    [SAFE_HTML_KEY]: true;
    html: string;
};

export function markSafe(html: string): SafeHtmlValue {
    return {[SAFE_HTML_KEY]: true, html};
}

export function isSafeHtml(v: IrValue): v is SafeHtmlValue {
    return (
        !!v &&
        typeof v === "object" &&
        !Array.isArray(v) &&
        (v as SafeHtmlValue)[SAFE_HTML_KEY] === true &&
        typeof (v as SafeHtmlValue).html === "string"
    );
}

export function htmlEscape(s: string): string {
    return s
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#39;");
}

export function valueToString(v: IrValue): string {
    if (v === null || v === undefined) return "";
    if (isSafeHtml(v)) return v.html;
    if (typeof v === "string") return v;
    if (typeof v === "boolean" || typeof v === "number") return String(v);
    return JSON.stringify(v);
}

function unwrapString(v: IrValue): string {
    return valueToString(v);
}

/** Minimal strftime for Contract filters: %Y %m %d %H %M %S %% */
export function formatDate(input: IrValue, pattern: string): string {
    const d = toDate(input);
    if (!d) return "";
    const pad = (n: number) => String(n).padStart(2, "0");
    return pattern.replace(/%[%YmdHMS]/g, (tok) => {
        switch (tok) {
            case "%%":
                return "%";
            case "%Y":
                return String(d.getFullYear());
            case "%m":
                return pad(d.getMonth() + 1);
            case "%d":
                return pad(d.getDate());
            case "%H":
                return pad(d.getHours());
            case "%M":
                return pad(d.getMinutes());
            case "%S":
                return pad(d.getSeconds());
            default:
                return tok;
        }
    });
}

function toDate(input: IrValue): Date | null {
    if (input === null || input === undefined || input === "") return null;
    if (typeof input === "number") {
        const d = new Date(input);
        return Number.isNaN(d.getTime()) ? null : d;
    }
    if (typeof input === "string") {
        if (input === "now") return new Date();
        const d = new Date(input);
        return Number.isNaN(d.getTime()) ? null : d;
    }
    return null;
}

function slugify(s: string): string {
    return s
        .normalize("NFKD")
        .replace(/[\u0300-\u036f]/g, "")
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/^-+|-+$/g, "");
}

export const builtinFilters: Record<string, FilterFn> = {
    uppercase: (v) => unwrapString(v).toUpperCase(),
    lowercase: (v) => unwrapString(v).toLowerCase(),
    upper: (v) => unwrapString(v).toUpperCase(),
    lower: (v) => unwrapString(v).toLowerCase(),
    trim: (v) => unwrapString(v).trim(),
    default: (v, args) => (v === null || v === "" ? (args[0] ?? null) : v),
    length: (v) => {
        if (typeof v === "string") return [...v].length;
        if (Array.isArray(v)) return v.length;
        if (v && typeof v === "object" && !isSafeHtml(v)) return Object.keys(v).length;
        return 0;
    },
    join: (v, args) => {
        const sep = args[0] != null ? unwrapString(args[0]) : ",";
        if (Array.isArray(v)) return v.map(unwrapString).join(sep);
        return unwrapString(v);
    },
    replace: (v, args) => {
        const from = unwrapString(args[0] ?? "");
        const to = unwrapString(args[1] ?? "");
        return unwrapString(v).split(from).join(to);
    },
    escape: (v) => markSafe(htmlEscape(unwrapString(v))),
    e: (v) => markSafe(htmlEscape(unwrapString(v))),
    /** Mark string as trusted HTML (skip auto-escape on interpolate). */
    safe: (v) => markSafe(unwrapString(v)),
    raw: (v) => markSafe(unwrapString(v)),
    date: (v, args) => {
        const pattern = args[0] != null ? unwrapString(args[0]) : "%Y-%m-%d";
        return formatDate(v, pattern);
    },
    slug: (v) => slugify(unwrapString(v)),
};

export function applyFilter(name: string, value: IrValue, args: IrValue[]): IrValue {
    const fn = builtinFilters[name];
    if (!fn) throw new Error(`unknown filter \`${name}\``);
    return fn(value, args);
}
