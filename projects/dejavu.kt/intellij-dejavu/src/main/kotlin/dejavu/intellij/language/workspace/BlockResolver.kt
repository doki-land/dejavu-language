package dejavu.intellij.language.workspace

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.psi.PsiElementVisitor
import com.intellij.psi.PsiManager
import dejavu.intellij.language.file.DejavuFileNode
import dejavu.intellij.language.psi.DejavuTypes
import dejavu.intellij.language.psi.fragments.BlockFragmentNode
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.locks.ReentrantReadWriteLock
import kotlin.concurrent.read
import kotlin.concurrent.write

/**
 * 跨文件 block 解析服务
 * 用于从其他文件中解析 block 定义
 */
class BlockResolver(private val project: Project) {
    private val pathResolver = PathResolver.getInstance(project)
    private val blockCache: MutableMap<VirtualFile, Map<String, BlockFragmentNode>> = ConcurrentHashMap()
    private val lock = ReentrantReadWriteLock()

    /**
     * 解析指定文件中定义的所有 block
     * @param file 要解析的文件
     * @return block 名称到 block 定义的映射
     */
    fun resolveBlocks(file: VirtualFile): Map<String, BlockFragmentNode> {
        lock.read {
            blockCache[file]?.let { return it }
        }

        // 解析文件中的 block
        val blocks = parseBlocks(file)
        lock.write {
            blockCache[file] = blocks
        }
        return blocks
    }

    /**
     * 从指定文件中解析 block
     * @param filePath 文件路径
     * @param contextFile 上下文文件
     * @return block 名称到 block 定义的映射
     */
    fun resolveBlocksFromPath(filePath: String, contextFile: VirtualFile): Map<String, BlockFragmentNode> {
        val resolvedFile = pathResolver.resolveTemplatePath(filePath, contextFile) ?: return emptyMap()
        return resolveBlocks(resolvedFile)
    }

    /**
     * 解析文件中的 block
     * @param file 要解析的文件
     * @return block 名称到 block 定义的映射
     */
    private fun parseBlocks(file: VirtualFile): Map<String, BlockFragmentNode> {
        val psiFile = PsiManager.getInstance(project).findFile(file) as? DejavuFileNode ?: return emptyMap()
        val blocks = mutableMapOf<String, BlockFragmentNode>()

        // 遍历文件中的所有 block
        psiFile.accept(object : PsiElementVisitor() {
            override fun visitElement(element: com.intellij.psi.PsiElement) {
                if (element is BlockFragmentNode) {
                    val blockName = element.getBlockName()
                    if (blockName != null) {
                        blocks[blockName] = element
                    }
                }
                super.visitElement(element)
            }
        })

        return blocks
    }

    /**
     * 获取 block 的名称
     * @param blockFragment block fragment
     * @return block 名称
     */
    private fun getBlockName(blockFragment: BlockFragmentNode): String? {
        val children = blockFragment.children
        for (child in children) {
            if (child.node.elementType == DejavuTypes.IDENTIFIER) {
                return child.text
            }
        }
        return null
    }

    /**
     * 清除指定文件的 block 缓存
     * @param file 要清除缓存的文件
     */
    fun clearBlockCache(file: VirtualFile) {
        lock.write {
            blockCache.remove(file)
        }
    }

    /**
     * 清除所有 block 缓存
     */
    fun clearAllBlockCaches() {
        lock.write {
            blockCache.clear()
        }
    }

    companion object {
        /**
         * 获取 block 解析服务的实例
         * @param project 项目
         * @return block 解析服务实例
         */
        fun getInstance(project: Project): BlockResolver {
            return project.getService(BlockResolver::class.java)
        }
    }
}
