package dejavu.intellij.language.workspace

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.openapi.vfs.VirtualFileManager
import com.intellij.psi.search.FilenameIndex
import com.intellij.psi.search.GlobalSearchScope
import java.util.*
import java.util.concurrent.ConcurrentHashMap

/**
 * 框架配置文件检测服务
 * 用于识别 Dejavu.toml 和 Doki.toml 文件，并确定工作区根目录
 */
class FrameworkConfigDetector(private val project: Project) {
    private val cache: MutableMap<VirtualFile, VirtualFile> = ConcurrentHashMap()
    private val frameworkConfigNames = setOf("dejavu.toml", "doki.toml")

    /**
     * 检测给定文件所在的工作区根目录
     * @param file 要检测的文件
     * @return 工作区根目录，如果不存在则返回 null
     */
    fun detectWorkspaceRoot(file: VirtualFile): VirtualFile? {
        // 检查缓存
        cache[file]?.let { return it }

        // 从当前文件开始向上查找
        var current: VirtualFile? = file
        while (current != null) {
            // 检查当前目录是否包含框架配置文件
            val configFile = findFrameworkConfigFile(current)
            if (configFile != null) {
                val workspaceRoot = configFile.parent
                cache[file] = workspaceRoot
                return workspaceRoot
            }
            current = current.parent
        }

        // 不缓存未找到的结果
        return null
    }

    /**
     * 在指定目录中查找框架配置文件
     * @param directory 要搜索的目录
     * @return 找到的配置文件，如果不存在则返回 null
     */
    private fun findFrameworkConfigFile(directory: VirtualFile): VirtualFile? {
        if (!directory.isDirectory) return null

        // 遍历目录中的文件
        directory.children.forEach { child ->
            if (child.isValid && child.name.lowercase(Locale.ROOT) in frameworkConfigNames) {
                return child
            }
        }

        return null
    }

    /**
     * 清除缓存
     */
    fun clearCache() {
        cache.clear()
    }

    companion object {
        /**
         * 获取框架配置文件检测服务的实例
         * @param project 项目
         * @return 框架配置文件检测服务实例
         */
        fun getInstance(project: Project): FrameworkConfigDetector {
            return project.getService(FrameworkConfigDetector::class.java)
        }
    }
}
