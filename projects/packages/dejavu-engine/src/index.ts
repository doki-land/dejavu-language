/**
 * @dejavu/engine —— 纯 TypeScript AOT 运行时库入口。
 *
 * 只包含 runtime helpers，供 dejavu-codegen AOT 产物 import。
 * 不含 parser、模板编译器、loader、resolver 或 Node 文件系统依赖。
 */

export {
    htmlEscape,
    valueToString,
    isTruthy,
    applyFilter,
    binary,
    unary,
    exprMember,
    exprIndex,
} from "./runtime/index.js";
