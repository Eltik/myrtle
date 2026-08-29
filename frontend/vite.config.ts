import fs from "node:fs";
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

/** DEV ONLY: serve an ALTERNATE export root at `/altassets/`.
 *
 *  Scoring an EXPORTER change used to mean installing the new export over the tree the
 *  backend serves, which is not ours to write, so exporter gates shipped unmeasured. This
 *  serves a second, read-only export root straight off the dev server instead, under the
 *  same relative path shape the backend uses (`/spine/DynIllust/...`), so a scratch export
 *  can be scored without the deployed tree being touched at all.
 *
 *  Set `DYNCHAR_ALT_ROOT` to an export directory (the one CONTAINING `spine/`) and pass
 *  `?assetroot=/altassets` to the renderer. Absent either, nothing changes: the plugin
 *  registers no middleware and `chibiAssetURL` resolves to the backend exactly as before.
 *  `apply: "serve"` keeps it out of any production build.
 *
 *  Read-only and path-guarded: it resolves under the root and refuses anything that escapes,
 *  and it never writes. */
function altAssetRoot(): Plugin {
    return {
        name: "myrtle:alt-asset-root",
        apply: "serve",
        configureServer(server) {
            const root = process.env.DYNCHAR_ALT_ROOT;
            if (!root) return;
            const base = path.resolve(root);
            const TYPES: Record<string, string> = {
                ".json": "application/json",
                ".png": "image/png",
                ".atlas": "text/plain",
                ".skel": "application/octet-stream",
            };
            server.config.logger.info(`  myrtle:alt-asset-root  /altassets -> ${base}`);
            server.middlewares.use("/altassets", (req, res, next) => {
                let rel: string;
                try {
                    rel = decodeURIComponent((req.url ?? "").split("?")[0]);
                } catch {
                    next();
                    return;
                }
                const file = path.resolve(base, `.${rel}`);
                // Resolve-then-verify, so `..` cannot climb out of the root.
                if (file !== base && !file.startsWith(base + path.sep)) {
                    res.statusCode = 403;
                    res.end("outside alt root");
                    return;
                }
                fs.stat(file, (err, st) => {
                    if (err || !st.isFile()) {
                        next();
                        return;
                    }
                    res.setHeader("Content-Type", TYPES[path.extname(file)] ?? "application/octet-stream");
                    res.setHeader("Cache-Control", "no-store");
                    fs.createReadStream(file).pipe(res);
                });
            });
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
    plugins: [externalizeResvg(), altAssetRoot(), devtools(), tailwindcss(), tanstackStart(), nitro(), viteReact(), tsconfigPaths()],
    optimizeDeps: { exclude: ["@resvg/resvg-js"] },
});

export default config;
