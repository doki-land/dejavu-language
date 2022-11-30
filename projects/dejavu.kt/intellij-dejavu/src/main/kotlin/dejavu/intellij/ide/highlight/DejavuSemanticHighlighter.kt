package dejavu.intellij.ide.highlight

import com.intellij.codeInsight.daemon.impl.HighlightInfo
import com.intellij.codeInsight.daemon.impl.HighlightInfoType
import com.intellij.codeInsight.daemon.impl.HighlightVisitor
import com.intellij.codeInsight.daemon.impl.analysis.HighlightInfoHolder
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import dejavu.intellij.language.file.DejavuFileNode
import dejavu.intellij.language.file.DokiFileNode
import dejavu.intellij.language.psi.DejavuRecursiveVisitor
import dejavu.intellij.language.psi.DejavuTypes
import dejavu.intellij.language.psi.expressions.FunctionCallNode
import dejavu.intellij.language.psi.fragments.*

class DejavuSemanticHighlighter : DejavuRecursiveVisitor(), HighlightVisitor {
    private var infoHolder: HighlightInfoHolder? = null

    override fun visit(element: PsiElement) = element.accept(this)

    override fun analyze(file: PsiFile, whole: Boolean, holder: HighlightInfoHolder, action: Runnable): Boolean {
        infoHolder = holder
        action.run()
        return true
    }

    private fun highlight(element: PsiElement, color: HighlightColor) {
        val builder = HighlightInfo.newHighlightInfo(HighlightInfoType.INFORMATION)
        builder.textAttributes(color.textAttributesKey)
        builder.range(element)
        infoHolder?.add(builder.create())
    }

    override fun suitableForFile(file: PsiFile): Boolean {
        return file is DejavuFileNode || file is DokiFileNode
    }

    override fun clone(): HighlightVisitor = DejavuSemanticHighlighter()

    /**
     * 访问 BlockFragmentNode，为 block 名称添加语义高亮
     */
    override fun visitBlockFragmentNode(node: BlockFragmentNode) {
        val blockNameElement = node.node.findChildByType(DejavuTypes.IDENTIFIER)?.psi
        if (blockNameElement != null) {
            highlight(blockNameElement, HighlightColor.TYPE_HINT)
        }
        // 高亮 super 软关键词
        highlightSoftKeyword(node, "super", HighlightColor.KEYWORD)
        super.visitBlockFragmentNode(node)
    }

    /**
     * 访问 LoopFragmentNode，为 looper 软关键词添加高亮
     */
    override fun visitLoopFragmentNode(node: LoopFragmentNode) {
        highlightSoftKeyword(node, "looper", HighlightColor.KEYWORD)
        super.visitLoopFragmentNode(node)
    }

    /**
     * 访问 WhileFragmentNode，为 looper 软关键词添加高亮
     */
    override fun visitWhileFragmentNode(node: WhileFragmentNode) {
        highlightSoftKeyword(node, "looper", HighlightColor.KEYWORD)
        super.visitWhileFragmentNode(node)
    }

    /**
     * 访问 UntilFragmentNode，为 looper 软关键词添加高亮
     */
    override fun visitUntilFragmentNode(node: UntilFragmentNode) {
        highlightSoftKeyword(node, "looper", HighlightColor.KEYWORD)
        super.visitUntilFragmentNode(node)
    }

    /**
     * 访问函数调用节点，为函数名添加高亮
     */
    override fun visitFunctionCallNode(node: FunctionCallNode) {
        // 获取函数调用的第一个子元素作为函数名
        val functionNameElement = node.firstChild
        if (functionNameElement != null && functionNameElement.node.elementType == DejavuTypes.IDENTIFIER) {
            // 检查是否是 macro 定义
            if (isMacroDefinition(node)) {
                // 高亮 macro 名称
                highlight(functionNameElement, HighlightColor.SYM_MACRO)
                // 高亮 macro 参数
                highlightMacroParameters(node)
            } else {
                highlight(functionNameElement, HighlightColor.SYM_FUNCTION)
            }
        }
        super.visitFunctionCallNode(node)
    }

    /**
     * 检查是否是 macro 定义
     * macro 定义格式：macro f(arg) ... end macro
     */
    private fun isMacroDefinition(node: FunctionCallNode): Boolean {
        // 检查父节点是否是表达式模板
        val parent = node.parent
        if (parent == null) return false

        // 检查前面是否有 macro 关键词
        val prevSibling = node.prevSibling
        if (prevSibling != null && prevSibling.node.elementType == DejavuTypes.KEYWORD_MACRO) {
            return true
        }

        // 检查祖父节点中是否有 macro 关键词
        val grandParent = parent.parent
        if (grandParent != null) {
            val children = grandParent.children
            for (child in children) {
                if (child.node.elementType == DejavuTypes.KEYWORD_MACRO) {
                    return true
                }
                if (child == node) break
            }
        }

        return false
    }

    /**
     * 高亮 macro 定义中的参数
     */
    private fun highlightMacroParameters(node: FunctionCallNode) {
        // 获取 macro 定义中的所有标识符作为参数
        val macroParams = mutableSetOf<String>()

        // 遍历函数调用的子元素获取参数名
        node.children.forEach { child ->
            if (child.node.elementType == DejavuTypes.IDENTIFIER) {
                macroParams.add(child.text)
            }
        }

        // 在 macro 定义范围内高亮参数使用
        if (macroParams.isNotEmpty()) {
            highlightMacroParamUsages(node, macroParams)
        }
    }

    /**
     * 在 macro 定义范围内高亮参数使用
     */
    private fun highlightMacroParamUsages(node: FunctionCallNode, params: Set<String>) {
        // 从当前节点开始向后遍历，直到遇到 end macro
        var sibling = node.nextSibling
        while (sibling != null) {
            // 检查是否是 end macro
            if (isEndMacro(sibling)) {
                break
            }

            // 高亮参数使用
            highlightParamsInElement(sibling, params)

            sibling = sibling.nextSibling
        }
    }

    /**
     * 检查是否是 end macro
     */
    private fun isEndMacro(element: PsiElement): Boolean {
        // 检查是否是 end fragment 且包含 macro
        if (element is EndFragmentNode) {
            val text = element.text
            return text.contains("macro")
        }
        return false
    }

    /**
     * 在元素中高亮参数
     */
    private fun highlightParamsInElement(element: PsiElement, params: Set<String>) {
        if (element.node.elementType == DejavuTypes.IDENTIFIER && params.contains(element.text)) {
            highlight(element, HighlightColor.SYM_PROPERTY)
        }

        // 递归处理子元素
        element.children.forEach { child ->
            highlightParamsInElement(child, params)
        }
    }

    /**
     * 高亮软关键词
     */
    private fun highlightSoftKeyword(node: DejavuFragmentNode, keyword: String, color: HighlightColor) {
        node.children.forEach { child ->
            if (child.node.elementType == DejavuTypes.IDENTIFIER && child.text == keyword) {
                highlight(child, color)
            }
        }
    }
}
