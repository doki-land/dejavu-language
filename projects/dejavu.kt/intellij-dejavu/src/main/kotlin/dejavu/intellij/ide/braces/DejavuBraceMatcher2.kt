package dejavu.intellij.ide.braces

import com.intellij.lang.BracePair
import com.intellij.lang.PairedBraceMatcher
import com.intellij.psi.PsiFile
import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import com.intellij.psi.tree.TokenSet

class DejavuBraceMatcher2 : PairedBraceMatcher {
    override fun getPairs(): Array<BracePair> = PAIRS

    override fun isPairedBracesAllowedBeforeType(lbraceType: IElementType, next: IElementType?): Boolean =
        next != null && InsertPairBraceBefore.contains(next)

    override fun getCodeConstructStart(file: PsiFile?, openingBraceOffset: Int): Int = openingBraceOffset

    companion object {
        private val PAIRS = arrayOf<BracePair>(
            // 暂时为空，后续可以根据需要添加括号对
        )

        private val InsertPairBraceBefore = TokenSet.create(
            TokenType.WHITE_SPACE
        )
    }
}
