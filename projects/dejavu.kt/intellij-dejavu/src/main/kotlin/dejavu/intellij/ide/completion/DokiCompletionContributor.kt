package dejavu.intellij.ide.completion

import com.intellij.codeInsight.completion.CompletionContributor
import com.intellij.codeInsight.completion.CompletionParameters
import com.intellij.codeInsight.completion.CompletionProvider
import com.intellij.codeInsight.completion.CompletionResultSet
import com.intellij.codeInsight.lookup.LookupElementBuilder
import com.intellij.psi.PsiElement
import com.intellij.util.ProcessingContext
import dejavu.intellij.language.DokiLanguage
import dejavu.intellij.language.workspace.DokiContextManager

/**
 * Doki代码补全贡献者
 * 提供Doki框架上下文关键词的自动完成功能
 */
class DokiCompletionContributor : CompletionContributor() {
    init {
        // 注册Doki上下文关键词补全
        extend(
            com.intellij.codeInsight.completion.CompletionType.BASIC,
            com.intellij.patterns.PlatformPatterns.psiElement()
                .withLanguage(DokiLanguage),
            object : CompletionProvider<CompletionParameters>() {
                override fun addCompletions(
                    parameters: CompletionParameters,
                    context: ProcessingContext,
                    result: CompletionResultSet
                ) {
                    val project = parameters.originalFile.project
                    val contextManager = DokiContextManager.getInstance(project)

                    // 补全上下文关键词
                    val contextKeywords = contextManager.getAllContextKeywords()
                    contextKeywords.forEach {
                        result.addElement(
                            LookupElementBuilder.create(it)
                                .withTailText(" (Doki context")
                                .withTypeText("Context")
                        )
                    }
                }
            }
        )
    }
}
