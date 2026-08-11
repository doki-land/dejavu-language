import {
    applyFilter,
    htmlEscape,
    isSafeHtml,
    valueToString,
    type CanonicalId,
    type IrDocument,
    type IrNode,
    type IrValue,
    type RenderOptions,
    type TemplateResolveOk,
    type TemplateLoader,
} from "@dejavu/types";

function loadTemplate(
    loader: TemplateLoader,
    ref: string,
    from: CanonicalId,
    onDependency?: (id: CanonicalId) => void,
): TemplateResolveOk {
    const loaded = loader.resolve(ref, { from });
    onDependency?.(loaded.id);
    return loaded;
}

function noteDependency(env: RenderEnv, id: CanonicalId): void {
    env.onDependency?.(id);
}

type BlockNode = Extract<IrNode, { type: "Stmt.Block" }> & {
    __parentDefault?: IrNode[];
};

function truthy(v: IrValue): boolean {
    if (v === null) return false;
    if (typeof v === "boolean") return v;
    if (typeof v === "number") return v !== 0;
    if (typeof v === "string") return v.length > 0;
    if (Array.isArray(v)) return v.length > 0;
    if (typeof v === "object") return Object.keys(v).length > 0;
    return false;
}

/** Render IR with a JSON object context. Optional loader enables extends/include/super. */
export function renderIr(
    doc: IrDocument,
    ctx: Record<string, IrValue> = {},
    options: RenderOptions = {},
): string {
    const scope = { ...ctx };
    let name: CanonicalId = options.name ?? "<main>";
    // Canonicalize entry name when possible so relative includes get a valid `from`.
    if (options.loader && name !== "<main>" && !/^[A-Za-z][A-Za-z0-9_+-]*:/.test(name)) {
        try {
            name = options.loader.resolve(name).id;
        } catch {
            /* keep caller name for T1 / ad-hoc docs */
        }
    }
    const env: RenderEnv = {
        loader: options.loader,
        stack: [name],
        parentBlocks: [],
        strictUndefined: options.strictUndefined === true,
        onDependency: options.onDependency,
    };
    if (name !== "<main>") {
        noteDependency(env, name);
    }
    const resolved = resolveInheritance(doc, scope, env, name, new Set());
    return renderNode(resolved.body, scope, env);
}

type RenderEnv = {
    loader?: TemplateLoader;
    stack: CanonicalId[];
    parentBlocks: IrNode[][];
    strictUndefined: boolean;
    onDependency?: (id: CanonicalId) => void;
};

function templateChildren(doc: IrDocument): IrNode[] {
    if (doc.body.type !== "Template") {
        throw new Error("IrDocument.body must be Template");
    }
    return doc.body.children;
}

function collectBlocks(nodes: IrNode[]): Map<string, IrNode[]> {
    const map = new Map<string, IrNode[]>();
    for (const n of nodes) {
        if (n.type === "Stmt.Block") map.set(n.name, n.body);
    }
    return map;
}

function findExtends(nodes: IrNode[]): Extract<IrNode, { type: "Stmt.Extends" }> | null {
    for (const n of nodes) {
        if (n.type === "Stmt.Extends") return n;
    }
    return null;
}

function requireStringPath(v: IrValue, kind: string): string {
    if (typeof v !== "string" || v.length === 0) {
        throw new Error(`${kind} path must evaluate to a non-empty string`);
    }
    return v;
}

function resolveInheritance(
    doc: IrDocument,
    scope: Record<string, IrValue>,
    env: RenderEnv,
    name: string,
    visiting: Set<string>,
): IrDocument {
    const children = templateChildren(doc);
    const ext = findExtends(children);
    if (!ext) return doc;
    if (!env.loader) {
        throw new Error(
            "extends/include/super require a template loader (not in T1 minimal render)",
        );
    }
    if (visiting.has(name)) {
        throw new Error(`circular template inheritance involving '${name}'`);
    }
    visiting.add(name);

    const parentRef = requireStringPath(evalExpr(ext.parent, scope, env), "extends");
    const parent = loadTemplate(env.loader, parentRef, name, env.onDependency);
    if (visiting.has(parent.id)) {
        throw new Error(`circular template inheritance: ${[...visiting, parent.id].join(" -> ")}`);
    }
    const parentResolved = resolveInheritance(parent.document, scope, env, parent.id, visiting);
    visiting.delete(name);

    const childBlocks = collectBlocks(children);
    const mergedChildren = applyBlocks(templateChildren(parentResolved), childBlocks);
    return {
        ...parentResolved,
        body: { type: "Template", children: mergedChildren },
    };
}

function applyBlocks(nodes: IrNode[], overrides: Map<string, IrNode[]>): IrNode[] {
    return nodes.map((n) => {
        if (n.type === "Stmt.Block") {
            const override = overrides.get(n.name);
            if (!override) {
                return { ...n, body: applyBlocks(n.body, overrides) };
            }
            const parentDefault = applyBlocks(n.body, overrides);
            const merged: BlockNode = {
                type: "Stmt.Block",
                name: n.name,
                trim: n.trim,
                body: override,
                __parentDefault: parentDefault,
            };
            return merged;
        }
        if (n.type === "Template") {
            return { ...n, children: applyBlocks(n.children, overrides) };
        }
        if (n.type === "Stmt.If") {
            return {
                ...n,
                consequent: applyBlocks(n.consequent, overrides),
                elseIfs: n.elseIfs.map((ei) =>
                    ei.type === "Stmt.ElseIf"
                        ? { ...ei, consequent: applyBlocks(ei.consequent, overrides) }
                        : ei,
                ),
                ...(n.alternate ? { alternate: applyBlocks(n.alternate, overrides) } : {}),
            };
        }
        if (n.type === "Stmt.For") {
            return { ...n, body: applyBlocks(n.body, overrides) };
        }
        return n;
    });
}

function renderNode(node: IrNode, scope: Record<string, IrValue>, env: RenderEnv): string {
    switch (node.type) {
        case "Template":
            return node.children.map((c) => renderNode(c, scope, env)).join("");
        case "Text":
            return node.value;
        case "Comment":
            return "";
        case "Interpolation": {
            const v = evalExpr(node.expression, scope, env);
            const s = valueToString(v);
            if (node.raw || isSafeHtml(v)) return s;
            return htmlEscape(s);
        }
        case "Stmt.If": {
            if (truthy(evalExpr(node.test, scope, env))) {
                return node.consequent.map((c) => renderNode(c, scope, env)).join("");
            }
            for (const ei of node.elseIfs) {
                if (ei.type === "Stmt.ElseIf" && truthy(evalExpr(ei.test, scope, env))) {
                    return ei.consequent.map((c) => renderNode(c, scope, env)).join("");
                }
            }
            if (node.alternate) {
                return node.alternate.map((c) => renderNode(c, scope, env)).join("");
            }
            return "";
        }
        case "Stmt.For": {
            const iter = evalExpr(node.iterable, scope, env);
            if (!Array.isArray(iter)) {
                throw new Error("for iterable must be array");
            }
            let out = "";
            iter.forEach((val, i) => {
                const prev = scope[node.item];
                scope[node.item] = val;
                let prevIdx: IrValue | undefined;
                if (node.index) {
                    prevIdx = scope[node.index];
                    scope[node.index] = i;
                }
                out += node.body.map((c) => renderNode(c, scope, env)).join("");
                if (prev === undefined) delete scope[node.item];
                else scope[node.item] = prev;
                if (node.index) {
                    if (prevIdx === undefined) delete scope[node.index];
                    else scope[node.index] = prevIdx;
                }
            });
            return out;
        }
        case "Stmt.Raw":
            return node.value;
        case "Stmt.Block": {
            const block = node as BlockNode;
            const nextEnv: RenderEnv = block.__parentDefault
                ? { ...env, parentBlocks: [...env.parentBlocks, block.__parentDefault] }
                : env;
            return block.body.map((c) => renderNode(c, scope, nextEnv)).join("");
        }
        case "Stmt.Super": {
            const parent = env.parentBlocks[env.parentBlocks.length - 1];
            if (!parent) {
                throw new Error("`super` used outside an overriding block");
            }
            const nextEnv: RenderEnv = {
                ...env,
                parentBlocks: env.parentBlocks.slice(0, -1),
            };
            return parent.map((c) => renderNode(c, scope, nextEnv)).join("");
        }
        case "Stmt.Extends":
            return "";
        case "Stmt.Include": {
            if (!env.loader) {
                throw new Error(
                    "extends/include/super require a template loader (not in T1 minimal render)",
                );
            }
            const includeRef = requireStringPath(evalExpr(node.path, scope, env), "include");
            const from = env.stack[env.stack.length - 1] ?? "<main>";
            const included = loadTemplate(env.loader, includeRef, from, env.onDependency);
            if (env.stack.includes(included.id)) {
                throw new Error(
                    `circular template include: ${[...env.stack, included.id].join(" -> ")}`,
                );
            }
            const resolved = resolveInheritance(
                included.document,
                scope,
                env,
                included.id,
                new Set(env.stack),
            );
            return renderNode(resolved.body, scope, {
                ...env,
                stack: [...env.stack, included.id],
            });
        }
        default:
            throw new Error(`node not renderable: ${(node as IrNode).type}`);
    }
}

function evalExpr(expr: IrNode, scope: Record<string, IrValue>, env: RenderEnv): IrValue {
    switch (expr.type) {
        case "Expr.Literal":
            return expr.value;
        case "Expr.Identifier": {
            if (!(expr.name in scope)) {
                if (env.strictUndefined) {
                    throw new Error(`undefined variable \`${expr.name}\``);
                }
                return null;
            }
            return scope[expr.name] ?? null;
        }
        case "Expr.Member": {
            const obj = evalExpr(expr.object, scope, env);
            if (obj && typeof obj === "object" && !Array.isArray(obj) && !isSafeHtml(obj)) {
                return (obj as Record<string, IrValue>)[expr.property] ?? null;
            }
            return null;
        }
        case "Expr.Index": {
            const obj = evalExpr(expr.object, scope, env);
            const idx = evalExpr(expr.index, scope, env);
            if (Array.isArray(obj) && typeof idx === "number") {
                return obj[idx] ?? null;
            }
            if (obj && typeof obj === "object" && !isSafeHtml(obj) && typeof idx === "string") {
                return (obj as Record<string, IrValue>)[idx] ?? null;
            }
            return null;
        }
        case "Expr.Binary": {
            const l = evalExpr(expr.left, scope, env);
            const r = evalExpr(expr.right, scope, env);
            return evalBinary(expr.operator, l, r);
        }
        case "Expr.Unary": {
            const v = evalExpr(expr.argument, scope, env);
            if (expr.operator === "!") return !truthy(v);
            if (expr.operator === "-" && typeof v === "number") return -v;
            if (expr.operator === "+") return v;
            return null;
        }
        case "Expr.Pipe": {
            let val = evalExpr(expr.expression, scope, env);
            const args = expr.arguments.map((a) => evalExpr(a, scope, env));
            val = applyFilter(expr.filter, val, args);
            return val;
        }
        case "Expr.Call":
            throw new Error("calls not supported in T1 eval");
        default:
            throw new Error(`invalid expression: ${(expr as IrNode).type}`);
    }
}

function evalBinary(op: string, l: IrValue, r: IrValue): IrValue {
    switch (op) {
        case "+":
            if (typeof l === "number" && typeof r === "number") return l + r;
            return valueToString(l) + valueToString(r);
        case "-":
            return typeof l === "number" && typeof r === "number" ? l - r : null;
        case "*":
            return typeof l === "number" && typeof r === "number" ? l * r : null;
        case "/":
            return typeof l === "number" && typeof r === "number" ? l / r : null;
        case "%":
            return typeof l === "number" && typeof r === "number" ? l % r : null;
        case "==":
            return l === r;
        case "!=":
            return l !== r;
        case "<":
            return typeof l === "number" && typeof r === "number" ? l < r : false;
        case "<=":
            return typeof l === "number" && typeof r === "number" ? l <= r : false;
        case ">":
            return typeof l === "number" && typeof r === "number" ? l > r : false;
        case ">=":
            return typeof l === "number" && typeof r === "number" ? l >= r : false;
        case "&&":
            return truthy(l) && truthy(r);
        case "||":
            return truthy(l) || truthy(r);
        case "in":
            if (Array.isArray(r)) return r.includes(l);
            if (typeof r === "string") return r.includes(valueToString(l));
            return false;
        default:
            throw new Error(`unknown binary ${op}`);
    }
}
