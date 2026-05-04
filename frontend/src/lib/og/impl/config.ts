import { env } from "#/env";

export const OG_CONFIG = {
    width: 1200,
    height: 630,

    designVersion: 2,

    siteName: "Myrtle",
    siteURL: env.VITE_SITE_URL ?? "http://localhost:3000",

    defaultImage: "/og/default.png",

    cacheDir: ".og-cache",
} as const;
