import type { RepoItem } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";

/**
 * Process characters into a more frontend-friendly format
 */
export function processCharsForFrontend(items: RepoItem[]) {
    // Filter for character directories (they start with "char_")
    const charDirectories = items.filter((item) => item.contentType === "directory" && item.name.startsWith("char_"));

    // Map to a more usable structure
    return charDirectories.map((charDir) => {
        // Extract operator code (e.g., "char_002_amiya")
        const operatorCode = charDir.name;

        // Get skins/versions available
        const skins =
            charDir.children
                ?.filter((child) => child.contentType === "directory")
                .map((skin) => ({
                    name: skin.name,
                    path: skin.path,
                    // Check if it has spine files (atlas, skel, png)
                    hasSpineData: hasRequiredSpineFiles(skin),
                    spineFiles: extractSpineFiles(skin),
                })) || [];

        return {
            operatorCode,
            name: operatorCodeToName(operatorCode),
            path: charDir.path,
            skins,
        };
    });
}

/**
 * Extract just a list of operator codes
 */
export function extractOperatorList(items: RepoItem[]) {
    return items.filter((item) => item.contentType === "directory" && item.name.startsWith("char_")).map((item) => item.name);
}

/**
 * Check if a directory has the required spine files
 */
function hasRequiredSpineFiles(dir: RepoItem) {
    if (!dir.children) return false;

    const hasAtlas = dir.children.some((file) => file.name.endsWith(".atlas"));
    const hasSkel = dir.children.some((file) => file.name.endsWith(".skel"));
    const hasPng = dir.children.some((file) => file.name.endsWith(".png"));

    return hasAtlas && hasSkel && hasPng;
}

/**
 * Extract spine files information
 */
function extractSpineFiles(dir: RepoItem) {
    if (!dir.children) return {};

    const atlas = dir.children.find((file) => file.name.endsWith(".atlas"));
    const skel = dir.children.find((file) => file.name.endsWith(".skel"));
    const png = dir.children.find((file) => file.name.endsWith(".png"));

    return {
        atlas: atlas?.path || null,
        skel: skel?.path || null,
        png: png?.path || null,
    };
}

/**
 * Convert operator code to a more readable name
 */
function operatorCodeToName(code: string) {
    // Remove "char_" prefix
    const withoutPrefix = code.replace("char_", "");

    // Extract the name part (after the numbers)
    const namePart = withoutPrefix.split("_").slice(1).join("_");

    // Capitalize first letter and replace underscores with spaces
    return namePart.charAt(0).toUpperCase() + namePart.slice(1).replace(/_/g, " ");
}
