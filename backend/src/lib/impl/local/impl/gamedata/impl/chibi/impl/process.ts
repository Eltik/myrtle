import type { RepoItem, SpineFiles, CharacterSkin, CharacterData } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";

/**
 * Process characters into a more frontend-friendly format
 */
export function processCharsForFrontend(items: RepoItem[]): CharacterData[] {
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
                .map((skin) => {
                    // Detect animation types available for this skin
                    const animationTypes: CharacterSkin["animationTypes"] = {};

                    // Check for dorm/base animations
                    if (hasRequiredSpineFiles(skin)) {
                        animationTypes.dorm = extractSpineFiles(skin);
                    }

                    // Check for combat animations (front, back)
                    // The combat animations would be in subdirectories named 'front' and 'back'
                    const frontDir = skin.children?.find((child) => child.contentType === "directory" && child.name.toLowerCase() === "front");

                    const backDir = skin.children?.find((child) => child.contentType === "directory" && child.name.toLowerCase() === "back");

                    if (frontDir && hasRequiredSpineFiles(frontDir)) {
                        animationTypes.front = extractSpineFiles(frontDir);
                    }

                    if (backDir && hasRequiredSpineFiles(backDir)) {
                        animationTypes.back = extractSpineFiles(backDir);
                    }

                    const hasAnySpineData = Boolean(animationTypes.dorm || animationTypes.front || animationTypes.back);

                    return {
                        name: skin.name,
                        path: skin.path,
                        hasSpineData: hasAnySpineData,
                        animationTypes,
                    };
                }) || [];

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
function extractSpineFiles(dir: RepoItem): SpineFiles {
    if (!dir.children) return { atlas: null, skel: null, png: null };

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
