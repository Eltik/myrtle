import tsconfigPaths from "vite-tsconfig-paths";
import { defineConfig } from "vitest/config";

/**
 * Separate from `vite.config.ts` on purpose: that config loads
 * `tanstackStart()` and `nitro()`, which build a server and have no business
 * running under a unit test. This one carries only what the tests need -
 * `#/` path resolution and a DOM for the component tests.
 */
export default defineConfig({
    plugins: [tsconfigPaths()],
    test: {
        environment: "jsdom",
        globals: false,
        setupFiles: ["./src/test-setup.ts"],
        include: ["src/**/*.test.{ts,tsx}"],
    },
});
