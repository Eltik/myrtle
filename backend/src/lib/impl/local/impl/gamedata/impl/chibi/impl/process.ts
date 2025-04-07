import type { RepoItem, SpineFiles, CharacterSkin, CharacterData } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";

/**
 * Process characters into a more frontend-friendly format
 */
export function processCharsForFrontend(items: RepoItem[]): CharacterData[] {
    // Filter for character directories (they are named like "amiya1", "amiya2", etc.)
    const charDirectories = items.filter((item) => item.contentType === "directory" && !item.name.startsWith("."));

    // Group directories by operator code
    const groupedDirs = new Map<string, RepoItem[]>();
    for (const dir of charDirectories) {
        const operatorCode = extractOperatorCode(dir.name);
        if (!groupedDirs.has(operatorCode)) {
            groupedDirs.set(operatorCode, []);
        }
        groupedDirs.get(operatorCode)!.push(dir);
    }

    // Process each operator
    return Array.from(groupedDirs.entries()).map(([operatorCode, dirs]) => {
        // Get skins/versions available from all directories for this operator
        const skins = dirs.flatMap((dir) => {
            // Each directory represents a skin variant
            const animationTypes: CharacterSkin["animationTypes"] = {};

            // Check for animation directories
            const frontDir = dir.children?.find((child) => child.contentType === "directory" && child.name.toLowerCase() === "front");
            const backDir = dir.children?.find((child) => child.contentType === "directory" && child.name.toLowerCase() === "back");
            const dormDir = dir.children?.find((child) => child.contentType === "directory" && child.name.toLowerCase() === "dorm");
            const buildDir = dir.children?.find((child) => child.contentType === "directory" && child.name.toLowerCase() === "build");

            // Check each animation type
            if (frontDir && hasRequiredSpineFiles(frontDir)) {
                animationTypes.front = extractSpineFiles(frontDir);
            }

            if (backDir && hasRequiredSpineFiles(backDir)) {
                animationTypes.back = extractSpineFiles(backDir);
            }

            // Use build directory for dorm animations if it exists
            if (buildDir && hasRequiredSpineFiles(buildDir)) {
                animationTypes.dorm = extractSpineFiles(buildDir);
            }
            // Otherwise use dorm directory if it exists
            else if (dormDir && hasRequiredSpineFiles(dormDir)) {
                animationTypes.dorm = extractSpineFiles(dormDir);
            }
            // Finally, check if the skin directory itself has spine files
            else if (hasRequiredSpineFiles(dir)) {
                animationTypes.dorm = extractSpineFiles(dir);
            }

            const hasAnySpineData = Boolean(animationTypes.dorm || animationTypes.front || animationTypes.back);

            return {
                name: dir.name,
                path: dir.path,
                hasSpineData: hasAnySpineData,
                animationTypes,
            };
        });

        return {
            operatorCode,
            name: operatorCodeToName(operatorCode),
            path: dirs[0].path, // Use the first directory's path as the base path
            skins,
        };
    });
}

/**
 * Extract just a list of operator codes
 */
export function extractOperatorList(items: RepoItem[]) {
    return Array.from(new Set(items.filter((item) => item.contentType === "directory" && !item.name.startsWith(".")).map((item) => extractOperatorCode(item.name))));
}

/**
 * Extract operator code from directory name
 */
function extractOperatorCode(dirName: string): string {
    // Remove any numbers at the end (e.g., "amiya1" -> "amiya")
    return dirName.replace(/\d+$/, "");
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
    // Capitalize first letter and replace underscores with spaces
    return code.charAt(0).toUpperCase() + code.slice(1).replace(/_/g, " ");
}
