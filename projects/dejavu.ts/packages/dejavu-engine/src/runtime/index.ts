/**
 * Dejavu AOT runtime helpers —— dejavu-codegen 产出的 ESM 模块会 import 本文件。
 *
 * 所有导出都是纯函数，无副作用、无状态。
 * 与 Rust `dejavu-codegen` Node stub 以及 `dejavu-types` 的 filter 系统语义一致。
 */

export { htmlEscape } from "./escape.js";
export { valueToString, isTruthy, valuesEqual } from "./value.js";
export { applyFilter } from "./filter.js";
export { FUNCTIONS, functionSpec, callFunction } from "./function.js";
export { TESTS, testSpec, applyTest } from "./test.js";
export { binary, unary, exprMember, exprIndex } from "./expr.js";
