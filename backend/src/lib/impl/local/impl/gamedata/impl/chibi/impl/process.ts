import type { RepoItem, SpineFiles, CharacterSkin, CharacterData } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";

/**
 * Process characters into a more frontend-friendly format
 */
export function processCharsForFrontend(items: RepoItem[]): CharacterData[] {
    // The items are already organized by operator code in our new structure
    return items.map((operatorDir) => {
        // Each top-level directory is an operator
        const operatorCode = operatorDir.name;

        // Process skins for this operator
        const skins: CharacterSkin[] = [];

        // Process base and skin directories
        if (operatorDir.children) {
            // The base skin
            const baseDir = operatorDir.children.find((child) => child.contentType === "dir" && child.name === "base");

            if (baseDir) {
                const baseSkin: CharacterSkin = {
                    name: "default",
                    path: baseDir.path,
                    hasSpineData: true,
                    animationTypes: {},
                };

                // Add spine files for base animation
                baseSkin.animationTypes.front = extractSpineFiles(baseDir);

                skins.push(baseSkin);
            }

            // The skin variants
            const skinDir = operatorDir.children.find((child) => child.contentType === "dir" && child.name === "skin");

            if (skinDir && skinDir.children) {
                // Each set of spine files in skins represents a different skin
                // Group files by their name prefix (before the extension)
                const skinGroups = groupFilesByPrefix(skinDir.children);

                // Process each skin group
                for (const [prefix, files] of skinGroups.entries()) {
                    const skinFiles: RepoItem = {
                        name: prefix,
                        path: `${skinDir.path}/${prefix}`,
                        contentType: "dir",
                        children: files,
                    };

                    const skin: CharacterSkin = {
                        name: prefix,
                        path: skinFiles.path,
                        hasSpineData: true,
                        animationTypes: {
                            front: extractSpineFiles(skinFiles),
                        },
                    };

                    skins.push(skin);
                }
            }
        }

        return {
            operatorCode,
            name: operatorCodeToName(operatorCode),
            path: operatorDir.path,
            skins,
        };
    });
}

/**
 * Extract just a list of operator codes
 */
export function extractOperatorList(items: RepoItem[]) {
    // With our new structure, each top-level item is already an operator
    return items.map((item) => item.name);
}

/**
 * Group files by their name prefix (part before the extension)
 */
function groupFilesByPrefix(files: RepoItem[]): Map<string, RepoItem[]> {
    const groups = new Map<string, RepoItem[]>();

    for (const file of files) {
        if (file.contentType === "file") {
            // Extract prefix (before the extension)
            const prefix = file.name.split(".")[0];

            if (!groups.has(prefix)) {
                groups.set(prefix, []);
            }

            groups.get(prefix)!.push(file);
        }
    }

    return groups;
}

/**
 * Extract spine files information from a directory or collection of files
 */
function extractSpineFiles(dir: RepoItem): SpineFiles {
    const children = dir.children || [];

    const atlas = children.find((file) => file.name.endsWith(".atlas"));
    const skel = children.find((file) => file.name.endsWith(".skel"));
    const png = children.find((file) => file.name.endsWith(".png"));

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
    // For now, we just return the code as the name
    // In a real implementation, you might want to look up the actual operator name
    return code;
}
