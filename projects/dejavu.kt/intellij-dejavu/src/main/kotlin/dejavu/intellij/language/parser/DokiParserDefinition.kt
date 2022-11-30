package dejavu.intellij.language.parser

import com.intellij.lang.ASTNode
import com.intellij.lang.ParserDefinition
import com.intellij.lang.ParserDefinition.SpaceRequirements
import com.intellij.lang.PsiParser
import com.intellij.lexer.Lexer
import com.intellij.openapi.project.Project
import com.intellij.psi.FileViewProvider
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.tree.IFileElementType
import com.intellij.psi.tree.TokenSet
import dejavu.intellij.language.DokiLanguage
import dejavu.intellij.language.file.DokiFileNode
import dejavu.intellij.language.psi.DejavuFactory
import dejavu.intellij.language.psi.DejavuTypes

class DokiParserDefinition : ParserDefinition {
    override fun createLexer(project: Project): Lexer = DejavuLexer(DokiLanguage.LanguageConfig)

    override fun createParser(project: Project): PsiParser = DejavuParser()

    override fun getFileNodeType(): IFileElementType = IFileElementType(DokiLanguage)

    override fun getCommentTokens(): TokenSet =
        TokenSet.create(DejavuTypes.COMMENT_L, DejavuTypes.COMMENT_CONTENT, DejavuTypes.COMMENT_R)

    override fun getStringLiteralElements(): TokenSet = TokenSet.create()

    override fun createElement(node: ASTNode): PsiElement {
        return DejavuFactory.createElement(node)
    }

    override fun createFile(viewProvider: FileViewProvider): PsiFile = DokiFileNode(viewProvider)

    override fun spaceExistenceTypeBetweenTokens(left: ASTNode, right: ASTNode): SpaceRequirements {
        return SpaceRequirements.MAY
    }
}