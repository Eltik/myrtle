import { createEnv } from "@t3-oss/env-core";
import { z } from "zod";

export const env = createEnv({
    server: {
        NODE_ENV: z.enum(["development", "test", "production"]).default("development"),
        BACKEND_URL: z.url().optional(),
        GITHUB_REPO: z.string().default("Eltik/myrtle"),
        GITHUB_TOKEN: z.string().optional(),
        GITHUB_BRANCH: z.string().optional(),
    },

    clientPrefix: "VITE_",

    client: {
        VITE_APP_TITLE: z.string().min(1).optional(),
        VITE_BACKEND_URL: z.url().optional(),
        VITE_SITE_URL: z.url().optional(),
        // Base of the object store's public asset tree (`S3_PUBLIC_URL` joined with
        // `S3_PREFIX`). Unset: raw-tree assets come from the backend's `/api/assets`.
        VITE_ASSET_BASE_URL: z.url().optional(),
    },

    runtimeEnv: {
        ...import.meta.env,
        BACKEND_URL: process.env.BACKEND_URL,
        VITE_BACKEND_URL: process.env.VITE_BACKEND_URL ?? import.meta.env.VITE_BACKEND_URL,
        VITE_SITE_URL: process.env.VITE_SITE_URL ?? import.meta.env.VITE_SITE_URL,
        VITE_APP_TITLE: process.env.VITE_APP_TITLE ?? import.meta.env.VITE_APP_TITLE,
        VITE_ASSET_BASE_URL: process.env.VITE_ASSET_BASE_URL ?? import.meta.env.VITE_ASSET_BASE_URL,
        NODE_ENV: process.env.NODE_ENV,
        GITHUB_REPO: process.env.GITHUB_REPO,
        GITHUB_TOKEN: process.env.GITHUB_TOKEN,
        GITHUB_BRANCH: process.env.GITHUB_BRANCH,
    },

    // `GITHUB_REPO=` in .env reads as unset, so the default applies instead of "".
    emptyStringAsUndefined: true,
});
