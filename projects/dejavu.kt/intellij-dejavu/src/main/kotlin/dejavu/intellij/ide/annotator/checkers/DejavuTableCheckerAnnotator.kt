package dejavu.intellij.ide.annotator.checkers


import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.psi.PsiElement


class DejavuTableCheckerAnnotator : CheckerAnnotator() {
    override fun check(element: PsiElement, holder: AnnotationHolder): CheckerAnnotatorResult =
        if (holder.isBatchMode) {
            CheckerAnnotatorResult.Ok
        } else {
            when (element) {
//                is _root_ide_package_.djv.intellij.language.psi.SahaPair -> checkPair(element)
                else -> CheckerAnnotatorResult.Ok
            }
        }
}