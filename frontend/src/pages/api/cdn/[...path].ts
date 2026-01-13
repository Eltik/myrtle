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

    // Log the raw incoming URL first thing
    // console.log("RAW CDN request URL:", req.url);

    // Determine if we're in development mode
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
        // Handle both encoded and unencoded special characters
        const encodedPath = pathPart
            ? pathPart
                  .split("/")
                  .map((segment) => {
                      // If the segment already contains encoded characters, don't encode it again
                      if (segment.includes("%")) {
                          return segment;
                      }
                      // Handle special characters like # that might be unencoded
                      // First replace # with %23, then encode any other special characters
                      const withHashEncoded = segment.replace(/#/g, "%23");
                      return encodeURIComponent(withHashEncoded);
                  })
                  .join("/")
            : "";
        const backendURL = `${env.BACKEND_URL ?? ""}/cdn/${encodedPath}`;
        const fullURL = queryPart ? `${backendURL}?${queryPart}` : backendURL;

        // console.log("CDN proxy request:", {
        //     originalPath: pathPart,
        //     encodedPath,
        //     backendURL,
        //     fullURL
        // });

        // Make request to backend
        const response = await fetch(fullURL, {
            headers: {
                // Forward specific headers from client request if needed
                Accept: req.headers.accept ?? "*/*",
                "Accept-Encoding": (req.headers["accept-encoding"] as string) ?? "",
                "User-Agent": req.headers["user-agent"] ?? "",
                "X-Forwarded-For": (req.headers["x-forwarded-for"] as string) ?? req.socket.remoteAddress ?? "",
            },
        });

        // If response failed, return error
        if (!response.ok) {
            console.error("Backend request failed:", {
                status: response.status,
                url: fullURL,
                statusText: response.statusText,
            });
            return res.status(response.status).json({
                error: "Failed to fetch asset",
                status: response.status,
            });
        }

        // Get content type and other headers from backend response
        const contentType = response.headers.get("Content-Type") ?? "application/octet-stream";
        const contentLength = response.headers.get("Content-Length");
        const cacheControl = isDevelopment ? "no-store, max-age=0" : (response.headers.get("Cache-Control") ?? null);
        const etag = response.headers.get("ETag");
        const lastModified = response.headers.get("Last-Modified");

        // Set response headers
        res.setHeader("Content-Type", contentType);
        if (contentLength) res.setHeader("Content-Length", contentLength);

        // In development, always use no-cache, otherwise use the backend's cache control
        if (cacheControl) res.setHeader("Cache-Control", cacheControl);

        if (etag) res.setHeader("ETag", etag);
        if (lastModified) res.setHeader("Last-Modified", lastModified);

        // If in development mode, log that caching is disabled
        if (isDevelopment) {
            console.log("Development mode: CDN cache disabled");
        }

        // Get the response body as buffer and send it
        const buffer = await response.arrayBuffer();
        res.status(response.status).send(Buffer.from(buffer));
    } catch (error) {
        console.error("Error proxying to CDN:", error);
        res.status(500).json({ error: "Internal server error" });
    }
}
