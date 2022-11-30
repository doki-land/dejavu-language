import {engine} from "./engine";
import type {DejavuLanguage} from "./language";

/** 注册模板 */
export function registerTemplate(name: string, source: string, language?: DejavuLanguage): void {
    engine.registerTemplate(name, source, language);
}

/** 渲染模板 */
export function renderTemplate(name: string, context: Record<string, unknown> = {}): string {
    return engine.renderTemplate(name, context);
}

/** 检查模板 */
export function checkTemplate(source: string, language?: DejavuLanguage): void {
    engine.checkTemplate(source, language);
}
