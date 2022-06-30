package dejavu.intellij.ide.assist.fixers

import com.intellij.lang.SmartEnterProcessorWithFixers
import com.intellij.openapi.editor.Editor
import com.intellij.psi.PsiElement

class CommaFixer : SmartEnterProcessorWithFixers.Fixer<SmartEnterProcessor>() {
    override fun apply(editor: Editor, processor: SmartEnterProcessor, element: PsiElement) {
        TODO("Not yet implemented")
    }
}
