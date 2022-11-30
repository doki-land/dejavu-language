import {exec} from "child_process";
import fs from "fs";
import path from "path";
import {fileURLToPath} from "url";
import {promisify} from "util";

const execAsync = promisify(exec);
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.join(__dirname, "..");

// 1. 删除 txt 文件
console.log("=== 步骤 1: 删除 txt 文件 ===");

const directories = ["src/test/testData/parser"];

directories.forEach((dir) => {
    const fullPath = path.join(rootDir, dir);
    if (fs.existsSync(fullPath)) {
        const files = fs.readdirSync(fullPath);
        const txtFiles = files.filter((file) => path.extname(file) === ".txt");
        txtFiles.forEach((file) => {
            const filePath = path.join(fullPath, file);
            fs.unlinkSync(filePath);
            console.log(`删除: ${path.join(dir, file)}`);
        });
    }
});

console.log("txt 文件已删除\n");

// 2. 运行 lexer 和 parser 测试，限时 60s
console.log("=== 步骤 2: 运行测试 (限时 60s) ===");

function runTest(testClass, timeoutMs) {
    return new Promise(async (resolve, reject) => {
        console.log(`\n开始运行: ${testClass}`);
        const startTime = Date.now();

        const isWindows = process.platform === "win32";
        const gradleCmd = isWindows ? "gradlew.bat" : "./gradlew";
        const command = isWindows
            ? `$env:JAVA_HOME='C:\\Users\\28059\\.jdks\\graalvm-jdk-21.0.7'; ./gradlew test --tests "${testClass}" -x prepareTestSandbox`
            : `${gradleCmd} test --tests "${testClass}" -x prepareTestSandbox`;

        try {
            const {stdout, stderr} = await execAsync(command, {
                cwd: rootDir,
                timeout: timeoutMs,
                killSignal: "SIGTERM",
            });

            const elapsed = Date.now() - startTime;
            console.log(stdout);
            if (stderr) console.error(stderr);

            if (stdout.includes("BUILD SUCCESSFUL")) {
                console.log(`\n✅ ${testClass} 完成 (${elapsed}ms)`);
                resolve({success: true, elapsed});
            } else {
                console.log(`\n❌ ${testClass} 失败 (${elapsed}ms)`);
                resolve({success: false, elapsed, stdout, stderr});
            }
        } catch (error) {
            const elapsed = Date.now() - startTime;

            if (error.killed || error.signal === "SIGTERM") {
                console.error(`\n❌ ${testClass} 超时！`);
                reject(new Error(`Timeout after ${timeoutMs}ms`));
            } else {
                console.log(`\n❌ ${testClass} 失败 (${elapsed}ms)`);
                console.log(error.stdout || "");
                console.error(error.stderr || "");
                resolve({success: false, elapsed, error});
            }
        }
    });
}

const TIMEOUT = 60000; // 60秒

(async () => {
    try {
        // 运行 parser 测试
        const parserResult = await runTest("dejavu.intellij.language.DejavuParserTest", TIMEOUT);

        if (!parserResult.success) {
            console.log("\n=== Parser 测试失败 ===");
        }

        console.log("\n=== 测试完成 ===");
    } catch (error) {
        console.error("\n=== 测试执行出错 ===");
        console.error(error.message);

        // 3. 如果超时，分析死循环位置
        if (error.message.includes("Timeout")) {
            console.log("\n=== 解析过程发生死循环，需分析死循环位置 ===");
            console.log("提示: 未生成 txt 的文件即为发生死循环的文件");

            // 检查哪些 dejavu 文件没有生成 txt 文件
            console.log("\n=== 分析未生成 txt 的文件 ===");
            directories.forEach((dir) => {
                const fullPath = path.join(rootDir, dir);
                if (fs.existsSync(fullPath)) {
                    const files = fs.readdirSync(fullPath);
                    const dejavuFiles = files.filter((file) => path.extname(file) === ".dejavu");
                    const txtFiles = files.filter((file) => path.extname(file) === ".txt");

                    // 获取所有 txt 文件的基础名称（转换为小写）
                    const txtFileNames = new Set(
                        txtFiles.map((file) => path.basename(file, ".txt").toLowerCase()),
                    );

                    // 检查哪些 dejavu 文件没有对应的 txt 文件（转换为小写比较）
                    const missingTxtFiles = dejavuFiles.filter((file) => {
                        const baseName = path.basename(file, ".dejavu").toLowerCase();
                        return !txtFileNames.has(baseName);
                    });

                    if (missingTxtFiles.length > 0) {
                        console.log(`\n目录 ${dir} 中未生成 txt 的文件:`);
                        missingTxtFiles.forEach((file) => {
                            console.log(`  - ${file}`);
                        });
                    } else {
                        console.log(`\n目录 ${dir} 中所有 dejavu 文件都生成了 txt 文件`);
                    }
                }
            });
        }

        process.exit(1);
    }
})();
