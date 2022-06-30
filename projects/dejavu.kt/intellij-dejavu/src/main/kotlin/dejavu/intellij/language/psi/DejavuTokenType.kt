package dejavu.intellij.language.psi

import com.intellij.psi.tree.IElementType
import dejavu.intellij.language.DejavuLanguage

class DejavuTokenType(debugName: String) : IElementType(debugName, DejavuLanguage) {
    override fun toString(): String = "DejavuToken.${super.toString()}"
}

