import colors from "colors";
import path from "path";
import middleware from "../../middleware";
import { env } from "../../../../env";

// Track basic CDN metrics
const metrics = {
    totalRequests: 0,
    successfulRequests: 0,
    failedRequests: 0,
    notFoundRequests: 0,
    bytesSent: 0,
    requestsByType: {} as Record<string, number>,
    lastReset: Date.now(),
};

// Different cache configurations based on file type
const cacheConfigs: Record<string, string> = {
    // Long-lived assets (1 month)
    images: "public, max-age=2592000",
    // Medium-lived assets (1 week)
    audio: "public, max-age=604800",
    video: "public, max-age=604800",
    // Shorter-lived assets (1 day)
    data: "public, max-age=86400",
    // Default (1 day)
    default: "public, max-age=86400",
};

// File extensions grouped by type
const fileTypes: Record<string, string[]> = {
    images: [".jpg", ".jpeg", ".png", ".gif", ".webp", ".svg"],
    audio: [".mp3", ".ogg", ".wav"],
    video: [".mp4", ".webm"],
    data: [".json", ".xml", ".csv", ".txt", ".pdf"],
};

/**
 * Determines the cache configuration based on the file extension
 */
function getCacheConfig(filePath: string): string {
    const ext = path.extname(filePath).toLowerCase();

    for (const [type, extensions] of Object.entries(fileTypes)) {
        if (extensions.includes(ext)) {
            return cacheConfigs[type];
        }
    }

    return cacheConfigs.default;
}

/**
 * Update metrics for monitoring
 */
function updateMetrics(assetPath: string, success: boolean, size?: number, statusCode?: number): void {
    metrics.totalRequests++;

    if (success) {
        metrics.successfulRequests++;
        if (size) metrics.bytesSent += size;
    } else {
        metrics.failedRequests++;
        if (statusCode === 404) metrics.notFoundRequests++;
    }

    // Track by file type
    const ext = path.extname(assetPath).toLowerCase();
    metrics.requestsByType[ext] = (metrics.requestsByType[ext] || 0) + 1;
}

/**
 * CDN route handler for serving static assets from the unpacked directory
 */
const handler = async (req: Request): Promise<Response> => {
    try {
        const url = new URL(req.url);
        const paths = url.pathname.split("/");

        // Remove first two segments (/cdn/...)
        paths.shift(); // Remove empty element
        paths.shift(); // Remove 'cdn'

        // Special endpoint for CDN info and metrics
        if (paths[0] === "info") {
            return middleware.createResponse(
                JSON.stringify({
                    name: "Myrtle CDN",
                    version: "1.0.0",
                    assetsDirectory: env.UNPACKED_DIR,
                    supportedTypes: fileTypes,
                    metrics,
                }),
            );
        }

        // Join the remaining path segments to form the asset path
        const assetPath = paths.join("/");

        if (!assetPath) {
            updateMetrics("", false, undefined, 400);
            return middleware.createResponse(
                JSON.stringify({
                    error: "Invalid path",
                    message: "Please specify a path to an asset",
                }),
                400,
            );
        }

        console.log(colors.gray(`CDN Request: ${colors.yellow(assetPath)}`));

        // Determine appropriate cache configuration based on file type
        const cacheControl = getCacheConfig(assetPath);

        // Use our cdn middleware to serve the asset
        const response = await middleware.serveAsset(req, assetPath, {
            cacheControl,
        });

        // Update metrics
        const success = response.status === 200;
        const size = success ? Number(response.headers.get("Content-Length") || 0) : 0;
        updateMetrics(assetPath, success, size, response.status);

        return response;
    } catch (error) {
        console.error("CDN error:", error);
        updateMetrics("error", false, undefined, 500);
        return middleware.createResponse(JSON.stringify({ error: "Internal server error" }), 500);
    }
};

// Export route definition with path and rate limit
export default {
    path: "/cdn",
    handler,
    rateLimit: 300, // Higher rate limit for CDN assets
};
