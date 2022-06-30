/**
 * Pure path-v1 template ref resolution (no parse).
 * Contract: specifications/template-contract/v1.md — Loader & Resolution.
 */

import type { TemplateDiagnostic, TemplateDiagnosticCode } from "@dejavu/types";

export type CatalogEntry =
    | string
    | { /* IrDocument duck */ readonly type?: string; [k: string]: unknown };

export type CatalogRoot = {
    name: string;
    priority: number;
    scheme: string;
    files: Map<string, CatalogEntry>;
    basePath?: string;
};

export type ResolveHit = {
    root: CatalogRoot;
    path: string;
};

export type ResolveRefResult =
    | { ok: true; hit: ResolveHit }
    | { ok: false; diagnostic: TemplateDiagnostic };

const SCHEME_RE = /^([A-Za-z][A-Za-z0-9_+-]*):(.*)$/;

export function posixNormalize(path: string): string {
    const parts: string[] = [];
    for (const seg of path.replace(/\\/g, "/").split("/")) {
        if (!seg || seg === ".") continue;
        if (seg === "..") {
            if (parts.length === 0) {
                const err = new Error(`template path escapes root: ${path}`);
                (err as Error & { code: TemplateDiagnosticCode }).code = "relative_escape_root";
                throw err;
            }
            parts.pop();
            continue;
        }
        parts.push(seg);
    }
    return parts.join("/");
}

export function dirnamePosix(path: string): string {
    const i = path.lastIndexOf("/");
    return i < 0 ? "" : path.slice(0, i);
}

export function joinPosix(dir: string, rel: string): string {
    if (!dir) return posixNormalize(rel);
    return posixNormalize(`${dir}/${rel}`);
}

export function parseCanonicalId(
    id: string,
    rootsByName: Map<string, CatalogRoot>,
    rootsByScheme: Map<string, CatalogRoot>,
): ResolveHit | null {
    const m = SCHEME_RE.exec(id);
    if (!m) return null;
    const root = rootsByName.get(m[1]!) ?? rootsByScheme.get(m[1]!);
    if (!root) return null;
    try {
        return { root, path: posixNormalize(m[2]!) };
    } catch {
        return null;
    }
}

export function canonicalId(rootName: string, path: string): string {
    return `${rootName}:${path}`;
}

function diag(
    code: TemplateDiagnosticCode,
    message: string,
    ref: string,
    extra?: Partial<TemplateDiagnostic>,
): TemplateDiagnostic {
    return { code, message, ref, ...extra };
}

function probeRoot(root: CatalogRoot, path: string, extensions: string[]): ResolveHit | null {
    let normalized: string;
    try {
        normalized = posixNormalize(path);
    } catch {
        return null;
    }
    const knownExts = extensions.filter((e) => e.length > 0);
    const hasKnownExt = knownExts.some((e) => normalized.endsWith(e));
    const candidates = hasKnownExt ? [normalized] : extensions.map((ext) => `${normalized}${ext}`);
    for (const key of candidates) {
        if (root.files.has(key)) return { root, path: key };
    }
    return null;
}

/**
 * Pure resolve: catalog + ref + optional canonical `from` → hit or diagnostic.
 */
export function resolveTemplateRef(
    roots: CatalogRoot[],
    rootsByName: Map<string, CatalogRoot>,
    rootsByScheme: Map<string, CatalogRoot>,
    extensions: string[],
    ref: string,
    from?: string,
): ResolveRefResult {
    const raw = ref.replace(/\\/g, "/").trim();
    if (!raw) {
        return {
            ok: false,
            diagnostic: diag("invalid_ref", "template ref must be a non-empty string", ref),
        };
    }

    const schemeMatch = SCHEME_RE.exec(raw);
    if (schemeMatch) {
        const scheme = schemeMatch[1]!;
        const path = schemeMatch[2]!;
        const root = rootsByScheme.get(scheme);
        if (!root) {
            return {
                ok: false,
                diagnostic: diag("unknown_scheme", `unknown template scheme: ${scheme}`, ref),
            };
        }
        const hit = probeRoot(root, path, extensions);
        if (!hit) {
            return {
                ok: false,
                diagnostic: diag("template_not_found", `template not found: ${ref}`, ref, {
                    searched: [root.name],
                }),
            };
        }
        return { ok: true, hit };
    }

    if (raw.startsWith("./") || raw.startsWith("../")) {
        if (!from) {
            return {
                ok: false,
                diagnostic: diag(
                    "relative_without_from",
                    `relative template ref '${ref}' requires a referring template (from)`,
                    ref,
                ),
            };
        }
        const fromHit = parseCanonicalId(from, rootsByName, rootsByScheme);
        if (!fromHit) {
            return {
                ok: false,
                diagnostic: diag(
                    "invalid_ref",
                    `relative template ref '${ref}' cannot resolve against non-canonical from '${from}'`,
                    ref,
                    { from },
                ),
            };
        }
        let joined: string;
        try {
            joined = joinPosix(dirnamePosix(fromHit.path), raw);
        } catch {
            return {
                ok: false,
                diagnostic: diag(
                    "relative_escape_root",
                    `template path escapes root: ${ref}`,
                    ref,
                    { from },
                ),
            };
        }
        const hit = probeRoot(fromHit.root, joined, extensions);
        if (!hit) {
            return {
                ok: false,
                diagnostic: diag(
                    "template_not_found",
                    `template not found: ${ref} (from ${from})`,
                    ref,
                    { from, searched: [fromHit.root.name] },
                ),
            };
        }
        return { ok: true, hit };
    }

    const ranked = [...roots].sort((a, b) => b.priority - a.priority);
    const searched: string[] = [];
    for (const root of ranked) {
        searched.push(root.name);
        const hit = probeRoot(root, raw, extensions);
        if (hit) return { ok: true, hit };
    }
    const fromHint = from ? ` (from ${from})` : "";
    return {
        ok: false,
        diagnostic: diag("template_not_found", `template not found: ${ref}${fromHint}`, ref, {
            from,
            searched,
        }),
    };
}
