import type { RepoItem, CharacterSkin, CharacterData, AnimationType } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";

/**
 * Clean folder name by removing $number pattern at the end
 */
function cleanFolderName(name: string): string {
    return name.replace(/\$\d+$/, '');
}

/**
 * Process characters into a more frontend-friendly format
 */
export function processCharsForFrontend(items: RepoItem[]): CharacterData[] {
    return items.map((operator) => {
        const operatorCode = cleanFolderName(operator.name);
        const characterData: CharacterData = {
            operatorCode,
            name: operatorCode,
            path: operatorCode,
            skins: [],
        };

        if (operator.children) {
            // Process all children (skins) of the operator
            operator.children.forEach((skinFolder) => {
                if (skinFolder.children) {
                    // Determine if this is a base skin or a special skin
                    const cleanedSkinFolderName = cleanFolderName(skinFolder.name);
                    const isBaseSkin = cleanedSkinFolderName === "base";
                    const skinName = isBaseSkin ? "default" : extractSkinName(skinFolder);

                    const skin: CharacterSkin = {
                        name: skinName,
                        path: skinFolder.path,
                        hasSpineData: false,
                        animationTypes: {},
                    };

                    // Map common animation folder names to the proper animation types
                    // Include both clean and normalized versions for flexibility
                    const animationTypeMapping: Record<string, AnimationType> = {
                        // Original raw folder names
                        BattleFront: "front",
                        BattleBack: "back",
                        Building: "dorm",
                        build: "dorm",
                        // Normalized folder names
                        battlefront: "front",
                        battleback: "back",
                        building: "dorm",
                        // Additional variants
                        battle_front: "front",
                        battle_back: "back",
                        front: "front",
                        back: "back",
                        dorm: "dorm",
                    };

                    let hasProperSpineData = false;

                    // Process each animation folder
                    skinFolder.children.forEach((file) => {
                        // Just use the raw folder name or a normalized version, but don't transform it
                        const fileName = file.name;
                        // Look up the animation type directly or try normalized version
                        let animType = animationTypeMapping[fileName];
                        if (!animType) {
                            // If direct lookup fails, try normalizing just the folder name
                            const normalizedFolderName = fileName.includes("dyn_illust") ? fileName : fileName.toLowerCase().split("/")[0];
                            animType = animationTypeMapping[normalizedFolderName];
                        }

                        if (animType) {
                            const filePath = file.path;
                            let processedPath = "";
                            const fileNameLower = filePath.toLowerCase();

                            // Format the path based on skin type
                            if (isBaseSkin) {
                                // For default skin: /chararts/<child path>
                                const pathParts = filePath.split(`${operatorCode}/base/`);
                                if (pathParts.length > 1) {
                                    const childPath = pathParts[1];
                                    const parts = childPath.split("/");
                                    const fileName = parts[parts.length - 1];
                                    const fileBaseName = fileName.split(".")[0];
                                    processedPath = `/chararts/${parts[0]}/${cleanFolderName(fileBaseName)}/${fileName}`;
                                } else {
                                    // Fallback if the expected pattern isn't found
                                    processedPath = `/chararts/${filePath}`;
                                }
                            } else if (filePath.includes("dynIllust") || filePath.includes("dyn_illust")) {
                                // For dynamic illustrations
                                processedPath = filePath;
                            } else {
                                // For regular skins: /skinpack/<child path>
                                const parts = filePath.split("/");
                                const fileName = parts[parts.length - 1];
                                // Extract the file path for skinpack
                                const cleanedName = cleanFolderName(file.name.split(".")[0]);
                                processedPath = `/skinpack/${cleanedName}/${fileName}`;
                            }

                            // Initialize spineFiles object for this animation type if it doesn't exist
                            if (!skin.animationTypes[animType]) {
                                skin.animationTypes[animType] = {
                                    atlas: null,
                                    skel: null,
                                    png: null,
                                };
                            }

                            // Check file extensions (case insensitive)
                            if (fileNameLower.endsWith(".atlas")) {
                                skin.animationTypes[animType]!.atlas = processedPath;
                                hasProperSpineData = true;
                            } else if (fileNameLower.endsWith(".skel")) {
                                skin.animationTypes[animType]!.skel = processedPath;
                                hasProperSpineData = true;
                            } else if (fileNameLower.endsWith(".png")) {
                                skin.animationTypes[animType]!.png = processedPath;
                                hasProperSpineData = true;
                            }
                        }
                    });

                    skin.hasSpineData = hasProperSpineData;
                    characterData.skins.push(skin);
                }
            });
        }

        return characterData;
    });
}

/**
 * Extract skin name from a skin folder's path or files
 */
function extractSkinName(skinFolder: RepoItem): string {
    // Check if it's a dynamic illustration
    if (skinFolder.path && (skinFolder.path.includes("dynIllust") || skinFolder.path.includes("dyn_illust"))) {
        // Extract the dyn_illust_char_XXXX part
        const dynMatch = skinFolder.path.match(/dyn_illust_char_[\w\d_#]+/i);
        if (dynMatch) {
            return dynMatch[0];
        }
    }

    // For regular skins, find a file path to extract from
    if (skinFolder.children) {
        for (const animFolder of skinFolder.children) {
            if (animFolder.children) {
                for (const file of animFolder.children) {
                    if (file.path) {
                        // Extract skin name from the file path
                        // Format usually like: char_109_fmout_epoque#2
                        const parts = file.path.split("/");
                        const fileName = parts[parts.length - 1];
                        // Assume format is like: <prefix>_char_XXX_YYY_<skin>.ext
                        // or char_XXX_YYY_<skin>.ext
                        const match = fileName.match(/(?:.*_)?(char_[\w\d]+(?:_[\w\d#]+)*)/i);
                        if (match) {
                            return match[1];
                        }
                    }
                }
            }
        }
    }

    // If we couldn't extract a name, use the folder name
    return cleanFolderName(skinFolder.name);
}

/**
 * Extract just a list of operator codes
 */
export function extractOperatorList(items: RepoItem[]) {
    // With our new structure, each top-level item is already an operator
    return items.map((item) => cleanFolderName(item.name));
}
