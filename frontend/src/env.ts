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

    /**
     * The prefix that client-side variables must have. This is enforced both at
     * a type-level and at runtime.
     */
    clientPrefix: "VITE_",

    client: {
        VITE_APP_TITLE: z.string().min(1).optional(),
        VITE_BACKEND_URL: z.url().optional(),
        VITE_SITE_URL: z.url().optional(),
    },

    /**
     * What object holds the environment variables at runtime. This is usually
     * `process.env` or `import.meta.env`.
     */
    runtimeEnv: {
        ...import.meta.env,
        BACKEND_URL: process.env.BACKEND_URL,
        NODE_ENV: process.env.NODE_ENV,
        GITHUB_REPO: process.env.GITHUB_REPO,
        GITHUB_TOKEN: process.env.GITHUB_TOKEN,
        GITHUB_BRANCH: process.env.GITHUB_BRANCH,
    },

    /**
     * By default, this library will feed the environment variables directly to
     * the Zod validator.
     *
     * This means that if you have an empty string for a value that is supposed
     * to be a number (e.g. `PORT=` in a ".env" file), Zod will incorrectly flag
     * it as a type mismatch violation. Additionally, if you have an empty string
     * for a value that is supposed to be a string with a default value (e.g.
     * `DOMAIN=` in an ".env" file), the default value will never be applied.
     *
     * In order to solve these issues, we recommend that all new projects
     * explicitly specify this option as true.
     */
    emptyStringAsUndefined: true,
});
