package dejavu.intellij.language.workspace

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.psi.PsiElementVisitor
import com.intellij.psi.PsiManager
import dejavu.intellij.language.file.DejavuFileNode
import dejavu.intellij.language.psi.fragments.ExtendsFragmentNode
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.locks.ReentrantReadWriteLock
import kotlin.concurrent.read
import kotlin.concurrent.write

/**
 * 跨文件 extends 解析服务
 * 用于解析 extends 语句引用的其他文件
 */
class ExtendsResolver(private val project: Project) {
    private val pathResolver = PathResolver.getInstance(project)
    private val extendsCache: MutableMap<VirtualFile, VirtualFile?> = ConcurrentHashMap()
    private val lock = ReentrantReadWriteLock()
    private val maxInheritanceDepth = 10

    /**
     * 解析指定文件的 extends 引用
     * @param file 要解析的文件
     * @return extends 引用的文件，如果不存在则返回 null
     */
    fun resolveExtends(file: VirtualFile?): VirtualFile? {
        if (file == null) return null

        lock.read {
            extendsCache[file]?.let { return it }
        }

        // 解析文件中的 extends 语句
        val extendsFile = parseExtends(file)
        lock.write {
            extendsCache[file] = extendsFile
        }
        return extendsFile
    }

    /**
     * 解析 extends 链
     * @param file 起始文件
     * @return extends 链，从起始文件到最终基文件
     */
    fun resolveExtendsChain(file: VirtualFile?): List<VirtualFile> {
        if (file == null) return emptyList()

        val chain = mutableListOf<VirtualFile>()
        val visited = mutableSetOf<VirtualFile>()
        var current: VirtualFile? = file

        var depth = 0
        while (current != null && depth < maxInheritanceDepth) {
            if (visited.contains(current)) {
                // 检测到循环依赖
                break
            }

            chain.add(current)
            visited.add(current)
            current = resolveExtends(current)
            depth++
        }

        return chain
    }

    /**
     * 解析文件中的 extends 语句
     * @param file 要解析的文件
     * @return extends 引用的文件，如果不存在则返回 null
     */
    private fun parseExtends(file: VirtualFile): VirtualFile? {
        val psiFile = PsiManager.getInstance(project).findFile(file) as? DejavuFileNode ?: return null
        var extendsPath: String? = null

        // 遍历文件中的所有 extends 语句
        psiFile.accept(object : PsiElementVisitor() {
            override fun visitElement(element: com.intellij.psi.PsiElement) {
                if (element is ExtendsFragmentNode) {
                    extendsPath = element.getExtendsPath()
                }
                super.visitElement(element)
            }
        })

        // 解析 extends 路径
        return extendsPath?.let { pathResolver.resolveTemplatePath(it, file) }
    }

    /**
     * 清除指定文件的 extends 缓存
     * @param file 要清除缓存的文件
     */
    fun clearExtendsCache(file: VirtualFile?) {
        if (file == null) return

        lock.write {
            extendsCache.remove(file)
        }
    }

    /**
     * 清除所有 extends 缓存
     */
    fun clearAllExtendsCaches() {
        lock.write {
            extendsCache.clear()
        }
    }

    companion object {
        /**
         * 获取 extends 解析服务的实例
         * @param project 项目
         * @return extends 解析服务实例
         */
        fun getInstance(project: Project): ExtendsResolver {
            return project.getService(ExtendsResolver::class.java)
        }
    }
}
