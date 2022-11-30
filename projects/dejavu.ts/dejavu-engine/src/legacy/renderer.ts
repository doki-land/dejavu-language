/**
 * DejaVu 模板引擎渲染器
 * 负责将 AST 转换为最终输出
 */
import {
    NodeType,
    Program,
    Statement,
    Expression,
    Text,
    IfStatement,
    ForStatement,
    ForInStatement,
    WhileStatement,
    BlockStatement,
    VariableDeclaration,
    FunctionDeclaration,
    AssignmentStatement,
    ExpressionStatement,
    Identifier,
    Literal,
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    MemberExpression,
    ArrayExpression,
    TupleExpression,
    ObjectExpression,
    TemplateLiteral,
    TemplateExpression,
    RangeExpression,
    AsExpression,
    MatchStatement,
    MatchCase,
    NamespaceDeclaration,
    UsingDeclaration,
    ElseIfClause,
    BreakStatement,
    ContinueStatement,
    ExtendsStatement,
    BlockDeclaration,
    PrependStatement,
    AppendStatement,
    SuperExpression,
    PipeExpression,
    IncludeStatement,
    Comment,
} from "./ast";
import {FilterRegistry, Filter, filterRegistry} from "./filter";
import {RecursionDepthError, LoopIterationError} from "./error-types";
import {TemplateConfig, DEFAULT_TEMPLATE_CONFIG} from "./language";
import {htmlEscape} from "./escape";

/**
 * 原始值包装类
 * 用于标记不需要 HTML 转义的值
 */
export class RawValue {
    /** 原始值内容 */
    public readonly value: string;

    /**
     * 创建原始值包装
     * @param value 原始字符串值
     */
    constructor(value: string) {
        this.value = value;
    }
}

/**
 * 创建原始值（不进行 HTML 转义）
 * @param value 字符串值
 * @returns 原始值包装对象
 */
export function raw(value: string): RawValue {
    return new RawValue(value);
}

/**
 * 控制流异常 - Break
 */
class BreakException extends Error {
    constructor() {
        super("break");
        this.name = "BreakException";
    }
}

/**
 * 控制流异常 - Continue
 */
class ContinueException extends Error {
    constructor() {
        super("continue");
        this.name = "ContinueException";
    }
}

/**
 * 渲染上下文接口
 */
export interface RenderContext {
    /** 变量作用域 */
    scope: Record<string, any>;
    /** 父上下文 */
    parent?: RenderContext;
    /** 当前递归深度 */
    recursionDepth: number;
    /** 最大递归深度 */
    maxRecursionDepth: number;
    /** 最大循环迭代次数 */
    maxLoopIterations: number;
}

/**
 * 渲染器配置接口
 */
export interface RendererConfig {
    /** 是否启用 HTML 自动转义 */
    autoEscape: boolean;
    /** 模板配置 */
    template?: TemplateConfig;
    /** 模板注册表 */
    templateRegistry?: Map<string, Program>;
}

/**
 * 默认渲染器配置
 */
const DEFAULT_RENDERER_CONFIG: RendererConfig = {
    autoEscape: true,
    template: DEFAULT_TEMPLATE_CONFIG,
};

/**
 * DejaVu 渲染器类
 */
export class DejavuRenderer {
    /** 渲染器配置 */
    private readonly config: RendererConfig;

    /**
     * 创建渲染器实例
     * @param config 渲染器配置
     */
    constructor(config: Partial<RendererConfig> = {}) {
        this.config = {...DEFAULT_RENDERER_CONFIG, ...config};
    }

    /**
     * 格式化输出值
     * @param value 要格式化的值
     * @returns 格式化后的字符串
     */
    private formatValue(value: any): string {
        if (value === null || value === undefined) {
            return "";
        }
        if (value instanceof RawValue) {
            return value.value;
        }
        const strValue = String(value);
        if (this.config.autoEscape) {
            return htmlEscape(strValue);
        }
        return strValue;
    }

    /**
     * 创建渲染上下文
     * @param initialScope 初始作用域
     * @returns 渲染上下文
     */
    private createContext(initialScope: Record<string, any> = {}): RenderContext {
        return {
            scope: {...initialScope},
            recursionDepth: 0,
            maxRecursionDepth: this.config.template.maxRecursionDepth,
            maxLoopIterations: this.config.template.maxLoopIterations,
        };
    }

    /**
     * 创建子上下文（用于函数调用等场景）
     * @param parent 父上下文
     * @returns 子渲染上下文
     */
    private createChildContext(parent: RenderContext): RenderContext {
        const newDepth = parent.recursionDepth + 1;
        if (newDepth > parent.maxRecursionDepth) {
            throw new RecursionDepthError(newDepth, parent.maxRecursionDepth);
        }
        return {
            scope: {},
            parent: parent,
            recursionDepth: newDepth,
            maxRecursionDepth: parent.maxRecursionDepth,
            maxLoopIterations: parent.maxLoopIterations,
        };
    }

    /**
     * 查找变量
     * @param context 渲染上下文
     * @param name 变量名
     * @returns 变量值
     */
    private findVariable(context: RenderContext, name: string): any {
        if (context.scope.hasOwnProperty(name)) {
            return context.scope[name];
        }
        if (context.parent) {
            return this.findVariable(context.parent, name);
        }
        return undefined;
    }

    /**
     * 设置变量
     * @param context 渲染上下文
     * @param name 变量名
     * @param value 变量值
     */
    private setVariable(context: RenderContext, name: string, value: any): void {
        context.scope[name] = value;
    }

    /**
     * 执行二元运算
     * @param left 左操作数
     * @param operator 运算符
     * @param right 右操作数
     * @returns 运算结果
     */
    private executeBinaryOperation(left: any, operator: string, right: any): any {
        switch (operator) {
            case "+":
                return left + right;
            case "-":
                return left - right;
            case "*":
                return left * right;
            case "/":
                return left / right;
            case "%":
                return left % right;
            case "==":
                return left == right;
            case "===":
                return left === right;
            case "!=":
                return left != right;
            case "!==":
                return left !== right;
            case "<":
                return left < right;
            case "<=":
                return left <= right;
            case ">":
                return left > right;
            case ">=":
                return left >= right;
            case "&&":
                return left && right;
            case "||":
                return left || right;
            default:
                throw new Error(`Unknown operator: ${operator}`);
        }
    }

    /**
     * 执行一元运算
     * @param operator 运算符
     * @param argument 操作数
     * @returns 运算结果
     */
    private executeUnaryOperation(operator: string, argument: any): any {
        switch (operator) {
            case "!":
                return !argument;
            case "-":
                return -argument;
            case "+":
                return +argument;
            case "typeof":
                return typeof argument;
            case "void":
                return undefined;
            default:
                throw new Error(`Unknown operator: ${operator}`);
        }
    }

    /**
     * 评估表达式
     * @param node 表达式节点
     * @param context 渲染上下文
     * @returns 表达式结果
     */
    private evaluateExpression(node: Expression, context: RenderContext): any {
        switch (node.type) {
            case NodeType.Identifier:
                return this.findVariable(context, (node as Identifier).name);

            case NodeType.Literal:
                return (node as Literal).value;

            case NodeType.BinaryExpression:
                const binaryExpr = node as BinaryExpression;
                const leftValue = this.evaluateExpression(binaryExpr.left, context);
                const rightValue = this.evaluateExpression(binaryExpr.right, context);
                return this.executeBinaryOperation(leftValue, binaryExpr.operator, rightValue);

            case NodeType.UnaryExpression:
                const unaryExpr = node as UnaryExpression;
                const argValue = this.evaluateExpression(unaryExpr.argument, context);
                return this.executeUnaryOperation(unaryExpr.operator, argValue);

            case NodeType.CallExpression:
                const callExpr = node as CallExpression;
                const callee = this.evaluateExpression(callExpr.callee, context);
                if (typeof callee !== "function") {
                    throw new Error("Callee is not a function");
                }
                const args = callExpr.arguments.map((arg) => this.evaluateExpression(arg, context));
                return callee(...args);

            case NodeType.MemberExpression:
                const memberExpr = node as MemberExpression;
                const object = this.evaluateExpression(memberExpr.object, context);
                const property = memberExpr.computed
                    ? this.evaluateExpression(memberExpr.property, context)
                    : (memberExpr.property as any).name;
                return object[property];

            case NodeType.ArrayExpression:
                const arrayExpr = node as ArrayExpression;
                return arrayExpr.elements.map((element) =>
                    this.evaluateExpression(element, context),
                );

            case NodeType.TupleExpression:
                const tupleExpr = node as TupleExpression;
                return tupleExpr.elements.map((element) =>
                    this.evaluateExpression(element, context),
                );

            case NodeType.ObjectExpression:
                const objectExpr = node as ObjectExpression;
                const obj: Record<string, any> = {};
                objectExpr.properties.forEach((prop) => {
                    const key =
                        prop.key.type === NodeType.Identifier
                            ? (prop.key as Identifier).name
                            : (prop.key as Literal).value;
                    const value = this.evaluateExpression(prop.value, context);
                    obj[key as string] = value;
                });
                return obj;

            case NodeType.TemplateLiteral:
                const templateLiteral = node as TemplateLiteral;
                let templateResult = "";
                templateLiteral.quasis.forEach((quasi, index) => {
                    templateResult += quasi.value.cooked || quasi.value.raw;
                    if (index < templateLiteral.expressions.length) {
                        const exprValue = this.evaluateExpression(
                            templateLiteral.expressions[index],
                            context,
                        );
                        templateResult += String(exprValue);
                    }
                });
                return templateResult;

            case NodeType.TemplateExpression:
                const templateExpr = node as TemplateExpression;
                return templateExpr.content;

            case NodeType.RangeExpression:
                const rangeExpr = node as RangeExpression;
                const startValue = this.evaluateExpression(rangeExpr.start, context);
                const endValue = this.evaluateExpression(rangeExpr.end, context);
                const start = Math.floor(Number(startValue));
                const end = Math.floor(Number(endValue));
                const result: number[] = [];
                const rangeLength = Math.abs(end - start) + 1;
                if (rangeLength > context.maxLoopIterations) {
                    throw new LoopIterationError(rangeLength, context.maxLoopIterations);
                }
                if (start <= end) {
                    for (let i = start; i <= end; i++) {
                        result.push(i);
                    }
                } else {
                    for (let i = start; i >= end; i--) {
                        result.push(i);
                    }
                }
                return result;

            case NodeType.AsExpression:
                const asExpr = node as AsExpression;
                const asValue = this.evaluateExpression(asExpr.expression, context);
                return asValue;

            case NodeType.SuperExpression:
                return "";

            case NodeType.PipeExpression:
                const pipeExpr = node as PipeExpression;
                const inputValue = this.evaluateExpression(pipeExpr.input, context);
                const filterName = pipeExpr.filter.name;
                const filterArgs = pipeExpr.arguments.map((arg) =>
                    this.evaluateExpression(arg, context),
                );

                const customFilters = this.config.template?.customFilters;
                if (customFilters && customFilters[filterName]) {
                    return customFilters[filterName](inputValue, filterArgs);
                }

                return filterRegistry.apply(filterName, inputValue, filterArgs);

            default:
                throw new Error(`Unknown expression type: ${node.type}`);
        }
    }

    /**
     * 渲染语句
     * @param node 语句节点
     * @param context 渲染上下文
     * @returns 渲染结果
     */
    private renderStatement(node: Statement, context: RenderContext): string {
        switch (node.type) {
            case NodeType.ExpressionStatement:
                const exprStmt = node as ExpressionStatement;
                const value = this.evaluateExpression(exprStmt.expression, context);
                return this.formatValue(value);

            case NodeType.AssignmentStatement:
                const assignStmt = node as AssignmentStatement;
                if (assignStmt.left.type === NodeType.Identifier) {
                    const name = (assignStmt.left as Identifier).name;
                    const value = this.evaluateExpression(assignStmt.right, context);
                    this.setVariable(context, name, value);
                }
                return "";

            case NodeType.VariableDeclaration:
                const varDecl = node as VariableDeclaration;
                const varName = varDecl.name.name;
                const initValue = varDecl.init
                    ? this.evaluateExpression(varDecl.init, context)
                    : undefined;
                this.setVariable(context, varName, initValue);
                return "";

            case NodeType.FunctionDeclaration:
                const funcDecl = node as FunctionDeclaration;
                const funcName = funcDecl.name.name;
                const func = (...args: any[]) => {
                    const funcContext = this.createChildContext(context);
                    funcDecl.params.forEach((param, index) => {
                        funcContext.scope[param.name] = args[index];
                    });
                    return this.renderBlockStatement(funcDecl.body, funcContext);
                };
                this.setVariable(context, funcName, func);
                return "";

            case NodeType.IfStatement:
                const ifStmt = node as IfStatement;
                const ifCondition = this.evaluateExpression(ifStmt.test, context);
                if (ifCondition) {
                    return this.renderStatement(ifStmt.consequent, context);
                } else if (ifStmt.elseIfs && ifStmt.elseIfs.length > 0) {
                    for (const elseIf of ifStmt.elseIfs) {
                        const elseIfCondition = this.evaluateExpression(elseIf.test, context);
                        if (elseIfCondition) {
                            return this.renderStatement(elseIf.consequent, context);
                        }
                    }
                }
                if (ifStmt.alternate) {
                    return this.renderStatement(ifStmt.alternate, context);
                }
                return "";

            case NodeType.ForStatement:
                const forStmt = node as ForStatement;
                let forResult = "";
                let forIterations = 0;
                if (forStmt.init) {
                    if (
                        forStmt.init.type === NodeType.VariableDeclaration ||
                        forStmt.init.type === NodeType.AssignmentStatement
                    ) {
                        this.renderStatement(forStmt.init as Statement, context);
                    } else {
                        this.evaluateExpression(forStmt.init as Expression, context);
                    }
                }
                try {
                    while (!forStmt.test || this.evaluateExpression(forStmt.test, context)) {
                        forIterations++;
                        if (forIterations > context.maxLoopIterations) {
                            throw new LoopIterationError(forIterations, context.maxLoopIterations);
                        }
                        try {
                            forResult += this.renderStatement(forStmt.body, context);
                        } catch (e) {
                            if (e instanceof ContinueException) {
                                // 继续下一次迭代
                            } else if (e instanceof BreakException) {
                                break;
                            } else {
                                throw e;
                            }
                        }
                        if (forStmt.update) {
                            this.evaluateExpression(forStmt.update, context);
                        }
                    }
                } catch (e) {
                    if (!(e instanceof BreakException)) {
                        throw e;
                    }
                }
                return forResult;

            case NodeType.WhileStatement:
                const whileStmt = node as WhileStatement;
                let whileResult = "";
                let whileIterations = 0;
                try {
                    while (this.evaluateExpression(whileStmt.test, context)) {
                        whileIterations++;
                        if (whileIterations > context.maxLoopIterations) {
                            throw new LoopIterationError(
                                whileIterations,
                                context.maxLoopIterations,
                            );
                        }
                        try {
                            whileResult += this.renderStatement(whileStmt.body, context);
                        } catch (e) {
                            if (e instanceof ContinueException) {
                                // 继续下一次迭代
                            } else if (e instanceof BreakException) {
                                break;
                            } else {
                                throw e;
                            }
                        }
                    }
                } catch (e) {
                    if (!(e instanceof BreakException)) {
                        throw e;
                    }
                }
                return whileResult;

            case NodeType.ForInStatement:
                const forInStmt = node as ForInStatement;
                let forInResult = "";
                let forInIterations = 0;
                const iterable = this.evaluateExpression(forInStmt.right, context);
                if (Array.isArray(iterable)) {
                    try {
                        for (let i = 0; i < iterable.length; i++) {
                            forInIterations++;
                            if (forInIterations > context.maxLoopIterations) {
                                throw new LoopIterationError(
                                    forInIterations,
                                    context.maxLoopIterations,
                                );
                            }
                            if (Array.isArray(forInStmt.left)) {
                                this.setVariable(context, forInStmt.left[0].name, i);
                                this.setVariable(context, forInStmt.left[1].name, iterable[i]);
                            } else {
                                this.setVariable(context, forInStmt.left.name, iterable[i]);
                            }
                            try {
                                forInResult += this.renderStatement(forInStmt.body, context);
                            } catch (e) {
                                if (e instanceof ContinueException) {
                                    // 继续下一次迭代
                                } else if (e instanceof BreakException) {
                                    break;
                                } else {
                                    throw e;
                                }
                            }
                        }
                    } catch (e) {
                        if (!(e instanceof BreakException)) {
                            throw e;
                        }
                    }
                }
                return forInResult;

            case NodeType.MatchStatement:
                const matchStmt = node as MatchStatement;
                const discriminantValue = this.evaluateExpression(matchStmt.discriminant, context);
                for (const matchCase of matchStmt.cases) {
                    if (matchCase.pattern === null) {
                        return this.renderStatement(matchCase.consequent, context);
                    }
                    const patternValue = this.evaluateExpression(matchCase.pattern, context);
                    if (discriminantValue === patternValue) {
                        return this.renderStatement(matchCase.consequent, context);
                    }
                }
                return "";

            case NodeType.NamespaceDeclaration:
                return "";

            case NodeType.UsingDeclaration:
                return "";

            case NodeType.BlockStatement:
                return this.renderBlockStatement(node as BlockStatement, context);

            case NodeType.BreakStatement:
                throw new BreakException();

            case NodeType.ContinueStatement:
                throw new ContinueException();

            case NodeType.ExtendsStatement:
                return "";

            case NodeType.BlockDeclaration:
                const blockDecl = node as BlockDeclaration;
                return this.renderBlockStatement(
                    {type: NodeType.BlockStatement, body: blockDecl.body} as BlockStatement,
                    context,
                );

            case NodeType.PrependStatement:
                return "";

            case NodeType.AppendStatement:
                return "";

            case NodeType.IncludeStatement:
                const includeStmt = node as IncludeStatement;
                const templateName = this.evaluateExpression(includeStmt.template, context);
                const template = this.config.templateRegistry?.get(String(templateName));
                if (!template) {
                    throw new Error(`Template not found: ${templateName}`);
                }
                const includeContext = includeStmt.context
                    ? {...context.scope, ...this.evaluateExpression(includeStmt.context, context)}
                    : context.scope;
                return this.render(template, includeContext);

            default:
                throw new Error(`Unknown statement type: ${node.type}`);
        }
    }

    /**
     * 渲染块语句
     * @param node 块语句节点
     * @param context 渲染上下文
     * @returns 渲染结果
     */
    private renderBlockStatement(node: BlockStatement, context: RenderContext): string {
        let result = "";
        for (const item of node.body) {
            if (item.type === NodeType.Comment) {
                continue;
            } else if (item.type === NodeType.Text) {
                result += (item as Text).value;
            } else {
                const nodeType = item.type;
                if (
                    nodeType === NodeType.ExpressionStatement ||
                    nodeType === NodeType.AssignmentStatement ||
                    nodeType === NodeType.VariableDeclaration ||
                    nodeType === NodeType.FunctionDeclaration ||
                    nodeType === NodeType.IfStatement ||
                    nodeType === NodeType.ForStatement ||
                    nodeType === NodeType.ForInStatement ||
                    nodeType === NodeType.WhileStatement ||
                    nodeType === NodeType.MatchStatement ||
                    nodeType === NodeType.NamespaceDeclaration ||
                    nodeType === NodeType.UsingDeclaration ||
                    nodeType === NodeType.BlockStatement
                ) {
                    result += this.renderStatement(item as Statement, context);
                } else {
                    const value = this.evaluateExpression(item as Expression, context);
                    result += this.formatValue(value);
                }
            }
        }
        return result;
    }

    /**
     * 渲染程序
     * @param node 程序节点
     * @param context 渲染上下文
     * @returns 渲染结果
     */
    private renderProgram(node: Program, context: RenderContext): string {
        let result = "";
        for (const item of node.body) {
            if (item.type === NodeType.Comment) {
                continue;
            } else if (item.type === NodeType.Text) {
                result += (item as Text).value;
            } else {
                const nodeType = item.type;
                if (
                    nodeType === NodeType.ExpressionStatement ||
                    nodeType === NodeType.AssignmentStatement ||
                    nodeType === NodeType.VariableDeclaration ||
                    nodeType === NodeType.FunctionDeclaration ||
                    nodeType === NodeType.IfStatement ||
                    nodeType === NodeType.ForStatement ||
                    nodeType === NodeType.ForInStatement ||
                    nodeType === NodeType.WhileStatement ||
                    nodeType === NodeType.MatchStatement ||
                    nodeType === NodeType.NamespaceDeclaration ||
                    nodeType === NodeType.UsingDeclaration ||
                    nodeType === NodeType.BlockStatement
                ) {
                    result += this.renderStatement(item as Statement, context);
                } else {
                    const value = this.evaluateExpression(item as Expression, context);
                    result += this.formatValue(value);
                }
            }
        }
        return result;
    }

    /**
     * 渲染 AST
     * @param ast 抽象语法树
     * @param initialScope 初始作用域
     * @returns 渲染结果
     */
    public render(ast: Program, initialScope: Record<string, any> = {}): string {
        const context = this.createContext(initialScope);
        return this.renderProgram(ast, context);
    }
}

/**
 * 渲染器实例
 */
export const renderer = new DejavuRenderer();
