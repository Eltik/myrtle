import fs from "node:fs";
import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { nitro } from "nitro/vite";
import { defineConfig, type Plugin } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

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

/** Dev only: serve an alternate export root at `/altassets/` so an exporter change can be
 *  scored without installing over the tree the backend serves. Same path shape as the
 *  backend (`/spine/DynIllust/...`).
 *
 *  Set `DYNCHAR_ALT_ROOT` to the directory containing `spine/` and pass
 *  `?assetroot=/altassets` to the renderer; without either, nothing changes. Read-only. */
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
    server: {
        // Polling is for the Docker dev container (Dockerfile.dev sets VITE_USE_POLLING=1), where a
        // bind mount from the host does not reliably deliver file events. A native run gets FSEvents
        // or inotify instead, while polling stats every watched file each 100 ms and burns most of a core at idle.
        watch: process.env.VITE_USE_POLLING === "1" ? { usePolling: true, interval: 100 } : {},
    },
    plugins: [externalizeResvg(), altAssetRoot(), devtools(), tailwindcss(), tanstackStart(), nitro(), viteReact(), tsconfigPaths()],
    optimizeDeps: { exclude: ["@resvg/resvg-js"] },
});

export default config;
