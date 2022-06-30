import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";

export default defineConfig({
    plugins: [vue()],
    resolve: {
        alias: {
            "@": resolve(__dirname, "src"),
        },
    },
    server: {
        fs: {
            // pnpm hoists monaco under the workspace root
            allow: [resolve(__dirname, "../.."), resolve(__dirname, "../../..")],
        },
    },
    build: {
        target: "esnext",
    },
});
