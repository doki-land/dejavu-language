package dejavu.intellij.language.psi.fragments

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuTypes
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * Extends fragment 节点
 * 表示 `<% extends "path" %>` 语句
 */
class ExtendsFragmentNode(node: ASTNode) : DejavuFragmentNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitExtendsFragmentNode(this)
    }

    /**
     * 获取 extends 的路径
     * @return extends 路径
     */
    fun getExtendsPath(): String? {
        val children = children
        for (child in children) {
            if (child.node.elementType == DejavuTypes.STRING) {
                return child.text
            }
        }
        return null
    }
}
