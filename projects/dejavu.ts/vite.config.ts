import { defineConfig } from "vitest/config";

export default defineConfig({
    test: {
        globals: true,
        environment: "node",
        include: [
            "packages/**/__tests__/**/*.{test,spec}.{ts,tsx}",
            "packages/**/tests/**/*.{test,spec}.{ts,tsx}",
            "../dejavu.rs/dejavu-engine/tests/**/*.{test,spec}.{ts,tsx}",
        ],
    },
});
