/** Dejavu IR v1 types (aligned with specifications/ir/v1/schema.json). */

export type Trim = "none" | "ws" | "nl" | "ws_nl" | "all";

export interface Span {
    start: number;
    end: number;
}

export interface TemplateConfig {
    codeStart: string;
    codeEnd: string;
    commentStart: string;
    commentEnd: string;
    supportFilterPipe: boolean;
    legacyFor: boolean;
}

export interface Language {
    syntaxMode: "template" | "programming";
    template: TemplateConfig;
}

export const DEFAULT_LANGUAGE: Language = {
    syntaxMode: "template",
    template: {
        codeStart: "<%",
        codeEnd: "%>",
        commentStart: "<#",
        commentEnd: "#>",
        supportFilterPipe: true,
        legacyFor: false,
    },
};

export type IrValue = null | boolean | number | string | IrValue[] | { [k: string]: IrValue };

export interface IrDocument {
    irVersion: "1.0";
    language: Language;
    body: IrNode;
}

export type IrNode =
    | { type: "Template"; children: IrNode[]; span?: Span }
    | { type: "Text"; value: string; span?: Span }
    | { type: "Comment"; value: string; span?: Span }
    | {
    type: "Interpolation";
    expression: IrNode;
    trim: Trim;
    raw?: boolean;
    span?: Span;
}
    | {
    type: "Stmt.If";
    test: IrNode;
    consequent: IrNode[];
    elseIfs: IrNode[];
    alternate?: IrNode[];
    trim: Trim;
    span?: Span;
}
    | {
    type: "Stmt.ElseIf";
    test: IrNode;
    consequent: IrNode[];
    trim: Trim;
    span?: Span;
}
    | {
    type: "Stmt.For";
    item: string;
    index?: string;
    iterable: IrNode;
    body: IrNode[];
    trim: Trim;
    span?: Span;
}
    | {
    type: "Stmt.Block";
    name: string;
    body: IrNode[];
    trim: Trim;
    span?: Span;
}
    | { type: "Stmt.Extends"; parent: IrNode; trim: Trim; span?: Span }
    | { type: "Stmt.Include"; path: IrNode; trim: Trim; span?: Span }
    | { type: "Stmt.Super"; trim: Trim; span?: Span }
    | { type: "Stmt.Raw"; value: string; trim: Trim; span?: Span }
    | { type: "Expr.Literal"; value: IrValue; span?: Span }
    | { type: "Expr.Identifier"; name: string; span?: Span }
    | { type: "Expr.Member"; object: IrNode; property: string; span?: Span }
    | { type: "Expr.Index"; object: IrNode; index: IrNode; span?: Span }
    | { type: "Expr.Call"; callee: IrNode; arguments: IrNode[]; span?: Span }
    | {
    type: "Expr.Binary";
    operator: string;
    left: IrNode;
    right: IrNode;
    span?: Span;
}
    | { type: "Expr.Unary"; operator: string; argument: IrNode; span?: Span }
    | {
    type: "Expr.Pipe";
    expression: IrNode;
    filter: string;
    arguments: IrNode[];
    span?: Span;
};
