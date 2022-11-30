package dejavu.intellij.language.file

import com.intellij.extapi.psi.PsiFileBase
import com.intellij.openapi.fileTypes.FileType
import com.intellij.psi.FileViewProvider
import dejavu.intellij.language.DejavuLanguage
import dejavu.intellij.language.file.DejavuFileType

class DejavuFileNode(viewProvider: FileViewProvider) : PsiFileBase(viewProvider, DejavuLanguage) {
    override fun getFileType(): FileType = DejavuFileType

    override fun toString(): String = "DejavuFileNode"
}