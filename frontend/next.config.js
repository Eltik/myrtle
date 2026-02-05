/**
 * Run `build` or `dev` with `SKIP_ENV_VALIDATION` to skip env validation. This is especially useful
 * for Docker builds.
 */
import "./src/env.js";

/** @type {import("next").NextConfig} */
const config = {
    reactStrictMode: true,

    /**
     * If you are using `appDir` then you must comment the below `i18n` config out.
     *
     * @see https://github.com/vercel/next.js/issues/41980
     */
    i18n: {
        locales: ["en"],
        defaultLocale: "en",
    },

    // Optimize package imports to reduce bundle size
    experimental: {
        optimizePackageImports: ["lucide-react", "@radix-ui/react-icons", "motion/react", "recharts"],
    },

    images: {
        remotePatterns: [
            {
                protocol: "https",
                hostname: "**.*.*",
            },
            {
                protocol: "https",
                hostname: "**.**.*.*",
            },
            {
                protocol: "https",
                hostname: "**.*.*.*",
            },
        ],
        minimumCacheTTL: 86400, // 1 day minimum cache for optimized images
    },
    async headers() {
        return [
            {
                source: "/api/cdn/:path*",
                headers: [
                    {
                        key: "Cache-Control",
                        value: "public, max-age=86400, stale-while-revalidate=604800",
                    },
                ],
            },
        ];
    },
    async redirects() {
        return [
            {
                source: "/discord",
                destination: "https://discord.gg/j2CjDvVztA",
                permanent: false,
            },
            {
                source: "/github",
                destination: "https://github.com/Eltik/myrtle",
                permanent: false,
            },
            // Route migration redirects
            {
                source: "/operators/list",
                destination: "/collection/operators",
                permanent: true,
            },
            {
                source: "/operators",
                has: [{ type: "query", key: "id" }],
                destination: "/collection/operators",
                permanent: true,
            },
            {
                source: "/operators",
                destination: "/collection/operators",
                permanent: true,
            },
            {
                source: "/operators/tier-list",
                destination: "/tier-list",
                permanent: true,
            },
            {
                source: "/tools/tier-list",
                destination: "/tier-list",
                permanent: true,
            },
            {
                source: "/tools/enemies",
                destination: "/collection/enemies",
                permanent: true,
            },
            {
                source: "/my/gacha",
                destination: "/gacha/history",
                permanent: true,
            },
            {
                source: "/stats/gacha",
                destination: "/gacha/community",
                permanent: true,
            },
        ];
    },
};

export default config;
