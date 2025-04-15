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
    transpilePackages: ["geist"],
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
    },
    // Add rewrites for proxying CDN requests
    async rewrites() {
        return [
            {
                source: "/assets/:path*",
                destination: `${process.env.BACKEND_URL ?? "http://localhost:3060"}/cdn/:path*`,
            },
        ];
    },
};

export default config;
