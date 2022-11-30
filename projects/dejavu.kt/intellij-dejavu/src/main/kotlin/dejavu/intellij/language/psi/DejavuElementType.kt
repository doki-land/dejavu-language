package dejavu.intellij.language.psi

import com.intellij.psi.tree.IElementType
import dejavu.intellij.language.DejavuLanguage

class DejavuElementType(debugName: String) : IElementType(debugName, DejavuLanguage) {
    override fun toString(): String = "DejavuElement.${super.toString()}"
}


