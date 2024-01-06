import { z } from "zod";
import { createEnv } from "@t3-oss/env-nextjs";

export const env = createEnv({
    server: {
        BACKEND_URL: z.string().url(),
        NODE_ENV: z.enum(["development", "test", "production"]),
    },
    runtimeEnv: {
        BACKEND_URL: process.env.BACKEND_URL ?? "http://localhost:3060",
        NODE_ENV: process.env.NODE_ENV,
    },
    skipValidation: !!process.env.SKIP_ENV_VALIDATION,
    emptyStringAsUndefined: true,
});
