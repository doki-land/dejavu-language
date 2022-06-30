package dejavu.intellij.ide.codeStyle

import com.intellij.psi.codeStyle.CodeStyleSettings
import com.intellij.psi.codeStyle.CustomCodeStyleSettings

class DejavuCodeStyleSettings(settings: CodeStyleSettings?) : CustomCodeStyleSettings(
    "DejavuCodeStyleSettings",
    settings!!
)
