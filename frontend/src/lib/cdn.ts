/**
 * Utilities for working with the CDN service
 */

/**
 * Get the CDN URL for a given asset path
 *
 * @param path - The path to the asset within the CDN
 * @param useApiRoute - Whether to use the API route (more control) or the rewrite (more efficient)
 * @returns The full URL to the asset
 */
export function getCDNURL(path: string, useApiRoute = false): string {
    // Remove any leading slashes
    const cleanPath = path.replace(/^\/+/, "");

    // Choose between API route or direct rewrite
    const baseUrl = useApiRoute ? "/api/cdn/" : "/assets/";

    return `${baseUrl}${cleanPath}`;
}

/**
 * Get the full URL for a chibi asset
 *
 * @param path - The path to the chibi asset
 * @param useApiRoute - Whether to use the API route (more control) or the rewrite (more efficient)
 * @returns The full URL to the chibi asset
 */
export function getChibiAssetURL(path: string, useApiRoute = false): string {
    return getCDNURL(`chibis/${path}`, useApiRoute);
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
 * @returns The full URL to the asset
 */
export function getAssetURL(type: AssetType, path: string, useApiRoute = false): string {
    return getCDNURL(`${type}/${path}`, useApiRoute);
}
