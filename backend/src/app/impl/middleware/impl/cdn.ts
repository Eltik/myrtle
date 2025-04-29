import fs from "fs";
import path from "path";
import mime from "mime-types";
import { env } from "../../../../env";
import createResponse from "./response";
import type { CdnOptions } from "../../../../types/impl/app";

// Default allowed file extensions for security
const DEFAULT_ALLOWED_EXTENSIONS = [
    // Images
    ".jpg",
    ".jpeg",
    ".png",
    ".gif",
    ".webp",
    ".svg",
    // Audio
    ".mp3",
    ".ogg",
    ".wav",
    // Video
    ".mp4",
    ".webm",
    // Documents
    ".pdf",
    ".json",
    // Other
    ".txt",
    ".csv",
    ".xml",
    ".skel",
    ".atlas",
];

/**
 * Validates and normalizes the asset path to prevent path traversal attacks
 */
function validateAndNormalizePath(assetPath: string): string | null {
    try {
        // Remove any null bytes
        const sanitizedPath = assetPath.replace(/\0/g, "");

        // Decode the path once to handle URL encoding
        const decodedPath = decodeURIComponent(sanitizedPath);

        // Normalize the path to resolve any '..' or '.' segments
        const normalizedPath = path.normalize(decodedPath);

        // Check if the normalized path contains any attempts to navigate outside
        if (normalizedPath.includes("..")) {
            return null;
        }

        // Strip any leading slashes to prevent absolute path access
        return normalizedPath.replace(/^\/+/, "");
    } catch (error) {
        console.error("Error in validateAndNormalizePath:", error);
        return null;
    }
}

/**
 * Serves static assets from the unpacked directory with security measures
 */
const serveAsset = async (req: Request, assetPath: string, options: CdnOptions = {}): Promise<Response> => {
    try {
        // Validate and normalize the path
        const normalizedPath = validateAndNormalizePath(assetPath);

        if (!normalizedPath) {
            return createResponse(JSON.stringify({ error: "Invalid path" }), 400);
        }

        // Get the full path to the asset
        const fullPath = path.join(env.UNPACKED_DIR, normalizedPath);

        // Check if the file exists
        if (!fs.existsSync(fullPath)) {
            return createResponse(JSON.stringify({ error: "Asset not found" }), 404);
        }

        // Get the file stats
        const stats = fs.statSync(fullPath);

        // If it's a directory, reject
        if (stats.isDirectory()) {
            return createResponse(JSON.stringify({ error: "Cannot serve directory" }), 400);
        }

        // Verify file extension is allowed
        const fileExtension = path.extname(fullPath).toLowerCase();
        const allowedExtensions = options.allowedExtensions || DEFAULT_ALLOWED_EXTENSIONS;

        if (!allowedExtensions.includes(fileExtension)) {
            return createResponse(JSON.stringify({ error: "File type not allowed" }), 403);
        }

        // Get file contents
        const fileContents = Bun.file(fullPath);

        // Determine content type based on file extension
        const contentType = mime.lookup(fullPath) || "application/octet-stream";

        // Check for conditional requests
        const ifModifiedSince = req.headers.get("if-modified-since");
        if (ifModifiedSince) {
            const ifModifiedSinceDate = new Date(ifModifiedSince);
            const modifiedDate = new Date(stats.mtime);

            // If the file hasn't been modified since the client's last request
            if (ifModifiedSinceDate >= modifiedDate) {
                return new Response(null, {
                    status: 304, // Not Modified
                    headers: {
                        "Cache-Control": options.cacheControl || "public, max-age=86400",
                    },
                });
            }
        }

        // Set up headers
        const headers = {
            "Content-Type": contentType,
            "Cache-Control": options.cacheControl || "public, max-age=86400",
            "Content-Length": String(stats.size),
            "Last-Modified": stats.mtime.toUTCString(),
            ETag: `"${stats.size}-${stats.mtime.getTime()}"`,
            ...options.headers,
        };

        return new Response(fileContents, {
            status: 200,
            headers,
        });
    } catch (err) {
        console.error("Error serving asset:", err);
        return createResponse(JSON.stringify({ error: "Internal server error" }), 500);
    }
};

export default serveAsset;
