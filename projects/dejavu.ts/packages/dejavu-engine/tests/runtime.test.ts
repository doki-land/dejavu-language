import { describe, it, expect } from "vitest";
import {
    htmlEscape,
    valueToString,
    isTruthy,
    applyFilter,
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

    it("Array → 递归 join", () => {
        expect(valueToString([1, "a", true])).toBe("1,a,true");
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

    it("raw / safe / s → 跳过 escape", () => {
        expect(applyFilter("raw", "<b>", [])).toBe("<b>");
        expect(applyFilter("safe", "<b>", [])).toBe("<b>");
        expect(applyFilter("s", "<b>", [])).toBe("<b>");
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

    it("default —— truthy 时返回原值，否则返回 args[0]", () => {
        expect(applyFilter("default", "hi", ["fallback"])).toBe("hi");
        expect(applyFilter("default", "", ["fallback"])).toBe("fallback");
        expect(applyFilter("default", 0, ["fallback"])).toBe("fallback");
        expect(applyFilter("default", "", [])).toBe("");
    });

    it("length", () => {
        expect(applyFilter("length", "hello", [])).toBe(5);
        expect(applyFilter("length", [1, 2, 3], [])).toBe(3);
    });

    it("join —— 数组用分隔符 join，非数组走 string", () => {
        expect(applyFilter("join", [1, 2, 3], ["-"])).toBe("1-2-3");
        expect(applyFilter("join", "abc", [])).toBe("abc");
    });

    it("replace", () => {
        expect(applyFilter("replace", "hello world", ["world", "dejavu"])).toBe(
            "hello dejavu",
        );
    });

    it("date —— Date 对象 → ISO", () => {
        const d = new Date("2024-01-02T03:04:05.000Z");
        expect(applyFilter("date", d, [])).toBe("2024-01-02T03:04:05.000Z");
    });

    it("slug", () => {
        expect(applyFilter("slug", "Hello World!", [])).toBe("hello-world");
        expect(applyFilter("slug", "  --hello--  ", [])).toBe("hello");
        // Unicode combining marks
        expect(applyFilter("slug", "Café", [])).toBe("cafe");
    });

    it("未识别 filter → 原样返回 value（宽松语义）", () => {
        expect(applyFilter("does-not-exist", 42, [])).toBe(42);
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

    it("== / != JS 宽松相等", () => {
        expect(binary("==", 1, 1)).toBe(true);
        expect(binary("==", 1, "1")).toBe(true);
        expect(binary("!=", 1, 2)).toBe(true);
    });

    it("=== / !== JS 严格相等", () => {
        expect(binary("===", 1, 1)).toBe(true);
        expect(binary("===", 1, "1")).toBe(false);
    });

    it("< <= > >=", () => {
        expect(binary("<", 1, 2)).toBe(true);
        expect(binary("<=", 2, 2)).toBe(true);
        expect(binary(">", 2, 1)).toBe(true);
        expect(binary(">=", 2, 2)).toBe(true);
    });

    it("&& / || 短路返回原值", () => {
        expect(binary("&&", "left", "right")).toBe("right");
        expect(binary("&&", false, "right")).toBe(false);
        expect(binary("||", false, "right")).toBe("right");
        expect(binary("||", "left", "right")).toBe("left");
    });

    it("in —— object 检查 key 存在", () => {
        expect(binary("in", "a", { a: 1 })).toBe(true);
        expect(binary("in", "b", { a: 1 })).toBe(false);
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

    it("- 数值负", () => {
        expect(unary("-", 5)).toBe(-5);
        expect(unary("-", "3")).toBe(-3);
        // -(0 || 0) === -0，JS 中与 0 数值相等但 Object.is 不同
        expect(unary("-", null)).toBe(-0);
        expect(Object.is(unary("-", null), -0)).toBe(true);
    });

    it("+ 数值正", () => {
        expect(unary("+", 5)).toBe(5);
        expect(unary("+", "3")).toBe(3);
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
