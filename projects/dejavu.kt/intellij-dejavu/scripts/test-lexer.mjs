import {exec} from "child_process";
import fs from "fs";
import path from "path";
import {fileURLToPath} from "url";
import {promisify} from "util";

const execAsync = promisify(exec);
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.join(__dirname, "..");

// 1. 检查 txt 文件
console.log("=== 步骤 1: 检查 txt 文件 ===");

const directories = ["src/test/testData/lexer"];

directories.forEach((dir) => {
    const fullPath = path.join(rootDir, dir);
    if (fs.existsSync(fullPath)) {
        const files = fs.readdirSync(fullPath);
        const txtFiles = files.filter((file) => path.extname(file) === ".txt");
        if (txtFiles.length > 0) {
            console.log(`找到 ${txtFiles.length} 个 txt 文件:`);
            txtFiles.forEach((file) => {
                console.log(`- ${path.join(dir, file)}`);
            });
        } else {
            console.log(`在 ${dir} 目录中没有找到 txt 文件`);
        }
    } else {
        console.log(`目录 ${dir} 不存在`);
    }
});

console.log("\n");

// 2. 运行 lexer 测试，限时 60s
console.log("=== 步骤 2: 运行 lexer 测试 (限时 60s) ===");

function runTest(testClass, timeoutMs) {
    return new Promise(async (resolve, reject) => {
        console.log(`\n开始运行: ${testClass}`);
        const startTime = Date.now();

        const isWindows = process.platform === "win32";
        const gradleCmd = isWindows ? "gradlew.bat" : "./gradlew";
        const command = `${gradleCmd} test --tests "${testClass}"`;

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
        // 运行 lexer 测试
        const lexerResult = await runTest("dejavu.intellij.language.DejavuLexerTest", TIMEOUT);

        if (!lexerResult.success) {
            console.log("\n=== Lexer 测试失败 ===");
        }

        console.log("\n=== 测试完成 ===");
    } catch (error) {
        console.error("\n=== 测试执行出错 ===");
        console.error(error.message);

        // 3. 如果超时，分析死循环位置
        if (error.message.includes("Timeout")) {
            console.log("\n=== 分析死循环位置 ===");
            console.log("可能的死循环位置:");
            console.log(
                "1. DejavuLexer.parseText() - 当处理文本时，如果 slotStart 或 commentStart 为空字符串，会导致死循环",
            );
            console.log(
                "2. DejavuLexer.parseExprContent() - 当 slotEnd 为空字符串时，会导致死循环",
            );
            console.log(
                "3. DejavuLexer.parseCommentContent() - 当 commentEnd 为空字符串时，会导致死循环",
            );
            console.log("\n建议检查:");
            console.log(
                "- TemplateConfig 中的 slotStart, slotEnd, commentStart, commentEnd 是否为空字符串",
            );
        }

        process.exit(1);
    }
})();
