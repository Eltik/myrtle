import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";

import { tanstackStart } from "@tanstack/react-start/plugin/vite";

import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const config = defineConfig({
    resolve: {
        tsconfigPaths: true,
        dedupe: ["@base-ui/react", "@base-ui/utils"],
    },
    optimizeDeps: {
        include: ["@base-ui/utils", "@base-ui/react"],
    },
    plugins: [devtools(), tailwindcss(), tanstackStart(), viteReact()],
});

export default config;
