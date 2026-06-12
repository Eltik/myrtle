import { createRequire } from "node:module";
import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { nitro } from "nitro/vite";
import { defineConfig, type Plugin } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

const require = createRequire(import.meta.url);
const utilsPkgJson = require.resolve("@base-ui/utils/package.json");
const utilsRoot = path.dirname(utilsPkgJson);

function externalizeResvg(): Plugin {
    return {
        name: "myrtle:externalize-resvg",
        enforce: "pre",
        resolveId(id) {
            if (/^@resvg\//.test(id)) {
                return { id, external: true };
            }
            return null;
        },
    };
}

const config = defineConfig({
    resolve: {
        alias: [
            {
                find: /^@base-ui\/utils\/store$/,
                replacement: path.join(utilsRoot, "esm/store/index.js"),
            },
            {
                find: /^@base-ui\/utils\/(.+)$/,
                replacement: path.join(utilsRoot, "$1.js"),
            },
        ],
    },
    server: {
        watch: {
            usePolling: true,
            interval: 100,
        },
    },
    plugins: [externalizeResvg(), devtools(), tailwindcss(), tanstackStart(), nitro(), viteReact(), tsconfigPaths()],
    optimizeDeps: { exclude: ["@resvg/resvg-js"] },
});

export default config;
