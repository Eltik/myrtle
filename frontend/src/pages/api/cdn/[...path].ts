import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "../../../env";

/**
 * API route that proxies requests to the backend CDN service
 * Optimizations:
 * - Request coalescing (dedupe concurrent identical requests)
 * - Aggressive caching with stale-while-revalidate
 * - ETag/conditional request support for revalidation
 */

// Cached response data for request coalescing
interface CachedResponse {
    status: number;
    contentType: string;
    etag: string | null;
    lastModified: string | null;
    buffer: Buffer;
}

// In-memory request coalescing to dedupe concurrent requests
const pendingRequests = new Map<string, Promise<CachedResponse | null>>();

export const config = {
    api: {
        responseLimit: false, // Allow large files
    },
};

async function fetchAndBuffer(url: string, headers: Record<string, string>): Promise<CachedResponse | null> {
    const response = await fetch(url, { headers });

    // Handle 304 Not Modified
    if (response.status === 304) {
        return null;
    }

    const buffer = Buffer.from(await response.arrayBuffer());

    return {
        status: response.status,
        contentType: response.headers.get("Content-Type") ?? "application/octet-stream",
        etag: response.headers.get("ETag"),
        lastModified: response.headers.get("Last-Modified"),
        buffer,
    };
}

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    // Only allow GET requests
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    const isDevelopment = env.NODE_ENV === "development";

    try {
        // Get raw URL to handle special characters properly
        const rawURL = req.url ?? "";
        const urlParts = rawURL.split("/api/cdn/")[1];
        if (!urlParts) {
            return res.status(400).json({ error: "Invalid path" });
        }

        // Split at query string if it exists
        const [pathPart, queryPart] = urlParts.split("?");

        // Build the backend URL, maintaining the original encoding
        const encodedPath = pathPart
            ? pathPart
                  .split("/")
                  .map((segment) => {
                      // If the segment already contains encoded characters, don't encode it again
                      if (segment.includes("%")) {
                          return segment;
                      }
                      // Handle special characters like # that might be unencoded
                      const withHashEncoded = segment.replace(/#/g, "%23");
                      return encodeURIComponent(withHashEncoded);
                  })
                  .join("/")
            : "";
        const backendURL = `${env.BACKEND_URL ?? ""}/cdn/${encodedPath}`;
        const fullURL = queryPart ? `${backendURL}?${queryPart}` : backendURL;

        // Build request headers
        const fetchHeaders: Record<string, string> = {
            Accept: req.headers.accept ?? "*/*",
            "Accept-Encoding": (req.headers["accept-encoding"] as string) ?? "",
            "User-Agent": req.headers["user-agent"] ?? "",
            "X-Forwarded-For": (req.headers["x-forwarded-for"] as string) ?? req.socket.remoteAddress ?? "",
        };
        // Forward conditional request headers for revalidation
        if (req.headers["if-none-match"]) {
            fetchHeaders["If-None-Match"] = req.headers["if-none-match"] as string;
        }
        if (req.headers["if-modified-since"]) {
            fetchHeaders["If-Modified-Since"] = req.headers["if-modified-since"] as string;
        }

        // Request coalescing: reuse in-flight requests for same URL
        let responsePromise = pendingRequests.get(fullURL);
        if (!responsePromise) {
            responsePromise = fetchAndBuffer(fullURL, fetchHeaders);
            pendingRequests.set(fullURL, responsePromise);
            // Clean up after response completes
            responsePromise.finally(() => {
                setTimeout(() => pendingRequests.delete(fullURL), 100);
            });
        }

        const cachedResponse = await responsePromise;

        // Handle 304 Not Modified
        if (cachedResponse === null) {
            return res.status(304).end();
        }

        // If response failed, return error
        if (cachedResponse.status < 200 || cachedResponse.status >= 300) {
            console.error("Backend request failed:", {
                status: cachedResponse.status,
                url: fullURL,
            });
            return res.status(cachedResponse.status).json({
                error: "Failed to fetch asset",
                status: cachedResponse.status,
            });
        }

        const isImage = cachedResponse.contentType.startsWith("image/");

        // Set response headers
        res.setHeader("Content-Type", cachedResponse.contentType);
        res.setHeader("Content-Length", cachedResponse.buffer.length);

        // Cache headers: stale-while-revalidate for occasionally-updated assets
        if (isDevelopment) {
            res.setHeader("Cache-Control", "no-store, max-age=0");
        } else if (isImage) {
            // 1 day fresh, 7 days stale-while-revalidate
            res.setHeader("Cache-Control", "public, max-age=86400, stale-while-revalidate=604800");
        } else {
            // Non-images: shorter cache
            res.setHeader("Cache-Control", "public, max-age=3600, stale-while-revalidate=86400");
        }

        // ETag and Last-Modified for conditional requests
        if (cachedResponse.etag) res.setHeader("ETag", cachedResponse.etag);
        if (cachedResponse.lastModified) res.setHeader("Last-Modified", cachedResponse.lastModified);

        // Send the buffered response
        res.status(cachedResponse.status).send(cachedResponse.buffer);
    } catch (error) {
        console.error("Error proxying to CDN:", error);
        res.status(500).json({ error: "Internal server error" });
    }
}
