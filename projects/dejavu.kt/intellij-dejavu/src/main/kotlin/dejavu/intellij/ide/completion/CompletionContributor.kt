package dejavu.intellij.ide.completion

import com.intellij.codeInsight.completion.CompletionContributor
import com.intellij.codeInsight.completion.CompletionType
import com.intellij.patterns.PlatformPatterns
import dejavu.intellij.language.psi.DejavuTypes

class CompletionContributor : CompletionContributor() {
    init {
        extend(
            CompletionType.BASIC,
            PlatformPatterns.psiElement(DejavuTypes.TEXT),
            TextCompletionProvider()
        )
    }

}
