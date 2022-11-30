package dejavu.intellij.language.workspace

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import java.io.File

/**
 * 智能路径解析服务
 * 用于解析模板文件的路径，支持相对路径和绝对路径
 */
class PathResolver(private val project: Project) {
    private val detector = FrameworkConfigDetector.getInstance(project)
    private val indexer = WorkspaceFileIndexer.getInstance(project)
    private val templateExtensions = listOf(".dejavu", ".doki")

    /**
     * 解析模板文件路径
     * @param referencePath 引用路径
     * @param contextFile 上下文文件
     * @return 解析后的文件，如果不存在则返回 null
     */
    fun resolveTemplatePath(referencePath: String, contextFile: VirtualFile): VirtualFile? {
        // 检测工作区根目录
        val workspaceRoot = detector.detectWorkspaceRoot(contextFile) ?: return null

        // 规范化路径
        val normalizedPath = normalizePath(referencePath)

        // 尝试解析路径
        return resolveRelativePath(normalizedPath, contextFile, workspaceRoot)
            ?: resolveAbsolutePath(normalizedPath, workspaceRoot)
    }

    /**
     * 规范化路径
     * @param path 原始路径
     * @return 规范化后的路径
     */
    private fun normalizePath(path: String): String {
        // 移除引号
        val unquotedPath = path.removeSurrounding("\"", "\"")
            .removeSurrounding("'", "'")

        // 规范化路径分隔符
        return unquotedPath.replace("\\", "/")
    }

    /**
     * 解析相对路径
     * @param path 相对路径
     * @param contextFile 上下文文件
     * @param workspaceRoot 工作区根目录
     * @return 解析后的文件，如果不存在则返回 null
     */
    private fun resolveRelativePath(path: String, contextFile: VirtualFile, workspaceRoot: VirtualFile): VirtualFile? {
        // 从上下文文件所在目录开始解析
        val contextDir = contextFile.parent ?: return null

        // 构建完整路径
        val fullPath = buildRelativePath(contextDir, path)

        // 尝试解析带扩展名和不带扩展名的路径
        return findTemplateFile(fullPath, workspaceRoot)
            ?: findTemplateFileWithExtensions(fullPath, workspaceRoot)
    }

    /**
     * 解析绝对路径
     * @param path 绝对路径
     * @param workspaceRoot 工作区根目录
     * @return 解析后的文件，如果不存在则返回 null
     */
    private fun resolveAbsolutePath(path: String, workspaceRoot: VirtualFile): VirtualFile? {
        // 绝对路径从工作区根目录开始
        val fullPath = if (path.startsWith("/")) path.substring(1) else path

        // 尝试解析带扩展名和不带扩展名的路径
        return findTemplateFile(fullPath, workspaceRoot)
            ?: findTemplateFileWithExtensions(fullPath, workspaceRoot)
    }

    /**
     * 构建相对路径
     * @param baseDir 基础目录
     * @param relativePath 相对路径
     * @return 构建后的路径
     */
    private fun buildRelativePath(baseDir: VirtualFile, relativePath: String): String {
        val basePath = baseDir.path
        val fullPath = File(basePath, relativePath).normalize().path
        return fullPath
    }

    /**
     * 查找模板文件
     * @param path 文件路径
     * @param workspaceRoot 工作区根目录
     * @return 找到的文件，如果不存在则返回 null
     */
    private fun findTemplateFile(path: String, workspaceRoot: VirtualFile): VirtualFile? {
        val templateFiles = indexer.getTemplateFiles(workspaceRoot)

        // 尝试直接匹配路径
        templateFiles.values.find { it.path == path }?.let { return it }

        // 尝试匹配相对路径
        val relativePath = path.substringAfter(workspaceRoot.path).removePrefix("/")
        return templateFiles[relativePath]
    }

    /**
     * 尝试添加扩展名后查找模板文件
     * @param path 不带扩展名的文件路径
     * @param workspaceRoot 工作区根目录
     * @return 找到的文件，如果不存在则返回 null
     */
    private fun findTemplateFileWithExtensions(path: String, workspaceRoot: VirtualFile): VirtualFile? {
        val templateFiles = indexer.getTemplateFiles(workspaceRoot)

        // 尝试添加不同的扩展名
        for (extension in templateExtensions) {
            val pathWithExt = if (path.endsWith(extension)) path else "$path$extension"

            // 尝试直接匹配路径
            templateFiles.values.find { it.path == pathWithExt }?.let { return it }

            // 尝试匹配相对路径
            val relativePath = pathWithExt.substringAfter(workspaceRoot.path).removePrefix("/")
            templateFiles[relativePath]?.let { return it }
        }

        return null
    }

    companion object {
        /**
         * 获取路径解析服务的实例
         * @param project 项目
         * @return 路径解析服务实例
         */
        fun getInstance(project: Project): PathResolver {
            return project.getService(PathResolver::class.java)
        }
    }
}
