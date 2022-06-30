package dejavu.intellij.language.workspace

import com.intellij.openapi.project.Project

/**
 * Doki框架上下文关键词管理器
 * 负责识别和管理Doki框架的特殊上下文关键词
 */
class DokiContextManager(private val project: Project) {

    /**
     * 上下文关键词定义
     */
    data class ContextKeyword(
        val name: String,
        val properties: Set<String>
    )

    /**
     * 支持的上下文关键词列表
     */
    private val contextKeywords = setOf(
        ContextKeyword("post", setOf("title", "date", "slug", "excerpt", "content")),
        ContextKeyword("posts", emptySet()), // 集合类型，没有直接属性
        ContextKeyword("title", emptySet()), // 简单类型，没有属性
        ContextKeyword("year", emptySet()), // 简单类型，没有属性
        ContextKeyword("page", setOf("title", "content", "url", "id", "metadata"))
    )

    /**
     * 检查是否是上下文关键词
     * @param keyword 关键词
     * @return 是否是上下文关键词
     */
    fun isContextKeyword(keyword: String): Boolean {
        return contextKeywords.any { it.name == keyword }
    }

    /**
     * 获取上下文关键词的属性列表
     * @param keyword 关键词
     * @return 属性列表，如果不是上下文关键词则返回空集合
     */
    fun getContextKeywordProperties(keyword: String): Set<String> {
        return contextKeywords.find { it.name == keyword }?.properties ?: emptySet()
    }

    /**
     * 检查上下文关键词是否具有指定属性
     * @param keyword 关键词
     * @param property 属性名
     * @return 是否具有该属性
     */
    fun hasContextKeywordProperty(keyword: String, property: String): Boolean {
        return getContextKeywordProperties(keyword).contains(property)
    }

    /**
     * 获取所有上下文关键词
     * @return 上下文关键词列表
     */
    fun getAllContextKeywords(): Set<String> {
        return contextKeywords.map { it.name }.toSet()
    }

    companion object {
        /**
         * 获取Doki上下文关键词管理器实例
         * @param project 项目
         * @return Doki上下文关键词管理器实例
         */
        fun getInstance(project: Project): DokiContextManager {
            return project.getService(DokiContextManager::class.java)
        }
    }
}
