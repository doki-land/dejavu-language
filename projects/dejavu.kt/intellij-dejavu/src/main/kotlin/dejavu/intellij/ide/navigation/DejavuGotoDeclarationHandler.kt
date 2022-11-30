package dejavu.intellij.ide.navigation

import com.intellij.codeInsight.navigation.actions.GotoDeclarationHandler
import com.intellij.openapi.editor.Editor
import com.intellij.psi.PsiElement
import com.intellij.openapi.actionSystem.DataContext
import dejavu.intellij.language.psi.fragments.BlockFragmentNode
import dejavu.intellij.language.psi.fragments.ExtendsFragmentNode
import dejavu.intellij.language.psi.DejavuTypes
import dejavu.intellij.language.workspace.BlockResolver
import dejavu.intellij.language.workspace.PathResolver
import dejavu.intellij.language.workspace.ExtendsResolver
import com.intellij.openapi.project.Project
import com.intellij.psi.PsiManager
import com.intellij.openapi.vfs.VirtualFile

/**
 * Dejavu 导航功能处理器
 * 支持跳转到子类 extends 和父类 block
 */
class DejavuGotoDeclarationHandler : GotoDeclarationHandler {
    override fun getGotoDeclarationTargets(
        sourceElement: PsiElement?,
        offset: Int,
        editor: Editor
    ): Array<PsiElement>? {
        if (sourceElement == null) return null

        // 处理 block 名称导航：跳转到父类的同名 block
        if (sourceElement.node.elementType == DejavuTypes.IDENTIFIER) {
            val blockFragment = sourceElement.parent as? BlockFragmentNode
            if (blockFragment != null) {
                return findParentBlock(blockFragment)
            }
        }

        // 处理 extends 导航：跳转到父模板文件
        val extendsFragment = sourceElement.parent as? ExtendsFragmentNode
        if (extendsFragment != null) {
            return findExtendsTarget(extendsFragment)
        }

        return null
    }

    /**
     * 查找父类中的同名 block
     */
    private fun findParentBlock(blockFragment: BlockFragmentNode): Array<PsiElement>? {
        val project = blockFragment.project
        val currentFile = blockFragment.containingFile.virtualFile ?: return null
        val blockName = blockFragment.getBlockName() ?: return null

        // 解析当前文件的 extends 关系
        val extendsResolver = ExtendsResolver.getInstance(project)
        val parentFile = extendsResolver.resolveExtends(currentFile)

        if (parentFile != null) {
            val blockResolver = BlockResolver.getInstance(project)
            val parentBlocks = blockResolver.resolveBlocks(parentFile)
            val parentBlock = parentBlocks[blockName]
            if (parentBlock != null) {
                return arrayOf(parentBlock)
            }
        }

        return null
    }

    /**
     * 查找 extends 指向的父模板文件
     */
    private fun findExtendsTarget(extendsFragment: ExtendsFragmentNode): Array<PsiElement>? {
        val project = extendsFragment.project
        val currentFile = extendsFragment.containingFile.virtualFile ?: return null
        val extendsPath = extendsFragment.getExtendsPath() ?: return null

        val pathResolver = PathResolver.getInstance(project)
        val parentFile = pathResolver.resolveTemplatePath(extendsPath, currentFile)

        if (parentFile != null) {
            val psiFile = PsiManager.getInstance(project).findFile(parentFile)
            if (psiFile != null) {
                return arrayOf(psiFile)
            }
        }

        return null
    }

    override fun getActionText(context: DataContext): String? {
        return null
    }
}
