package dejavu.intellij.language.file

import com.intellij.extapi.psi.PsiFileBase
import com.intellij.openapi.fileTypes.FileType
import com.intellij.psi.FileViewProvider
import dejavu.intellij.language.DokiLanguage
import dejavu.intellij.language.file.DokiFileType

class DokiFileNode(viewProvider: FileViewProvider) : PsiFileBase(viewProvider, DokiLanguage) {
    override fun getFileType(): FileType = DokiFileType

    override fun toString(): String = "DokiFileNode"
}