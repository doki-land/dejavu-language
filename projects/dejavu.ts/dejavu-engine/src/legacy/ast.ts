/**
 * DejaVu 模板引擎抽象语法树 (AST) 定义
 */

/**
 * AST 节点类型枚举
 */
export enum NodeType {
    /** 程序节点 */
    Program = "Program",
    /** 表达式语句节点 */
    ExpressionStatement = "ExpressionStatement",
    /** 赋值语句节点 */
    AssignmentStatement = "AssignmentStatement",
    /** 变量声明节点 */
    VariableDeclaration = "VariableDeclaration",
    /** 函数声明节点 */
    FunctionDeclaration = "FunctionDeclaration",
    /** If 语句节点 */
    IfStatement = "IfStatement",
    /** Else If 分支节点 */
    ElseIfClause = "ElseIfClause",
    /** For 循环语句节点 */
    ForStatement = "ForStatement",
    /** For-In 遍历循环语句节点 */
    ForInStatement = "ForInStatement",
    /** While 循环语句节点 */
    WhileStatement = "WhileStatement",
    /** 匹配语句节点 */
    MatchStatement = "MatchStatement",
    /** 匹配分支节点 */
    MatchCase = "MatchCase",
    /** 命名空间声明节点 */
    NamespaceDeclaration = "NamespaceDeclaration",
    /** 导入声明节点 */
    UsingDeclaration = "UsingDeclaration",
    /** 块语句节点 */
    BlockStatement = "BlockStatement",
    /** Break 语句节点 */
    BreakStatement = "BreakStatement",
    /** Continue 语句节点 */
    ContinueStatement = "ContinueStatement",
    /** Extends 语句节点 */
    ExtendsStatement = "ExtendsStatement",
    /** Block 声明节点 */
    BlockDeclaration = "BlockDeclaration",
    /** Prepend 语句节点 */
    PrependStatement = "PrependStatement",
    /** Append 语句节点 */
    AppendStatement = "AppendStatement",
    /** Super 表达式节点 */
    SuperExpression = "SuperExpression",
    /** 标识符节点 */
    Identifier = "Identifier",
    /** 字面量节点 */
    Literal = "Literal",
    /** 二元表达式节点 */
    BinaryExpression = "BinaryExpression",
    /** 一元表达式节点 */
    UnaryExpression = "UnaryExpression",
    /** 函数调用表达式节点 */
    CallExpression = "CallExpression",
    /** 成员访问表达式节点 */
    MemberExpression = "MemberExpression",
    /** 数组表达式节点 */
    ArrayExpression = "ArrayExpression",
    /** 元组表达式节点 */
    TupleExpression = "TupleExpression",
    /** 对象表达式节点 */
    ObjectExpression = "ObjectExpression",
    /** 对象属性节点 */
    ObjectProperty = "ObjectProperty",
    /** 范围表达式节点 */
    RangeExpression = "RangeExpression",
    /** 类型转换表达式节点 */
    AsExpression = "AsExpression",
    /** 管道表达式节点 */
    PipeExpression = "PipeExpression",
    /** 数组类型注解节点 */
    ArrayTypeAnnotation = "ArrayTypeAnnotation",
    /** 元组类型注解节点 */
    TupleTypeAnnotation = "TupleTypeAnnotation",
    /** 带类型注解的参数节点 */
    TypedParameter = "TypedParameter",
    /** 模板字符串节点 */
    TemplateLiteral = "TemplateLiteral",
    /** 模板字符串表达式节点 */
    TemplateExpression = "TemplateExpression",
    /** 注释节点 */
    Comment = "Comment",
    /** 文本节点 */
    Text = "Text",
    /** Include 语句节点 */
    IncludeStatement = "IncludeStatement",
}

/**
 * AST 节点基础接口
 */
export interface Node {
    /** 节点类型 */
    type: NodeType;
    /** 节点位置信息 */
    loc?: {
        /** 开始位置 */
        start: {
            line: number;
            column: number;
        };
        /** 结束位置 */
        end: {
            line: number;
            column: number;
        };
    };
}

/**
 * 表达式节点接口
 */
export interface Expression extends Node {
    /** 表达式类型 */
    type:
        | NodeType.Identifier
        | NodeType.Literal
        | NodeType.BinaryExpression
        | NodeType.UnaryExpression
        | NodeType.CallExpression
        | NodeType.MemberExpression
        | NodeType.ArrayExpression
        | NodeType.TupleExpression
        | NodeType.ObjectExpression
        | NodeType.RangeExpression
        | NodeType.TemplateLiteral
        | NodeType.TemplateExpression
        | NodeType.AsExpression
        | NodeType.SuperExpression
        | NodeType.PipeExpression;
}

/**
 * 语句节点接口
 */
export interface Statement extends Node {
    /** 语句类型 */
    type:
        | NodeType.ExpressionStatement
        | NodeType.AssignmentStatement
        | NodeType.VariableDeclaration
        | NodeType.FunctionDeclaration
        | NodeType.IfStatement
        | NodeType.ElseIfClause
        | NodeType.ForStatement
        | NodeType.ForInStatement
        | NodeType.WhileStatement
        | NodeType.MatchStatement
        | NodeType.MatchCase
        | NodeType.NamespaceDeclaration
        | NodeType.UsingDeclaration
        | NodeType.BlockStatement
        | NodeType.BreakStatement
        | NodeType.ContinueStatement
        | NodeType.ExtendsStatement
        | NodeType.BlockDeclaration
        | NodeType.PrependStatement
        | NodeType.AppendStatement
        | NodeType.IncludeStatement;
}

/**
 * 程序节点
 * 表示整个模板的根节点
 */
export interface Program extends Node {
    type: NodeType.Program;
    /** 程序体，包含所有语句 */
    body: (Statement | Expression | Text | Comment)[];
}

/**
 * 表达式语句节点
 * 表示一个作为语句的表达式
 */
export interface ExpressionStatement extends Statement {
    type: NodeType.ExpressionStatement;
    /** 表达式 */
    expression: Expression;
}

/**
 * 赋值语句节点
 * 表示变量赋值操作
 */
export interface AssignmentStatement extends Statement {
    type: NodeType.AssignmentStatement;
    /** 左值表达式 */
    left: Expression;
    /** 右值表达式 */
    right: Expression;
}

/**
 * 变量声明节点
 * 表示变量声明语句
 */
export interface VariableDeclaration extends Statement {
    type: NodeType.VariableDeclaration;
    /** 变量名称 */
    name: Identifier;
    /** 变量类型注解 */
    typeAnnotation?: TypeAnnotation;
    /** 变量初始化值 */
    init?: Expression;
}

/**
 * 函数声明节点
 * 表示函数声明语句
 */
export interface FunctionDeclaration extends Statement {
    type: NodeType.FunctionDeclaration;
    /** 函数名称 */
    name: Identifier;
    /** 函数参数 */
    params: Identifier[];
    /** 函数体 */
    body: BlockStatement;
}

/**
 * If 语句节点
 * 表示条件语句
 */
export interface IfStatement extends Statement {
    type: NodeType.IfStatement;
    /** 条件表达式 */
    test: Expression;
    /** 当条件为真时执行的语句 */
    consequent: Statement;
    /** else if 分支列表 */
    elseIfs?: ElseIfClause[];
    /** 当所有条件都为假时执行的语句 */
    alternate?: Statement;
}

/**
 * Else If 分支节点
 * 表示 else if 分支
 */
export interface ElseIfClause extends Statement {
    type: NodeType.ElseIfClause;
    /** 条件表达式 */
    test: Expression;
    /** 当条件为真时执行的语句 */
    consequent: Statement;
}

/**
 * For 循环语句节点
 * 表示 for 循环
 */
export interface ForStatement extends Statement {
    type: NodeType.ForStatement;
    /** 初始化表达式 */
    init: VariableDeclaration | AssignmentStatement | Expression | null;
    /** 条件表达式 */
    test: Expression | null;
    /** 增量表达式 */
    update: Expression | null;
    /** 循环体 */
    body: Statement;
}

/**
 * For-In 遍历循环语句节点
 * 表示 loop item in list 遍历循环
 */
export interface ForInStatement extends Statement {
    type: NodeType.ForInStatement;
    /** 循环变量（单个变量或索引+元素元组） */
    left: Identifier | [Identifier, Identifier];
    /** 被遍历的表达式（数组或范围） */
    right: Expression;
    /** 循环体 */
    body: Statement;
}

/**
 * While 循环语句节点
 * 表示 while 循环
 */
export interface WhileStatement extends Statement {
    type: NodeType.WhileStatement;
    /** 条件表达式 */
    test: Expression;
    /** 循环体 */
    body: Statement;
}

/**
 * 匹配语句节点
 * 表示 match 匹配语句
 */
export interface MatchStatement extends Statement {
    type: NodeType.MatchStatement;
    /** 被匹配的表达式 */
    discriminant: Expression;
    /** 匹配分支列表 */
    cases: MatchCase[];
}

/**
 * 匹配分支节点
 * 表示 match 语句中的 case 分支
 */
export interface MatchCase extends Statement {
    type: NodeType.MatchCase;
    /** 匹配模式（表达式或通配符 _） */
    pattern: Expression | null;
    /** 匹配成功时执行的语句 */
    consequent: Statement;
}

/**
 * 命名空间声明节点
 * 表示 namespace 声明
 */
export interface NamespaceDeclaration extends Statement {
    type: NodeType.NamespaceDeclaration;
    /** 命名空间名称 */
    name: Identifier;
}

/**
 * 导入声明节点
 * 表示 using 导入声明
 */
export interface UsingDeclaration extends Statement {
    type: NodeType.UsingDeclaration;
    /** 模块路径（如 header::render_header） */
    modulePath: Identifier[];
}

/**
 * 块语句节点
 * 表示一组语句的集合
 */
export interface BlockStatement extends Statement {
    type: NodeType.BlockStatement;
    /** 块内的语句 */
    body: (Statement | Expression | Comment)[];
}

/**
 * Break 语句节点
 * 表示跳出循环
 */
export interface BreakStatement extends Statement {
    type: NodeType.BreakStatement;
}

/**
 * Continue 语句节点
 * 表示跳过当前迭代
 */
export interface ContinueStatement extends Statement {
    type: NodeType.ContinueStatement;
}

/**
 * Extends 语句节点
 * 表示模板继承
 */
export interface ExtendsStatement extends Statement {
    type: NodeType.ExtendsStatement;
    /** 父模板名称 */
    template: Literal;
}

/**
 * Block 声明节点
 * 定义可被子模板覆盖的块
 */
export interface BlockDeclaration extends Statement {
    type: NodeType.BlockDeclaration;
    /** 块名称 */
    name: Identifier;
    /** 块内容 */
    body: (Statement | Expression | Comment)[];
}

/**
 * Prepend 语句节点
 * 在父块内容前追加内容
 */
export interface PrependStatement extends Statement {
    type: NodeType.PrependStatement;
    /** 块名称 */
    name: Identifier;
    /** 追加内容 */
    body: (Statement | Expression | Comment)[];
}

/**
 * Append 语句节点
 * 在父块内容后追加内容
 */
export interface AppendStatement extends Statement {
    type: NodeType.AppendStatement;
    /** 块名称 */
    name: Identifier;
    /** 追加内容 */
    body: (Statement | Expression | Comment)[];
}

/**
 * Super 表达式节点
 * 调用父块内容
 */
export interface SuperExpression extends Expression {
    type: NodeType.SuperExpression;
}

/**
 * 标识符节点
 * 表示变量名、函数名等标识符
 */
export interface Identifier extends Expression {
    type: NodeType.Identifier;
    /** 标识符名称 */
    name: string;
}

/**
 * 字面量节点
 * 表示字符串、数字、布尔值等字面量
 */
export interface Literal extends Expression {
    type: NodeType.Literal;
    /** 字面量值 */
    value: string | number | boolean | null | undefined;
    /** 原始字符串表示 */
    raw?: string;
}

/**
 * 二元表达式节点
 * 表示二元运算表达式
 */
export interface BinaryExpression extends Expression {
    type: NodeType.BinaryExpression;
    /** 运算符 */
    operator: string;
    /** 左操作数 */
    left: Expression;
    /** 右操作数 */
    right: Expression;
}

/**
 * 一元表达式节点
 * 表示一元运算表达式
 */
export interface UnaryExpression extends Expression {
    type: NodeType.UnaryExpression;
    /** 运算符 */
    operator: string;
    /** 操作数 */
    argument: Expression;
    /** 是否为前缀运算符 */
    prefix: boolean;
}

/**
 * 函数调用表达式节点
 * 表示函数调用
 */
export interface CallExpression extends Expression {
    type: NodeType.CallExpression;
    /** 被调用的函数表达式 */
    callee: Expression;
    /** 函数参数 */
    arguments: Expression[];
}

/**
 * 成员访问表达式节点
 * 表示对象成员访问
 */
export interface MemberExpression extends Expression {
    type: NodeType.MemberExpression;
    /** 对象表达式 */
    object: Expression;
    /** 成员表达式 */
    property: Expression;
    /** 是否使用点号访问 */
    computed: boolean;
}

/**
 * 数组表达式节点
 * 表示数组字面量
 */
export interface ArrayExpression extends Expression {
    type: NodeType.ArrayExpression;
    /** 数组元素 */
    elements: Expression[];
}

/**
 * 元组表达式节点
 * 表示元组字面量
 */
export interface TupleExpression extends Expression {
    type: NodeType.TupleExpression;
    /** 元组元素 */
    elements: Expression[];
}

/**
 * 对象表达式节点
 * 表示对象字面量
 */
export interface ObjectExpression extends Expression {
    type: NodeType.ObjectExpression;
    /** 对象属性 */
    properties: ObjectProperty[];
}

/**
 * 对象属性节点
 * 表示对象的属性
 */
export interface ObjectProperty extends Node {
    type: NodeType.ObjectProperty;
    /** 属性键 */
    key: Identifier | Literal;
    /** 属性值 */
    value: Expression;
    /** 是否为简写属性 */
    shorthand: boolean;
}

/**
 * 范围表达式节点
 * 表示 start..end 范围表达式
 */
export interface RangeExpression extends Expression {
    type: NodeType.RangeExpression;
    /** 范围起始值 */
    start: Expression;
    /** 范围结束值 */
    end: Expression;
}

/**
 * 类型转换表达式节点
 * 表示 expression as Type 类型转换
 */
export interface AsExpression extends Expression {
    type: NodeType.AsExpression;
    /** 要转换的表达式 */
    expression: Expression;
    /** 目标类型 */
    typeAnnotation: Identifier;
}

/**
 * 管道表达式节点
 * 表示 value |> filter 或 value |> filter: arg1, arg2 管道操作
 */
export interface PipeExpression extends Expression {
    type: NodeType.PipeExpression;
    /** 输入表达式 */
    input: Expression;
    /** 过滤器名称 */
    filter: Identifier;
    /** 过滤器参数 */
    arguments: Expression[];
}

/**
 * 数组类型注解节点
 * 表示 [Type; N] 数组类型
 */
export interface ArrayTypeAnnotation extends Node {
    type: NodeType.ArrayTypeAnnotation;
    /** 元素类型 */
    elementType: TypeAnnotation;
    /** 数组长度（可选） */
    size?: Literal;
}

/**
 * 元组类型注解节点
 * 表示 (Type1, Type2, ...) 元组类型
 */
export interface TupleTypeAnnotation extends Node {
    type: NodeType.TupleTypeAnnotation;
    /** 元素类型列表 */
    elementTypes: TypeAnnotation[];
}

/**
 * 类型注解联合类型
 */
export type TypeAnnotation = Identifier | ArrayTypeAnnotation | TupleTypeAnnotation;

/**
 * 带类型注解的参数节点
 * 表示函数参数带类型注解
 */
export interface TypedParameter extends Node {
    type: NodeType.TypedParameter;
    /** 参数名称 */
    name: Identifier;
    /** 参数类型 */
    typeAnnotation?: Identifier;
}

/**
 * 模板字符串节点
 * 表示模板字符串
 */
export interface TemplateLiteral extends Expression {
    type: NodeType.TemplateLiteral;
    /** 模板字符串片段 */
    quasis: {
        value: {
            raw: string;
            cooked: string;
        };
    }[];
    /** 模板字符串中的表达式 */
    expressions: Expression[];
}

/**
 * 模板表达式节点
 * 表示 DejaVu 模板表达式
 */
export interface TemplateExpression extends Expression {
    type: NodeType.TemplateExpression;
    /** 模板内容 */
    content: string;
}

/**
 * 注释节点
 * 表示注释
 */
export interface Comment extends Node {
    type: NodeType.Comment;
    /** 注释内容 */
    value: string;
    /** 是否为块注释 */
    block: boolean;
}

/**
 * 文本节点
 * 表示模板中的普通文本
 */
export interface Text extends Node {
    type: NodeType.Text;
    /** 文本内容 */
    value: string;
}

/**
 * Include 语句节点
 * 表示引入子模板
 */
export interface IncludeStatement extends Statement {
    type: NodeType.IncludeStatement;
    /** 模板名称 */
    template: Expression;
    /** 传递的上下文（可选） */
    context?: Expression;
}

/**
 * 创建标识符节点
 * @param name 标识符名称
 * @param loc 位置信息
 * @returns 标识符节点
 */
export function createIdentifier(name: string, loc?: Node["loc"]): Identifier {
    return {
        type: NodeType.Identifier,
        name,
        loc,
    };
}

/**
 * 创建字面量节点
 * @param value 字面量值
 * @param raw 原始字符串表示
 * @param loc 位置信息
 * @returns 字面量节点
 */
export function createLiteral(value: Literal["value"], raw?: string, loc?: Node["loc"]): Literal {
    return {
        type: NodeType.Literal,
        value,
        raw,
        loc,
    };
}

/**
 * 创建程序节点
 * @param body 程序体
 * @param loc 位置信息
 * @returns 程序节点
 */
export function createProgram(body: Program["body"], loc?: Node["loc"]): Program {
    return {
        type: NodeType.Program,
        body,
        loc,
    };
}

/**
 * 创建表达式语句节点
 * @param expression 表达式
 * @param loc 位置信息
 * @returns 表达式语句节点
 */
export function createExpressionStatement(
    expression: Expression,
    loc?: Node["loc"],
): ExpressionStatement {
    return {
        type: NodeType.ExpressionStatement,
        expression,
        loc,
    };
}

/**
 * 创建变量声明节点
 * @param name 变量名称
 * @param init 初始化值
 * @param loc 位置信息
 * @returns 变量声明节点
 */
export function createVariableDeclaration(
    name: Identifier,
    init?: Expression,
    typeAnnotation?: TypeAnnotation,
    loc?: Node["loc"],
): VariableDeclaration {
    return {
        type: NodeType.VariableDeclaration,
        name,
        typeAnnotation,
        init,
        loc,
    };
}

/**
 * 创建块语句节点
 * @param body 块内语句
 * @param loc 位置信息
 * @returns 块语句节点
 */
export function createBlockStatement(
    body: BlockStatement["body"],
    loc?: Node["loc"],
): BlockStatement {
    return {
        type: NodeType.BlockStatement,
        body,
        loc,
    };
}

/**
 * 创建 Break 语句节点
 * @param loc 位置信息
 * @returns Break 语句节点
 */
export function createBreakStatement(loc?: Node["loc"]): BreakStatement {
    return {
        type: NodeType.BreakStatement,
        loc,
    };
}

/**
 * 创建 Continue 语句节点
 * @param loc 位置信息
 * @returns Continue 语句节点
 */
export function createContinueStatement(loc?: Node["loc"]): ContinueStatement {
    return {
        type: NodeType.ContinueStatement,
        loc,
    };
}

/**
 * 创建 Extends 语句节点
 * @param template 父模板名称
 * @param loc 位置信息
 * @returns Extends 语句节点
 */
export function createExtendsStatement(template: Literal, loc?: Node["loc"]): ExtendsStatement {
    return {
        type: NodeType.ExtendsStatement,
        template,
        loc,
    };
}

/**
 * 创建 Block 声明节点
 * @param name 块名称
 * @param body 块内容
 * @param loc 位置信息
 * @returns Block 声明节点
 */
export function createBlockDeclaration(
    name: Identifier,
    body: (Statement | Expression | Comment)[],
    loc?: Node["loc"],
): BlockDeclaration {
    return {
        type: NodeType.BlockDeclaration,
        name,
        body,
        loc,
    };
}

/**
 * 创建 Prepend 语句节点
 * @param name 块名称
 * @param body 追加内容
 * @param loc 位置信息
 * @returns Prepend 语句节点
 */
export function createPrependStatement(
    name: Identifier,
    body: (Statement | Expression | Comment)[],
    loc?: Node["loc"],
): PrependStatement {
    return {
        type: NodeType.PrependStatement,
        name,
        body,
        loc,
    };
}

/**
 * 创建 Append 语句节点
 * @param name 块名称
 * @param body 追加内容
 * @param loc 位置信息
 * @returns Append 语句节点
 */
export function createAppendStatement(
    name: Identifier,
    body: (Statement | Expression | Comment)[],
    loc?: Node["loc"],
): AppendStatement {
    return {
        type: NodeType.AppendStatement,
        name,
        body,
        loc,
    };
}

/**
 * 创建 Super 表达式节点
 * @param loc 位置信息
 * @returns Super 表达式节点
 */
export function createSuperExpression(loc?: Node["loc"]): SuperExpression {
    return {
        type: NodeType.SuperExpression,
        loc,
    };
}

/**
 * 创建二元表达式节点
 * @param operator 运算符
 * @param left 左操作数
 * @param right 右操作数
 * @param loc 位置信息
 * @returns 二元表达式节点
 */
export function createBinaryExpression(
    operator: string,
    left: Expression,
    right: Expression,
    loc?: Node["loc"],
): BinaryExpression {
    return {
        type: NodeType.BinaryExpression,
        operator,
        left,
        right,
        loc,
    };
}

/**
 * 创建函数调用表达式节点
 * @param callee 被调用的函数
 * @param args 函数参数
 * @param loc 位置信息
 * @returns 函数调用表达式节点
 */
export function createCallExpression(
    callee: Expression,
    args: Expression[],
    loc?: Node["loc"],
): CallExpression {
    return {
        type: NodeType.CallExpression,
        callee,
        arguments: args,
        loc,
    };
}

/**
 * 创建文本节点
 * @param value 文本内容
 * @param loc 位置信息
 * @returns 文本节点
 */
export function createText(value: string, loc?: Node["loc"]): Text {
    return {
        type: NodeType.Text,
        value,
        loc,
    };
}

/**
 * 创建模板表达式节点
 * @param content 模板内容
 * @param loc 位置信息
 * @returns 模板表达式节点
 */
export function createTemplateExpression(content: string, loc?: Node["loc"]): TemplateExpression {
    return {
        type: NodeType.TemplateExpression,
        content,
        loc,
    };
}

/**
 * 创建一元表达式节点
 * @param operator 运算符
 * @param argument 操作数
 * @param prefix 是否为前缀运算符
 * @param loc 位置信息
 * @returns 一元表达式节点
 */
export function createUnaryExpression(
    operator: string,
    argument: Expression,
    prefix: boolean,
    loc?: Node["loc"],
): UnaryExpression {
    return {
        type: NodeType.UnaryExpression,
        operator,
        argument,
        prefix,
        loc,
    };
}

/**
 * 创建成员访问表达式节点
 * @param object 对象表达式
 * @param property 成员表达式
 * @param computed 是否使用计算访问
 * @param loc 位置信息
 * @returns 成员访问表达式节点
 */
export function createMemberExpression(
    object: Expression,
    property: Expression,
    computed: boolean,
    loc?: Node["loc"],
): MemberExpression {
    return {
        type: NodeType.MemberExpression,
        object,
        property,
        computed,
        loc,
    };
}

/**
 * 创建数组表达式节点
 * @param elements 数组元素
 * @param loc 位置信息
 * @returns 数组表达式节点
 */
export function createArrayExpression(elements: Expression[], loc?: Node["loc"]): ArrayExpression {
    return {
        type: NodeType.ArrayExpression,
        elements,
        loc,
    };
}

/**
 * 创建元组表达式节点
 * @param elements 元组元素
 * @param loc 位置信息
 * @returns 元组表达式节点
 */
export function createTupleExpression(elements: Expression[], loc?: Node["loc"]): TupleExpression {
    return {
        type: NodeType.TupleExpression,
        elements,
        loc,
    };
}

/**
 * 创建对象表达式节点
 * @param properties 对象属性
 * @param loc 位置信息
 * @returns 对象表达式节点
 */
export function createObjectExpression(
    properties: ObjectProperty[],
    loc?: Node["loc"],
): ObjectExpression {
    return {
        type: NodeType.ObjectExpression,
        properties,
        loc,
    };
}

/**
 * 创建对象属性节点
 * @param key 属性键
 * @param value 属性值
 * @param shorthand 是否为简写属性
 * @param loc 位置信息
 * @returns 对象属性节点
 */
export function createObjectProperty(
    key: Identifier | Literal,
    value: Expression,
    shorthand: boolean,
    loc?: Node["loc"],
): ObjectProperty {
    return {
        type: NodeType.ObjectProperty,
        key,
        value,
        shorthand,
        loc,
    };
}

/**
 * 创建If语句节点
 * @param test 条件表达式
 * @param consequent 当条件为真时执行的语句
 * @param elseIfs else if 分支列表
 * @param alternate 当所有条件都为假时执行的语句
 * @param loc 位置信息
 * @returns If语句节点
 */
export function createIfStatement(
    test: Expression,
    consequent: Statement,
    elseIfs?: ElseIfClause[],
    alternate?: Statement,
    loc?: Node["loc"],
): IfStatement {
    return {
        type: NodeType.IfStatement,
        test,
        consequent,
        elseIfs,
        alternate,
        loc,
    };
}

/**
 * 创建Else If分支节点
 * @param test 条件表达式
 * @param consequent 当条件为真时执行的语句
 * @param loc 位置信息
 * @returns Else If分支节点
 */
export function createElseIfClause(
    test: Expression,
    consequent: Statement,
    loc?: Node["loc"],
): ElseIfClause {
    return {
        type: NodeType.ElseIfClause,
        test,
        consequent,
        loc,
    };
}

/**
 * 创建For循环语句节点
 * @param init 初始化表达式
 * @param test 条件表达式
 * @param update 增量表达式
 * @param body 循环体
 * @param loc 位置信息
 * @returns For循环语句节点
 */
export function createForStatement(
    init: VariableDeclaration | AssignmentStatement | Expression | null,
    test: Expression | null,
    update: Expression | null,
    body: Statement,
    loc?: Node["loc"],
): ForStatement {
    return {
        type: NodeType.ForStatement,
        init,
        test,
        update,
        body,
        loc,
    };
}

/**
 * 创建While循环语句节点
 * @param test 条件表达式
 * @param body 循环体
 * @param loc 位置信息
 * @returns While循环语句节点
 */
export function createWhileStatement(
    test: Expression,
    body: Statement,
    loc?: Node["loc"],
): WhileStatement {
    return {
        type: NodeType.WhileStatement,
        test,
        body,
        loc,
    };
}

/**
 * 创建For-In遍历循环语句节点
 * @param left 循环变量
 * @param right 被遍历的表达式
 * @param body 循环体
 * @param loc 位置信息
 * @returns For-In遍历循环语句节点
 */
export function createForInStatement(
    left: Identifier | [Identifier, Identifier],
    right: Expression,
    body: Statement,
    loc?: Node["loc"],
): ForInStatement {
    return {
        type: NodeType.ForInStatement,
        left,
        right,
        body,
        loc,
    };
}

/**
 * 创建匹配语句节点
 * @param discriminant 被匹配的表达式
 * @param cases 匹配分支列表
 * @param loc 位置信息
 * @returns 匹配语句节点
 */
export function createMatchStatement(
    discriminant: Expression,
    cases: MatchCase[],
    loc?: Node["loc"],
): MatchStatement {
    return {
        type: NodeType.MatchStatement,
        discriminant,
        cases,
        loc,
    };
}

/**
 * 创建匹配分支节点
 * @param pattern 匹配模式
 * @param consequent 匹配成功时执行的语句
 * @param loc 位置信息
 * @returns 匹配分支节点
 */
export function createMatchCase(
    pattern: Expression | null,
    consequent: Statement,
    loc?: Node["loc"],
): MatchCase {
    return {
        type: NodeType.MatchCase,
        pattern,
        consequent,
        loc,
    };
}

/**
 * 创建命名空间声明节点
 * @param name 命名空间名称
 * @param loc 位置信息
 * @returns 命名空间声明节点
 */
export function createNamespaceDeclaration(
    name: Identifier,
    loc?: Node["loc"],
): NamespaceDeclaration {
    return {
        type: NodeType.NamespaceDeclaration,
        name,
        loc,
    };
}

/**
 * 创建导入声明节点
 * @param modulePath 模块路径
 * @param loc 位置信息
 * @returns 导入声明节点
 */
export function createUsingDeclaration(
    modulePath: Identifier[],
    loc?: Node["loc"],
): UsingDeclaration {
    return {
        type: NodeType.UsingDeclaration,
        modulePath,
        loc,
    };
}

/**
 * 创建范围表达式节点
 * @param start 范围起始值
 * @param end 范围结束值
 * @param loc 位置信息
 * @returns 范围表达式节点
 */
export function createRangeExpression(
    start: Expression,
    end: Expression,
    loc?: Node["loc"],
): RangeExpression {
    return {
        type: NodeType.RangeExpression,
        start,
        end,
        loc,
    };
}

/**
 * 创建类型转换表达式节点
 * @param expression 要转换的表达式
 * @param typeAnnotation 目标类型
 * @param loc 位置信息
 * @returns 类型转换表达式节点
 */
export function createAsExpression(
    expression: Expression,
    typeAnnotation: Identifier,
    loc?: Node["loc"],
): AsExpression {
    return {
        type: NodeType.AsExpression,
        expression,
        typeAnnotation,
        loc,
    };
}

/**
 * 创建数组类型注解节点
 * @param elementType 元素类型
 * @param size 数组长度
 * @param loc 位置信息
 * @returns 数组类型注解节点
 */
export function createArrayTypeAnnotation(
    elementType: TypeAnnotation,
    size?: Literal,
    loc?: Node["loc"],
): ArrayTypeAnnotation {
    return {
        type: NodeType.ArrayTypeAnnotation,
        elementType,
        size,
        loc,
    };
}

/**
 * 创建元组类型注解节点
 * @param elementTypes 元素类型列表
 * @param loc 位置信息
 * @returns 元组类型注解节点
 */
export function createTupleTypeAnnotation(
    elementTypes: TypeAnnotation[],
    loc?: Node["loc"],
): TupleTypeAnnotation {
    return {
        type: NodeType.TupleTypeAnnotation,
        elementTypes,
        loc,
    };
}

/**
 * 创建带类型注解的参数节点
 * @param name 参数名称
 * @param typeAnnotation 参数类型
 * @param loc 位置信息
 * @returns 带类型注解的参数节点
 */
export function createTypedParameter(
    name: Identifier,
    typeAnnotation?: Identifier,
    loc?: Node["loc"],
): TypedParameter {
    return {
        type: NodeType.TypedParameter,
        name,
        typeAnnotation,
        loc,
    };
}

/**
 * 创建函数声明节点
 * @param name 函数名称
 * @param params 函数参数
 * @param body 函数体
 * @param loc 位置信息
 * @returns 函数声明节点
 */
export function createFunctionDeclaration(
    name: Identifier,
    params: Identifier[],
    body: BlockStatement,
    loc?: Node["loc"],
): FunctionDeclaration {
    return {
        type: NodeType.FunctionDeclaration,
        name,
        params,
        body,
        loc,
    };
}

/**
 * 创建赋值语句节点
 * @param left 左值表达式
 * @param right 右值表达式
 * @param loc 位置信息
 * @returns 赋值语句节点
 */
export function createAssignmentStatement(
    left: Expression,
    right: Expression,
    loc?: Node["loc"],
): AssignmentStatement {
    return {
        type: NodeType.AssignmentStatement,
        left,
        right,
        loc,
    };
}

/**
 * 创建注释节点
 * @param value 注释内容
 * @param block 是否为块注释
 * @param loc 位置信息
 * @returns 注释节点
 */
export function createComment(value: string, block: boolean, loc?: Node["loc"]): Comment {
    return {
        type: NodeType.Comment,
        value,
        block,
        loc,
    };
}

/**
 * 创建管道表达式节点
 * @param input 输入表达式
 * @param filter 过滤器名称
 * @param args 过滤器参数
 * @param loc 位置信息
 * @returns 管道表达式节点
 */
export function createPipeExpression(
    input: Expression,
    filter: Identifier,
    args: Expression[],
    loc?: Node["loc"],
): PipeExpression {
    return {
        type: NodeType.PipeExpression,
        input,
        filter,
        arguments: args,
        loc,
    };
}

/**
 * 创建 Include 语句节点
 * @param template 模板名称
 * @param context 传递的上下文
 * @param loc 位置信息
 * @returns Include 语句节点
 */
export function createIncludeStatement(
    template: Expression,
    context?: Expression,
    loc?: Node["loc"],
): IncludeStatement {
    return {
        type: NodeType.IncludeStatement,
        template,
        context,
        loc,
    };
}
