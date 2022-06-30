package dejavu.intellij.language.psi.templates

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * Macro template 节点
 * 表示 macro 定义
 */
class MacroTemplateNode(node: ASTNode) : DejavuTemplateNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitMacroTemplateNode(this)
    }
}
