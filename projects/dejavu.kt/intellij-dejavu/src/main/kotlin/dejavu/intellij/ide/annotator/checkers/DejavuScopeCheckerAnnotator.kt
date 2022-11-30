package dejavu.intellij.ide.annotator.checkers

import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.psi.PsiElement


class DejavuScopeCheckerAnnotator : CheckerAnnotator() {
    override fun check(element: PsiElement, holder: AnnotationHolder): CheckerAnnotatorResult =
        if (holder.isBatchMode) {
            CheckerAnnotatorResult.Ok
        } else {
            when (element) {
//                is YggRegex -> checkScope(element)
                else -> CheckerAnnotatorResult.Ok
            }
        }
}

