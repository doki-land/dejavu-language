package dejavu.intellij.language.workspace

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.openapi.vfs.VirtualFileManager
import com.intellij.openapi.vfs.newvfs.BulkFileListener
import com.intellij.openapi.vfs.newvfs.events.VFileEvent
import com.intellij.util.concurrency.AppExecutorUtil
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.locks.ReentrantReadWriteLock
import kotlin.concurrent.read
import kotlin.concurrent.write

/**
 * 工作区文件索引服务
 * 用于扫描工作区内所有 .dejavu 和 .doki 文件并建立索引
 */
class WorkspaceFileIndexer(private val project: Project) {
    private val detector = FrameworkConfigDetector.getInstance(project)
    private val index: MutableMap<VirtualFile, Map<String, VirtualFile>> = ConcurrentHashMap()
    private val lock = ReentrantReadWriteLock()
    private val templateExtensions = setOf("dejavu", "doki")

    init {
        // 注册文件系统变化监听器
        project.messageBus.connect().subscribe(VirtualFileManager.VFS_CHANGES, object : BulkFileListener {
            override fun after(events: MutableList<out VFileEvent>) {
                handleVfsEvents(events)
            }
        })
    }

    /**
     * 获取指定工作区的所有模板文件
     * @param workspaceRoot 工作区根目录
     * @return 工作区内的模板文件映射，键为相对路径，值为文件对象
     */
    fun getTemplateFiles(workspaceRoot: VirtualFile): Map<String, VirtualFile> {
        lock.read {
            index[workspaceRoot]?.let { return it }
        }

        // 索引不存在，构建索引
        val files = buildIndex(workspaceRoot)
        lock.write {
            index[workspaceRoot] = files
        }
        return files
    }

    /**
     * 构建工作区索引
     * @param workspaceRoot 工作区根目录
     * @return 工作区内的模板文件映射
     */
    private fun buildIndex(workspaceRoot: VirtualFile): Map<String, VirtualFile> {
        val files = mutableMapOf<String, VirtualFile>()
        scanDirectory(workspaceRoot, workspaceRoot, files)
        return files
    }

    /**
     * 扫描目录及其子目录
     * @param current 当前目录
     * @param root 工作区根目录
     * @param files 文件映射
     */
    private fun scanDirectory(current: VirtualFile, root: VirtualFile, files: MutableMap<String, VirtualFile>) {
        if (!current.isDirectory) return

        current.children.forEach { file ->
            if (file.isDirectory) {
                scanDirectory(file, root, files)
            } else if (isTemplateFile(file)) {
                val relativePath = root.path.let {
                    file.path.substring(it.length).removePrefix("/")
                }
                files[relativePath] = file
            }
        }
    }

    /**
     * 检查文件是否为模板文件
     * @param file 要检查的文件
     * @return 如果是模板文件则返回 true
     */
    private fun isTemplateFile(file: VirtualFile): Boolean {
        return templateExtensions.contains(file.extension?.lowercase())
    }

    /**
     * 处理文件系统事件
     * @param events 文件系统事件列表
     */
    private fun handleVfsEvents(events: MutableList<out VFileEvent>) {
        // 在后台线程中处理事件
        AppExecutorUtil.getAppExecutorService().execute {
            val affectedWorkspaces = mutableSetOf<VirtualFile>()

            events.forEach { event ->
                event.file?.let { file ->
                    detector.detectWorkspaceRoot(file)?.let { workspace ->
                        affectedWorkspaces.add(workspace)
                    }
                }
            }

            // 重新索引受影响的工作区
            affectedWorkspaces.forEach {
                lock.write {
                    index.remove(it)
                }
            }
        }
    }

    /**
     * 清除指定工作区的索引
     * @param workspaceRoot 工作区根目录
     */
    fun clearIndex(workspaceRoot: VirtualFile) {
        lock.write {
            index.remove(workspaceRoot)
        }
    }

    /**
     * 清除所有索引
     */
    fun clearAllIndexes() {
        lock.write {
            index.clear()
        }
    }

    companion object {
        /**
         * 获取工作区文件索引服务的实例
         * @param project 项目
         * @return 工作区文件索引服务实例
         */
        fun getInstance(project: Project): WorkspaceFileIndexer {
            return project.getService(WorkspaceFileIndexer::class.java)
        }
    }
}
