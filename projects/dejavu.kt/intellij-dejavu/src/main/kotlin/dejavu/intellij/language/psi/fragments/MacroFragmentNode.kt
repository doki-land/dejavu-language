package dejavu.intellij.language.psi.fragments

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * Macro fragment 节点
 * 表示 `<% macro <name>(<params>) %>` 语句
 */
class MacroFragmentNode(node: ASTNode) : DejavuFragmentNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitMacroFragmentNode(this)
    }
}
