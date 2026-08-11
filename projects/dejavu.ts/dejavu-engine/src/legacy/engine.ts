/**
 * DejaVu Engine 类
 * 提供模板管理和渲染功能
 */
import { DejavuRenderer, RendererConfig } from "./renderer";
import { parse } from "./parser";
import type { Program } from "./ast";
import type { DejavuLanguage } from "./language";

/**
 * 模板条目接口
 */
interface TemplateEntry {
    /** 解析后的 AST */
    program: Program;
    /** 渲染器配置 */
    rendererConfig: RendererConfig;
}

export class DejavuEngine {
    /** 模板存储映射 */
    private readonly templates: Map<string, TemplateEntry> = new Map();
    /** 模板 AST 注册表（用于 include） */
    private readonly templateRegistry: Map<string, Program> = new Map();

    /**
     * 从语言配置创建渲染器配置
     * @param language 语言配置
     * @returns 渲染器配置
     */
    private createRendererConfig(language?: DejavuLanguage): RendererConfig {
        return {
            autoEscape: language?.template.autoEscape ?? true,
            template: language?.template,
            templateRegistry: this.templateRegistry,
        };
    }

    /**
     * 注册模板
     * @param name 模板名称
     * @param source 模板源代码
     * @param language 语言配置
     */
    public registerTemplate(name: string, source: string, language?: DejavuLanguage): void {
        const program = parse(source, language);
        const rendererConfig = this.createRendererConfig(language);
        this.templates.set(name, { program, rendererConfig });
        this.templateRegistry.set(name, program);
    }

    /**
     * 渲染已注册的模板
     * @param name 模板名称
     * @param context 模板上下文参数
     * @returns 渲染结果
     */
    public renderTemplate(name: string, context: Record<string, any> = {}): string {
        const entry = this.templates.get(name);

        if (!entry) {
            throw new Error(`Template "${name}" not found`);
        }

        const renderer = new DejavuRenderer(entry.rendererConfig);
        return renderer.render(entry.program, context);
    }

    /**
     * 检查模板
     * @param source 源代码路径
     * @param language 语言配置
     */
    public checkTemplate(source: string, language?: DejavuLanguage): Program {
        return parse(source, language);
    }
}

/**
 * 引擎单例实例
 */
export const engine = new DejavuEngine();
