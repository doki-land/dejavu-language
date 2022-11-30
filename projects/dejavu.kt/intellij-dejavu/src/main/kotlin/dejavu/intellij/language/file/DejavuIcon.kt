package dejavu.intellij.language.file

import com.intellij.openapi.util.IconLoader
import javax.swing.Icon

object DejavuIconProvider {
    val DejavuIcon: Icon by lazy {
        IconLoader.getIcon("/icons/DejavuIcon.svg", javaClass)
    }
}

