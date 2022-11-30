package dejavu.intellij.ide.structure

import com.intellij.ide.structureView.StructureViewBuilder
import com.intellij.ide.structureView.StructureViewModel
import com.intellij.ide.structureView.StructureViewModelBase
import com.intellij.ide.structureView.StructureViewTreeElement
import com.intellij.ide.structureView.TreeBasedStructureViewBuilder
import com.intellij.ide.structureView.impl.common.PsiTreeElementBase
import com.intellij.lang.PsiStructureViewFactory
import com.intellij.openapi.editor.Editor
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import dejavu.intellij.language.psi.DejavuTypes

/**
 * Dejavu 结构视图工厂
 */
class DejavuStructureViewFactory : PsiStructureViewFactory {
    override fun getStructureViewBuilder(psiFile: PsiFile): StructureViewBuilder {
        return object : TreeBasedStructureViewBuilder() {
            override fun createStructureViewModel(editor: Editor?): StructureViewModel {
                return DejavuStructureViewModel(psiFile)
            }
        }
    }
}

/**
 * Dejavu 结构视图模型
 */
class DejavuStructureViewModel(psiFile: PsiFile) :
    StructureViewModelBase(psiFile, DejavuStructureViewElement(psiFile)) {
    override fun getSuitableClasses(): Array<Class<*>> {
        return arrayOf(PsiFile::class.java)
    }
}

/**
 * Dejavu 结构视图元素
 */
class DejavuStructureViewElement(private val element: PsiElement) : PsiTreeElementBase<PsiElement>(element) {
    override fun getChildrenBase(): Collection<StructureViewTreeElement> {
        val children = mutableListOf<StructureViewTreeElement>()

        // 桩实现：返回空列表
        // 未来实现：
        // 1. 遍历 PSI 树
        // 2. 为每个重要元素（如 if、loop、match 块）创建结构视图元素
        // 3. 添加到 children 列表

        return children
    }

    override fun getPresentableText(): String? {
        return when (element) {
            is PsiFile -> element.name
            else -> element.toString()
        }
    }
}
