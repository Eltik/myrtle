/**
 * Utilities for working with the CDN service
 */

import { env } from "~/env.js";

// Client-safe way to detect development mode
const isDevelopment = env.NEXT_PUBLIC_NODE_ENV !== "production";

/**
 * Get the CDN URL for a given asset path
 *
 * @param path - The path to the asset within the CDN
 * @param useApiRoute - Whether to use the API route (more control) or the rewrite (more efficient)
 * @param bustCache - Whether to add a cache-busting query parameter
 * @returns The full URL to the asset
 */
export function getCDNURL(path: string, useApiRoute = true, bustCache?: boolean): string {
    // Check if path already has cache busting parameter
    if (path.includes("?v=") || path.includes("&v=")) {
        // Already has cache busting, return as is (with leading slash if needed)
        return path.startsWith("/") ? path : `/${path}`;
    }

    // Remove any leading slashes
    const cleanPath = path.replace(/^\/+/, "");

    // Check if the path already contains the prefix to avoid duplication
    if (cleanPath.startsWith("api/cdn/")) {
        // Path already has prefix, just use it directly
        const url = `/${cleanPath}`;

        // Determine whether to bust cache based on explicit parameter or development environment
        const shouldBustCache = bustCache ?? isDevelopment;

        // Add cache busting query parameter if needed
        return shouldBustCache ? `${url}${url.includes("?") ? "&" : "?"}v=${Date.now()}` : url;
    }

    // Always use the API route
    const baseUrl = "/api/cdn/";

    // Split the path into segments and encode each segment
    const pathSegments = cleanPath.split("/");
    const encodedPath = pathSegments
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
        .join("/");

    // Create base URL with encoded path
    const url = `${baseUrl}${encodedPath}`;

    // Determine whether to bust cache based on explicit parameter or development environment
    const shouldBustCache = bustCache ?? isDevelopment;

    // Add cache busting query parameter if needed
    return shouldBustCache ? `${url}${url.includes("?") ? "&" : "?"}v=${Date.now()}` : url;
}

/**
 * Get the full URL for a chibi asset
 *
 * @param path - The path to the chibi asset
 * @param useApiRoute - Whether to use the API route (more control) or the rewrite (more efficient)
 * @param bustCache - Whether to add a cache-busting query parameter
 * @returns The full URL to the chibi asset
 */
export function getChibiAssetURL(path: string, useApiRoute = false, bustCache?: boolean): string {
    return getCDNURL(`chibis/${path}`, useApiRoute, bustCache);
}

/**
 * Types of assets that can be requested from the CDN
 */
export enum AssetType {
    OPERATOR_ART = "operators",
    ITEM_ICON = "items",
    SKILL_ICON = "skills",
    UI_ELEMENT = "ui",
    CHIBI = "chibis",
    BACKGROUND = "backgrounds",
}

/**
 * Get a CDN URL for a specific asset type
 *
 * @param type - The type of asset
 * @param path - The path to the asset within its category
 * @param useApiRoute - Whether to use the API route
 * @param bustCache - Whether to add a cache-busting query parameter
 * @returns The full URL to the asset
 */
export function getAssetURL(type: AssetType, path: string, useApiRoute = false, bustCache?: boolean): string {
    return getCDNURL(`${type}/${path}`, useApiRoute, bustCache);
}
