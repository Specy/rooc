import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

const nodeBindings = fileURLToPath(
    new URL("./.test-pkg/rooc.js", import.meta.url),
);

export default defineConfig({
    resolve: {
        alias: [
            {
                find: /^(?:\.\.?\/)*(?:src\/)?pkg\/rooc(?:\.js)?$/,
                replacement: nodeBindings,
            },
        ],
    },
    test: {
        environment: "node",
        include: ["tests/**/*.test.ts"],
    },
});
