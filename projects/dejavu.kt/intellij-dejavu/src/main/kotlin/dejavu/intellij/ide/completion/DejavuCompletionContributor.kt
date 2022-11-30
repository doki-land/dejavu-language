package dejavu.intellij.ide.completion

import com.intellij.codeInsight.completion.CompletionContributor
import com.intellij.codeInsight.completion.CompletionParameters
import com.intellij.codeInsight.completion.CompletionProvider
import com.intellij.codeInsight.completion.CompletionResultSet
import com.intellij.codeInsight.lookup.LookupElementBuilder
import com.intellij.psi.PsiElement
import com.intellij.util.ProcessingContext
import dejavu.intellij.language.psi.DejavuTypes

/**
 * Dejavu 代码补全贡献者
 * 提供模板指令和变量的补全功能
 */
class DejavuCompletionContributor : CompletionContributor() {
    init {
        // 注册模板指令补全
        extend(
            com.intellij.codeInsight.completion.CompletionType.BASIC,
            com.intellij.patterns.PlatformPatterns.psiElement(),
            object : CompletionProvider<CompletionParameters>() {
                override fun addCompletions(
                    parameters: CompletionParameters,
                    context: ProcessingContext,
                    result: CompletionResultSet
                ) {
                    // 模板指令补全
                    val directives = listOf(
                        "if", "loop", "match", "while", "extends", "block", "include", "raw"
                    )
                    directives.forEach {
                        result.addElement(
                            LookupElementBuilder.create(it)
                                .withTailText(" (Dejavu directive")
                                .withTypeText("Directive")
                        )
                    }
                }
            }
        )

        // 注册变量补全（mock 数据）
        extend(
            com.intellij.codeInsight.completion.CompletionType.BASIC,
            com.intellij.patterns.PlatformPatterns.psiElement(),
            object : CompletionProvider<CompletionParameters>() {
                override fun addCompletions(
                    parameters: CompletionParameters,
                    context: ProcessingContext,
                    result: CompletionResultSet
                ) {
                    // 模拟变量数据
                    val mockVariables = listOf(
                        "user", "items", "index", "name", "title", "content", "data"
                    )
                    mockVariables.forEach {
                        result.addElement(
                            LookupElementBuilder.create(it)
                                .withTailText(" (Dejavu variable")
                                .withTypeText("Variable")
                        )
                    }
                }
            }
        )
    }
}
