import { describe, it, expect } from "vitest";
import type { CustomRegistry } from "../src/runtime/index.js";
import {
    htmlEscape,
    valueToString,
    isTruthy,
    applyFilter,
    callFunction,
    applyTest,
    binary,
    unary,
    exprMember,
    exprIndex,
} from "../src/runtime/index.js";

// ---------------------------------------------------------------------------
// htmlEscape
// ---------------------------------------------------------------------------
describe("htmlEscape", () => {
    it("转义 & < > \" '", () => {
        expect(htmlEscape('Hello <world> & "friends"')).toBe(
            "Hello &lt;world&gt; &amp; &quot;friends&quot;",
        );
    });

    it("单引号转义为 &#39;", () => {
        expect(htmlEscape("it's")).toBe("it&#39;s");
    });

    it("非危险字符原样返回", () => {
        expect(htmlEscape("hello world 你好")).toBe("hello world 你好");
    });
});

// ---------------------------------------------------------------------------
// valueToString
// ---------------------------------------------------------------------------
describe("valueToString", () => {
    it("null / undefined → 空串", () => {
        expect(valueToString(null)).toBe("");
        expect(valueToString(undefined)).toBe("");
    });

    it("boolean → true/false", () => {
        expect(valueToString(true)).toBe("true");
        expect(valueToString(false)).toBe("false");
    });

    it("number → String()", () => {
        expect(valueToString(42)).toBe("42");
        expect(valueToString(3.14)).toBe("3.14");
        expect(valueToString(NaN)).toBe("NaN");
        expect(valueToString(Infinity)).toBe("Infinity");
    });

    it("Date → toISOString()", () => {
        const d = new Date("2024-01-02T03:04:05.000Z");
        expect(valueToString(d)).toBe("2024-01-02T03:04:05.000Z");
    });

    it("Array → 紧凑 JSON", () => {
        expect(valueToString([1, "a", true])).toBe('[1,"a",true]');
    });

    it("object → JSON.stringify", () => {
        expect(valueToString({ a: 1 })).toBe('{"a":1}');
    });

    it("string 原样", () => {
        expect(valueToString("hello")).toBe("hello");
    });
});

// ---------------------------------------------------------------------------
// isTruthy —— Dejavu 1.0 truthiness（不是 JS falsy）
// ---------------------------------------------------------------------------
describe("isTruthy", () => {
    const falsyCases: unknown[] = [null, undefined, "", false, 0, [], {}];
    for (const v of falsyCases) {
        it(`将 ${JSON.stringify(v)} 判为 false`, () => {
            expect(isTruthy(v)).toBe(false);
        });
    }

    const truthyCases: unknown[] = [
        "0", // string "0" → true
        "hello",
        1,
        -1,
        true,
        [0],
        { a: 0 },
        " ",
    ];
    for (const v of truthyCases) {
        it(`将 ${JSON.stringify(v)} 判为 true`, () => {
            expect(isTruthy(v)).toBe(true);
        });
    }
});

// ---------------------------------------------------------------------------
// applyFilter
// ---------------------------------------------------------------------------
describe("applyFilter", () => {
    it("escape / e 别名 → htmlEscape(valueToString)", () => {
        expect(applyFilter("escape", "<b>", [])).toBe("&lt;b&gt;");
        expect(applyFilter("e", "<b>", [])).toBe("&lt;b&gt;");
    });

    it("raw / safe → 跳过 escape", () => {
        expect(applyFilter("raw", "<b>", [])).toBe("<b>");
        expect(applyFilter("safe", "<b>", [])).toBe("<b>");
    });

    it("uppercase / upper → toUpperCase", () => {
        expect(applyFilter("uppercase", "hi", [])).toBe("HI");
        expect(applyFilter("upper", "hi", [])).toBe("HI");
    });

    it("lowercase / lower → toLowerCase", () => {
        expect(applyFilter("lowercase", "HI", [])).toBe("hi");
        expect(applyFilter("lower", "HI", [])).toBe("hi");
    });

    it("trim", () => {
        expect(applyFilter("trim", "  hi  ", [])).toBe("hi");
    });

    it("default —— 只有 null / 空串算缺失", () => {
        expect(applyFilter("default", "hi", ["fallback"])).toBe("hi");
        expect(applyFilter("default", "", ["fallback"])).toBe("fallback");
        expect(applyFilter("default", null, ["fallback"])).toBe("fallback");
        // 0 / false / [] 保留原值（不是 JS falsy）
        expect(applyFilter("default", 0, ["fallback"])).toBe(0);
        expect(applyFilter("default", false, ["fallback"])).toBe(false);
        expect(applyFilter("default", [], ["fallback"])).toEqual([]);
        expect(() => applyFilter("default", "", [])).toThrow(
            "dejavu.template.filter-arity",
        );
    });

    it("length", () => {
        expect(applyFilter("length", "hello", [])).toBe(5);
        expect(applyFilter("length", [1, 2, 3], [])).toBe(3);
        expect(applyFilter("length", { a: 1, b: 2 }, [])).toBe(2);
        expect(applyFilter("length", 42, [])).toBe(0);
    });

    it("join —— 数组用分隔符 join，非数组走 valueToString", () => {
        expect(applyFilter("join", [1, 2, 3], ["-"])).toBe("1-2-3");
        expect(applyFilter("join", ["a", null], [])).toBe("a,");
        expect(applyFilter("join", "abc", [])).toBe("abc");
        expect(applyFilter("join", null, [])).toBe("");
    });

    it("replace —— 替换全部出现", () => {
        expect(applyFilter("replace", "hello world", ["world", "dejavu"])).toBe(
            "hello dejavu",
        );
        expect(applyFilter("replace", "a-a-a", ["a", "b"])).toBe("b-b-b");
    });

    it("date —— 默认 %Y-%m-%d，与 Rust 一致", () => {
        const d = new Date("2024-01-02T03:04:05.000Z");
        expect(applyFilter("date", d, [])).toBe("2024-01-02");
        expect(applyFilter("date", "2024-01-02T03:04:05.000Z", ["%Y/%m/%d %H:%M:%S"])).toBe(
            "2024/01/02 03:04:05",
        );
        expect(applyFilter("date", "", [])).toBe("");
        expect(applyFilter("date", "not-a-date", [])).toBe("");
    });

    it("slug", () => {
        expect(applyFilter("slug", "Hello World!", [])).toBe("hello-world");
        expect(applyFilter("slug", "  --hello--  ", [])).toBe("hello");
        // Unicode combining marks
        expect(applyFilter("slug", "Café", [])).toBe("cafe");
    });

    it("未识别 filter / 参数个数违约 → 抛结构化错误", () => {
        expect(() => applyFilter("does-not-exist", 42, [])).toThrow(
            "dejavu.template.unknown-filter: does-not-exist",
        );
        expect(() => applyFilter("replace", "a", ["a"])).toThrow(
            "dejavu.template.filter-arity",
        );
    });
});

// ---------------------------------------------------------------------------
// binary
// ---------------------------------------------------------------------------
describe("binary", () => {
    it("+ 数值加法", () => {
        expect(binary("+", 1, 2)).toBe(3);
    });

    it("+ 字符串拼接（任一为 string）", () => {
        expect(binary("+", 1, "2")).toBe("12");
        expect(binary("+", "a", "b")).toBe("ab");
    });

    it("- * / % 数值运算", () => {
        expect(binary("-", 5, 3)).toBe(2);
        expect(binary("*", 2, 3)).toBe(6);
        expect(binary("/", 10, 2)).toBe(5);
        expect(binary("%", 10, 3)).toBe(1);
    });

    it("== / != 结构相等，不做类型转换", () => {
        expect(binary("==", 1, 1)).toBe(true);
        expect(binary("==", 1, "1")).toBe(false);
        expect(binary("==", [1, 2], [1, 2])).toBe(true);
        expect(binary("==", { a: 1 }, { a: 1 })).toBe(true);
        expect(binary("==", { a: 1 }, { a: 2 })).toBe(false);
        expect(binary("!=", 1, 2)).toBe(true);
    });

    it("< <= > >= 只比较 number", () => {
        expect(binary("<", 1, 2)).toBe(true);
        expect(binary("<=", 2, 2)).toBe(true);
        expect(binary(">", 2, 1)).toBe(true);
        expect(binary(">=", 2, 2)).toBe(true);
        // 非 number 不借用 JS 字符串比较
        expect(binary("<", "10", "9")).toBe(false);
    });

    it("&& / || 返回 boolean", () => {
        expect(binary("&&", "left", "right")).toBe(true);
        expect(binary("&&", false, "right")).toBe(false);
        expect(binary("||", false, "right")).toBe(true);
        expect(binary("||", "", [])).toBe(false);
    });

    it("in —— 数组按成员、字符串按子串、对象按 key", () => {
        expect(binary("in", "a", { a: 1 })).toBe(true);
        expect(binary("in", "b", { a: 1 })).toBe(false);
        expect(binary("in", 2, [1, 2, 3])).toBe(true);
        expect(binary("in", "ell", "hello")).toBe(true);
        expect(binary("in", "a", null)).toBe(false);
    });

    it("未识别 op → null", () => {
        expect(binary("??", 1, 2)).toBeNull();
    });
});

// ---------------------------------------------------------------------------
// unary
// ---------------------------------------------------------------------------
describe("unary", () => {
    it("! 使用 Dejavu truthiness（不是 JS falsy）", () => {
        expect(unary("!", null)).toBe(true); // falsy → ! → true
        expect(unary("!", "")).toBe(true);
        expect(unary("!", 0)).toBe(true);
        expect(unary("!", "0")).toBe(false); // truthy → ! → false
        expect(unary("!", {})).toBe(true); // 空对象 truthiness = false
        expect(unary("!", { a: 1 })).toBe(false);
    });

    it("- 只接受 number", () => {
        expect(unary("-", 5)).toBe(-5);
        expect(unary("-", "3")).toBeNull();
        expect(unary("-", null)).toBeNull();
    });

    it("+ 是恒等", () => {
        expect(unary("+", 5)).toBe(5);
        expect(unary("+", "3")).toBe("3");
        expect(unary("+", null)).toBeNull();
    });

    it("未识别 op → null", () => {
        expect(unary("~", 5)).toBeNull();
    });
});

// ---------------------------------------------------------------------------
// exprMember
// ---------------------------------------------------------------------------
describe("exprMember", () => {
    it("obj.property", () => {
        expect(exprMember({ name: "alice" }, "name")).toBe("alice");
    });

    it("obj 为 null/undefined → null", () => {
        expect(exprMember(null, "name")).toBeNull();
        expect(exprMember(undefined, "name")).toBeNull();
    });

    it("键不存在 → null", () => {
        expect(exprMember({ a: 1 }, "b")).toBeNull();
    });

    it("值为 undefined 时也返回 null", () => {
        expect(exprMember({ a: undefined }, "a")).toBeNull();
    });
});

// ---------------------------------------------------------------------------
// exprIndex
// ---------------------------------------------------------------------------
describe("exprIndex", () => {
    it("arr[i]", () => {
        expect(exprIndex(["a", "b", "c"], 0)).toBe("a");
        expect(exprIndex(["a", "b", "c"], 2)).toBe("c");
    });

    it("越界 → null", () => {
        expect(exprIndex(["a", "b", "c"], 100)).toBeNull();
    });

    it("对象索引（用 String(index)）", () => {
        expect(exprIndex({ key: "val" }, "key")).toBe("val");
        expect(exprIndex({ key: "val" }, "missing")).toBeNull();
    });

    it("null / undefined → null", () => {
        expect(exprIndex(null, 0)).toBeNull();
        expect(exprIndex(undefined, 0)).toBeNull();
    });

    it("非数组非对象 → null", () => {
        expect(exprIndex(42, 0)).toBeNull();
    });
});

// ---------------------------------------------------------------------------
// function 命名空间
// ---------------------------------------------------------------------------
describe("callFunction", () => {
    it("range 生成整数序列", () => {
        expect(callFunction("range", [0, 3])).toEqual([0, 1, 2]);
        expect(callFunction("range", [3, 0, -1])).toEqual([3, 2, 1]);
    });

    it("range 非法参数 → null", () => {
        expect(callFunction("range", [0, 3, 0])).toBeNull();
        expect(callFunction("range", [0.5, 3])).toBeNull();
        expect(callFunction("range", [0, 1_000_000])).toBeNull();
    });

    it("min / max 只接受有限数字", () => {
        expect(callFunction("min", [4, 2.5])).toBe(2.5);
        expect(callFunction("max", [4, 2.5])).toBe(4);
        expect(callFunction("min", [4, "x"])).toBeNull();
    });

    it("未知函数与参数个数违约都抛错", () => {
        expect(() => callFunction("nope", [])).toThrow(
            "dejavu.template.unknown-function",
        );
        expect(() => callFunction("range", [0])).toThrow(
            "dejavu.template.function-arity",
        );
    });
});

// ---------------------------------------------------------------------------
// test 命名空间
// ---------------------------------------------------------------------------
describe("applyTest", () => {
    it("值模型谓词与 Rust 一致", () => {
        expect(applyTest("empty", [], [])).toBe(true);
        expect(applyTest("empty", "x", [])).toBe(false);
        expect(applyTest("array", [], [])).toBe(true);
        expect(applyTest("object", { a: 1 }, [])).toBe(true);
        expect(applyTest("truthy", "0", [])).toBe(true);
        expect(applyTest("truthy", 0, [])).toBe(false);
        expect(applyTest("null", null, [])).toBe(true);
        expect(applyTest("even", 4, [])).toBe(true);
        expect(applyTest("even", 4.5, [])).toBe(false);
        expect(applyTest("odd", -3, [])).toBe(true);
    });

    it("is not 取反", () => {
        expect(applyTest("empty", [], [], true)).toBe(false);
        expect(applyTest("none", null, [], true)).toBe(false);
    });

    it("未知 test 与参数个数违约都抛错", () => {
        expect(() => applyTest("nope", 1, [])).toThrow(
            "dejavu.template.unknown-test",
        );
        expect(() => applyTest("empty", 1, [1])).toThrow(
            "dejavu.template.test-arity",
        );
    });
});

// ---------------------------------------------------------------------------
// custom registry（AOT 编译期已解析 alias 与 arity）
// ---------------------------------------------------------------------------
describe("custom registry", () => {
    const custom: CustomRegistry = {
        filter: {
            shout: (value) => `${valueToString(value)}!`,
            bolder: (value) => `<b>${valueToString(value)}</b>`,
        },
        function: {
            twice: (args) => Number(args[0]) * 2,
        },
        test: {
            evenish: (value) => Number(value) % 2 === 0,
        },
    };

    it("applyFilter 命中自定义过滤器并以 canonical name 为键", () => {
        expect(applyFilter("shout", "hi", [], custom)).toBe("hi!");
        expect(applyFilter("bolder", "<i>", [], custom)).toBe("<b><i></b>");
    });

    it("applyFilter 内置优先于 custom", () => {
        expect(
            applyFilter("upper", "hi", [], { filter: { upper: () => "hijacked" } }),
        ).toBe("HI");
    });

    it("applyFilter 未命中仍抛 unknown-filter", () => {
        expect(() => applyFilter("absent", "x", [], custom)).toThrow(
            "dejavu.template.unknown-filter",
        );
    });

    it("callFunction 命中自定义函数", () => {
        expect(callFunction("twice", [21], custom)).toBe(42);
    });

    it("callFunction 未命中仍抛 unknown-function", () => {
        expect(() => callFunction("absent", [], custom)).toThrow(
            "dejavu.template.unknown-function",
        );
    });

    it("applyTest 命中自定义谓词并支持 is not 取反", () => {
        expect(applyTest("evenish", 4, [], false, custom)).toBe(true);
        expect(applyTest("evenish", 4, [], true, custom)).toBe(false);
        expect(applyTest("evenish", 3, [], true, custom)).toBe(true);
    });

    it("applyTest 未命中仍抛 unknown-test", () => {
        expect(() => applyTest("absent", 1, [], false, custom)).toThrow(
            "dejavu.template.unknown-test",
        );
    });

    it("宿主实现抛错统一为 custom-invocation", () => {
        const boom = (): never => {
            throw new Error("host exploded");
        };
        const failing: CustomRegistry = {
            filter: { shout: boom },
            function: { twice: boom },
            test: { evenish: boom },
        };
        const cases = [
            () => applyFilter("shout", "hi", [], failing),
            () => callFunction("twice", [21], failing),
            () => applyTest("evenish", 4, [], false, failing),
        ];
        const namespaces = ["filter", "function", "test"];
        for (const [index, run] of cases.entries()) {
            try {
                run();
                throw new Error("expected host failure");
            } catch (error) {
                const failure = error as { code: string; args: Record<string, unknown> };
                expect(failure.code).toBe("dejavu.template.custom-invocation");
                expect(failure.args.namespace).toBe(namespaces[index]);
                expect(failure.args["message"]).toBe("host exploded");
            }
        }
    });
});
