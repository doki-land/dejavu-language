package dejavu.intellij.language.file

import com.intellij.DynamicBundle
import org.jetbrains.annotations.PropertyKey
import dejavu.intellij.language.DejavuLanguage.Bundle
import java.util.function.Supplier

object DejavuBundle : DynamicBundle(Bundle) {
    fun message(@PropertyKey(resourceBundle = Bundle) key: String, vararg params: Any): String {
        return getMessage(key, *params)
    }

    fun messagePointer(@PropertyKey(resourceBundle = Bundle) key: String, vararg params: Any): Supplier<String> {
        return getLazyMessage(key, *params)
    }
}