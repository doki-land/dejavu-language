package dejavu.intellij.ide.folding

import com.intellij.lang.folding.FoldingBuilderEx
import com.intellij.lang.folding.FoldingDescriptor
import com.intellij.openapi.editor.Document
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement
import com.intellij.psi.util.PsiTreeUtil
import dejavu.intellij.language.psi.DejavuTypes

/**
 * Dejavu 代码折叠构建器
 * 支持折叠 if, loop, match 等模板块
 */
class DejavuFoldingBuilder : FoldingBuilderEx() {
    override fun buildFoldRegions(element: PsiElement, document: Document, quick: Boolean): Array<FoldingDescriptor> {
        val descriptors = mutableListOf<FoldingDescriptor>()

        // 查找所有模板块
        findTemplateBlocks(element, descriptors)

        return descriptors.toTypedArray()
    }

    private fun findTemplateBlocks(element: PsiElement, descriptors: MutableList<FoldingDescriptor>) {
        // 查找 if 块
        val allElements = PsiTreeUtil.findChildrenOfType(element, PsiElement::class.java)
        val ifBlocks = allElements.filter { it.node.elementType == DejavuTypes.IF_FRAGMENT }
        ifBlocks.forEach { ifFragment ->
            val endFragment = findMatchingEndFragment(ifFragment, DejavuTypes.END_FRAGMENT)
            if (endFragment != null) {
                val range = TextRange(ifFragment.textRange.startOffset, endFragment.textRange.endOffset)
                descriptors.add(FoldingDescriptor(ifFragment.node, range))
            }
        }

        // 查找 loop 块
        val loopBlocks = allElements.filter { it.node.elementType == DejavuTypes.LOOP_FRAGMENT }
        loopBlocks.forEach { loopFragment ->
            val endFragment = findMatchingEndFragment(loopFragment, DejavuTypes.END_FRAGMENT)
            if (endFragment != null) {
                val range = TextRange(loopFragment.textRange.startOffset, endFragment.textRange.endOffset)
                descriptors.add(FoldingDescriptor(loopFragment.node, range))
            }
        }

        // 查找 match 块
        val matchBlocks = allElements.filter { it.node.elementType == DejavuTypes.MATCH_FRAGMENT }
        matchBlocks.forEach { matchFragment ->
            val endFragment = findMatchingEndFragment(matchFragment, DejavuTypes.END_FRAGMENT)
            if (endFragment != null) {
                val range = TextRange(matchFragment.textRange.startOffset, endFragment.textRange.endOffset)
                descriptors.add(FoldingDescriptor(matchFragment.node, range))
            }
        }
    }

    private fun findMatchingEndFragment(
        startFragment: PsiElement,
        endElementType: com.intellij.psi.tree.IElementType
    ): PsiElement? {
        var current = startFragment.nextSibling
        var depth = 1

        while (current != null) {
            val type = current.node.elementType
            if (type == startFragment.node.elementType) {
                depth++
            } else if (type == endElementType) {
                depth--
                if (depth == 0) {
                    return current
                }
            }
            current = current.nextSibling
        }
        return null
    }

    override fun getPlaceholderText(node: com.intellij.lang.ASTNode): String? {
        return when (node.elementType) {
            DejavuTypes.IF_FRAGMENT -> "if ..."
            DejavuTypes.LOOP_FRAGMENT -> "loop ..."
            DejavuTypes.MATCH_FRAGMENT -> "match ..."
            else -> null
        }
    }

    override fun isCollapsedByDefault(node: com.intellij.lang.ASTNode): Boolean {
        return false
    }
}
