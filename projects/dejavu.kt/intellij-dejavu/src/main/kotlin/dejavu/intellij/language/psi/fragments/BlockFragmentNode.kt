package dejavu.intellij.language.psi.fragments

import com.intellij.lang.ASTNode
import com.intellij.psi.PsiElement
import dejavu.intellij.language.psi.DejavuTypes
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * Block fragment 节点
 * 表示 `<% block <name> %>` 语句
 */
class BlockFragmentNode(node: ASTNode) : DejavuFragmentNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitBlockFragmentNode(this)
    }

    /**
     * 获取 block 的名称
     * @return block 名称
     */
    fun getBlockName(): String? {
        return findChildByType<PsiElement>(DejavuTypes.IDENTIFIER)?.text
    }
}
