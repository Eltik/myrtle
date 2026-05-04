import { env } from "#/env";

export const OG_CONFIG = {
    width: 1200,
    height: 630,

    // Bump this when the visual design changes — invalidates every cached image.
    designVersion: 1,

    siteName: "Myrtle",
    // TODO: replace with a real env var (e.g. VITE_SITE_URL) once you deploy.
    siteUrl: env.VITE_SITE_URL ?? "http://localhost:3000",

    // Static fallback shipped from /public — used when no per-resource image exists.
    defaultImage: "/og/default.png",

    // On-disk cache for rendered PNGs. Lives outside /public; served through the API route.
    cacheDir: ".og-cache",
} as const;
