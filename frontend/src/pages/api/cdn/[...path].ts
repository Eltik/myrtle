import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "../../../env";

/**
 * API route that proxies requests to the backend CDN service
 * This allows for more control compared to rewrites, like:
 * - Adding authentication
 * - Request validation
 * - Error handling
 * - Response transformation
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    // Only allow GET requests
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    try {
        // Get path from request
        const path = req.query.path as string[];
        const pathString = path.join("/");

        // Build the backend URL
        const backendUrl = `${env.BACKEND_URL ?? "http://localhost:3060"}/cdn/${pathString}`;

        // Forward any query parameters
        const queryString = Object.entries(req.query)
            .filter(([key]) => key !== "path")
            .map(([key, value]) => `${key}=${encodeURIComponent(String(value))}`)
            .join("&");

        const fullUrl = queryString ? `${backendUrl}?${queryString}` : backendUrl;

        // Make request to backend
        const response = await fetch(fullUrl, {
            headers: {
                // Forward specific headers from client request if needed
                Accept: req.headers.accept ?? "*/*",
                "Accept-Encoding": (req.headers["accept-encoding"] as string) ?? "",
                "User-Agent": req.headers["user-agent"]! ?? "",
                "X-Forwarded-For": (req.headers["x-forwarded-for"] as string) ?? req.socket.remoteAddress! ?? "",
            },
        });

        // If response failed, return error
        if (!response.ok) {
            return res.status(response.status).json({
                error: "Failed to fetch asset",
                status: response.status,
            });
        }

        // Get content type and other headers from backend response
        const contentType = response.headers.get("Content-Type") ?? "application/octet-stream";
        const contentLength = response.headers.get("Content-Length");
        const cacheControl = response.headers.get("Cache-Control");
        const etag = response.headers.get("ETag");
        const lastModified = response.headers.get("Last-Modified");

        // Set response headers
        res.setHeader("Content-Type", contentType);
        if (contentLength) res.setHeader("Content-Length", contentLength);
        if (cacheControl) res.setHeader("Cache-Control", cacheControl);
        if (etag) res.setHeader("ETag", etag);
        if (lastModified) res.setHeader("Last-Modified", lastModified);

        // Get the response body as buffer and send it
        const buffer = await response.arrayBuffer();
        res.status(response.status).send(Buffer.from(buffer));
    } catch (error) {
        console.error("Error proxying to CDN:", error);
        res.status(500).json({ error: "Internal server error" });
    }
}
