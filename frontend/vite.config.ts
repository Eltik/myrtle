import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const config = defineConfig({
    resolve: {
        tsconfigPaths: true,
        alias: [
            {
                find: /^@base-ui\/utils\/store$/,
                replacement: path.resolve(__dirname, "node_modules/@base-ui/utils/esm/store/index.js"),
            },
            {
                find: /^@base-ui\/utils\/(.+)$/,
                replacement: path.resolve(__dirname, "node_modules/@base-ui/utils") + "/$1.js",
            },
        ],
    },
    plugins: [devtools(), tailwindcss(), tanstackStart(), viteReact()],
});

export default config;
