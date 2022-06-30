package dejavu.intellij.ide.annotator.checkers


import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.Annotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.psi.PsiElement

abstract class CheckerAnnotator : Annotator {
    protected abstract fun check(element: PsiElement, holder: AnnotationHolder): CheckerAnnotatorResult
    protected fun annotateInternal(element: PsiElement, holder: AnnotationHolder) {
        when (val result = check(element, holder)) {
            CheckerAnnotatorResult.Ok -> {}
            is CheckerAnnotatorResult.Error -> {
                val (errorText, subRange) = result
                holder
                    .newAnnotation(HighlightSeverity.ERROR, errorText)
                    .range(subRange)
                    .create()
            }
        }
    }

    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        annotateInternal(element, holder)
    }
}