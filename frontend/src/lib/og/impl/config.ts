import { env } from "#/env";

export const OG_CONFIG = {
    width: 1200,
    height: 630,

    designVersion: 3,

    siteName: "Myrtle",
    siteURL: env.VITE_SITE_URL ?? "http://localhost:3000",

    defaultImage: "/api/og/default",

    cacheDir: ".og-cache",
} as const;
