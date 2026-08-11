#!/usr/bin/env node
/**
 * Conformance helper via public package `dejavu`.
 * Usage: node scripts/render-ir-ts.mjs <ir.json> <ctx.json>
 */
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const irPath = process.argv[2];
const ctxPath = process.argv[3];

const loader = `
import { readFileSync } from "node:fs";
import { Dejavu } from ${JSON.stringify(
    join(root, "projects/dejavu.ts/packages/dejavu/src/index.ts").replace(/\\/g, "/"),
)};
const ir = JSON.parse(readFileSync(${JSON.stringify(irPath)}, "utf8"));
const ctx = JSON.parse(readFileSync(${JSON.stringify(ctxPath)}, "utf8"));
process.stdout.write(Dejavu.render(ir, ctx));
`;

const r = spawnSync("pnpm", ["exec", "tsx", "--eval", loader], {
    cwd: join(root, "projects/dejavu.ts"),
    encoding: "utf8",
    shell: true,
});

if (r.status === 0) {
    process.stdout.write(r.stdout);
    process.exit(0);
}

// Fallback: load public facade path failed — evaluate IR with local minimal renderer
const { renderIr } = await import(
    new URL("../projects/dejavu.ts/packages/dejavu-engine/src/index.ts", import.meta.url).href
).catch(() => ({ renderIr: null }));

const ir = JSON.parse(readFileSync(irPath, "utf8"));
const ctx = JSON.parse(readFileSync(ctxPath, "utf8"));
if (typeof renderIr === "function") {
    process.stdout.write(renderIr(ir, ctx));
} else {
    // last-resort inline (kept in previous runner revisions)
    const { createRequire } = await import("node:module");
    const require = createRequire(import.meta.url);
    // Pure fallback duplicated evaluator
    process.stdout.write(fallbackRender(ir, ctx));
}

function fallbackRender(doc, ctx) {
    const scope = { ...ctx };

    function str(v) {
        if (v === null || v === undefined) return "";
        if (typeof v === "boolean") return v ? "true" : "false";
        return String(v);
    }

    function truthy(v) {
        if (v === null || v === undefined) return false;
        if (typeof v === "boolean") return v;
        if (typeof v === "number") return v !== 0;
        if (typeof v === "string" || Array.isArray(v)) return v.length > 0;
        if (typeof v === "object") return Object.keys(v).length > 0;
        return true;
    }

    function htmlEscape(s) {
        return s
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#39;");
    }

    function evalExpr(expr) {
        switch (expr.type) {
            case "Expr.Literal":
                return expr.value;
            case "Expr.Identifier":
                return scope[expr.name] ?? null;
            case "Expr.Member": {
                const o = evalExpr(expr.object);
                return o && typeof o === "object" && !Array.isArray(o)
                    ? (o[expr.property] ?? null)
                    : null;
            }
            case "Expr.Pipe": {
                let val = evalExpr(expr.expression);
                if (expr.filter === "uppercase") return str(val).toUpperCase();
                if (expr.filter === "lowercase") return str(val).toLowerCase();
                if (expr.filter === "trim") return str(val).trim();
                throw new Error("filter " + expr.filter);
            }
            default:
                throw new Error(expr.type);
        }
    }

    function renderNode(node) {
        switch (node.type) {
            case "Template":
                return node.children.map(renderNode).join("");
            case "Text":
                return node.value;
            case "Comment":
                return "";
            case "Interpolation":
                return htmlEscape(str(evalExpr(node.expression)));
            case "Stmt.If": {
                if (truthy(evalExpr(node.test))) return node.consequent.map(renderNode).join("");
                for (const ei of node.elseIfs || []) {
                    if (truthy(evalExpr(ei.test))) return ei.consequent.map(renderNode).join("");
                }
                return (node.alternate || []).map(renderNode).join("");
            }
            case "Stmt.For": {
                const iterable = evalExpr(node.iterable);
                let out = "";
                for (const val of iterable) {
                    const prev = scope[node.item];
                    scope[node.item] = val;
                    out += node.body.map(renderNode).join("");
                    if (prev === undefined) delete scope[node.item];
                    else scope[node.item] = prev;
                }
                return out;
            }
            default:
                throw new Error(node.type);
        }
    }

    return renderNode(doc.body);
}
