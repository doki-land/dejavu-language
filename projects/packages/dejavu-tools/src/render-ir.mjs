#!/usr/bin/env node
/**
 * Tiny ESM bridge used by scripts/conformance-runner.mjs
 *
 * B3b 之后 JIT renderIr 已从 @dejavu/engine 移除。
 * 此脚本保留为占位符，但始终报错 —— 请用 dejavu-codegen AOT 输出替代 JIT 渲染。
 */
console.error(
    "render-ir.mjs: JIT renderIr was removed from @dejavu/engine in B3b. Use dejavu-codegen AOT output instead.",
);
process.exit(1);
