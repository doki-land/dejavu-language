/**
 * Canonical template ids used for diagnostics and relative references.
 */

import { basename, relative, resolve, sep } from "node:path";

/**
 * `<root name>:<path relative to root>` for files inside the root, otherwise the
 * bare basename. Passing no root keeps the caller's own path untouched.
 */
export function templateId(file: string, root?: string): string {
    if (!root) return file;
    const rel = relative(resolve(root), resolve(file)).split(sep).join("/");
    return rel.startsWith("..") ? basename(file) : `root:${rel}`;
}
