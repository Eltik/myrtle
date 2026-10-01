import { env } from "#/env";

export const OG_CONFIG = {
    width: 1200,
    height: 630,

    // Bump on any change that alters how every card renders: it feeds every ogHash, so it
    // invalidates all cached images across all kinds. 4 = CJK/Cyrillic fallback faces added.
    designVersion: 4,

    siteName: "Myrtle",
    siteURL: env.VITE_SITE_URL ?? "http://localhost:3000",

    defaultImage: "/api/og/default",

    cacheDir: ".og-cache",
} as const;
