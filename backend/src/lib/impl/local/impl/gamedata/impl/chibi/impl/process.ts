import type { RepoItem, CharacterSkin, CharacterData, AnimationType } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";

/**
 * Clean folder name by removing $number pattern at the end
 */
function cleanFolderName(name: string): string {
    // Also clean potential subtype prefixes like BattleFront/
    const baseName = name.split("/").pop() || name;
    // Remove common extensions and $number suffixes
    return baseName.replace(/(\.(png|atlas|skel)|\$\d+)$/i, "");
}

/**
 * Normalize skin names for merging (e.g., remove .png)
 */
function normalizeSkinName(name: string): string {
    return name.replace(/\.(png|atlas|skel)$/i, "");
}

/**
 * Process characters into a more frontend-friendly format
 */
export function processCharsForFrontend(items: RepoItem[]): CharacterData[] {
    return items.map((operator) => {
        const operatorCode = cleanFolderName(operator.name);
        const initialSkins: CharacterSkin[] = [];

        // Find the main folders for base, skin, and dynIllust
        const baseFolder = operator.children?.find((c) => c.name === "base");
        const skinParentFolder = operator.children?.find((c) => c.name === "skin");
        const dynIllustFolder = operator.children?.find((c) => c.name === "dynIllust");

        // --- Process Base Skin (default) ---
        if (baseFolder?.children?.length) {
            const baseSkin: CharacterSkin = {
                name: "default",
                path: baseFolder.path,
                hasSpineData: false,
                animationTypes: {},
            };
            processAnimationFiles(baseFolder.children, baseSkin, operatorCode, true);
            if (Object.keys(baseSkin.animationTypes).length > 0) {
                // Add base skin only if it has any animation data
                initialSkins.push(baseSkin);
            }
        }

        // --- Process Regular Skins ---
        if (skinParentFolder?.children?.length) {
            skinParentFolder.children.forEach((specificSkinFolder) => {
                if (specificSkinFolder.contentType === "dir" && specificSkinFolder.children?.length) {
                    const skinName = cleanFolderName(specificSkinFolder.name); // Skin name is the folder name
                    const skin: CharacterSkin = {
                        name: skinName,
                        path: specificSkinFolder.path,
                        hasSpineData: false,
                        animationTypes: {},
                    };
                    processAnimationFiles(specificSkinFolder.children, skin, operatorCode, false);
                    if (Object.keys(skin.animationTypes).length > 0) {
                        initialSkins.push(skin);
                    }
                }
            });
        }

        // --- Process Dynamic Illustrations ---
        if (dynIllustFolder?.children?.length) {
            const containsFiles = dynIllustFolder.children.some((item) => item.contentType === "file");

            if (containsFiles) {
                const dynSkinName = "dynamic_illustration";
                const dynSkin: CharacterSkin = {
                    name: dynSkinName,
                    path: dynIllustFolder.path,
                    hasSpineData: false,
                    animationTypes: {},
                };
                processAnimationFiles(dynIllustFolder.children, dynSkin, operatorCode, false, true);
                if (Object.keys(dynSkin.animationTypes).length > 0) {
                    initialSkins.push(dynSkin);
                }
            } else {
                dynIllustFolder.children.forEach((specificDynIllustFolder) => {
                    if (specificDynIllustFolder.contentType === "dir" && specificDynIllustFolder.children?.length) {
                        const dynSkinName = cleanFolderName(specificDynIllustFolder.name);
                        const dynSkin: CharacterSkin = {
                            name: dynSkinName,
                            path: specificDynIllustFolder.path,
                            hasSpineData: false,
                            animationTypes: {},
                        };
                        processAnimationFiles(specificDynIllustFolder.children, dynSkin, operatorCode, false, true);
                        if (Object.keys(dynSkin.animationTypes).length > 0) {
                            initialSkins.push(dynSkin);
                        }
                    }
                });
            }
        }

        // --- Merge Skins with similar names (e.g., "it_1" and "it_1.png") ---
        const mergedSkinsMap = new Map<string, CharacterSkin>();

        initialSkins.forEach((currentSkin) => {
            const normalizedName = normalizeSkinName(currentSkin.name);
            const existingSkin = mergedSkinsMap.get(normalizedName);

            if (existingSkin) {
                // Merge currentSkin into existingSkin
                existingSkin.path = existingSkin.path || currentSkin.path; // Keep the first path found
                existingSkin.hasSpineData = existingSkin.hasSpineData || currentSkin.hasSpineData;

                // Merge animationTypes, prioritizing non-null values
                (Object.keys(currentSkin.animationTypes) as AnimationType[]).forEach((animType) => {
                    const currentAnimData = currentSkin.animationTypes[animType];
                    if (!currentAnimData) return; // Skip if current skin has no data for this type

                    if (!existingSkin.animationTypes[animType]) {
                        // If existing skin doesn't have this type, copy it over
                        existingSkin.animationTypes[animType] = { ...currentAnimData };
                    } else {
                        // Merge file paths, keeping existing non-null ones
                        const existingAnimData = existingSkin.animationTypes[animType]!;
                        existingAnimData.atlas = existingAnimData.atlas ?? currentAnimData.atlas;
                        existingAnimData.skel = existingAnimData.skel ?? currentAnimData.skel;
                        existingAnimData.png = existingAnimData.png ?? currentAnimData.png;
                    }
                });
            } else {
                // Add the new skin to the map (create a deep copy to avoid modification issues)
                mergedSkinsMap.set(normalizedName, JSON.parse(JSON.stringify(currentSkin)));
            }
        });

        // Final character data structure
        const characterData: CharacterData = {
            operatorCode,
            name: operatorCode,
            path: operatorCode,
            skins: Array.from(mergedSkinsMap.values()),
        };

        return characterData;
    });
}

// Helper function to process animation files for a given skin
function processAnimationFiles(files: RepoItem[], skin: CharacterSkin, operatorCode: string, isBaseSkin: boolean, isDynIllust: boolean = false) {
    const animationTypeMapping: Record<string, AnimationType> = {
        BattleFront: "front",
        BattleBack: "back",
        Building: "dorm",
        battlefront: "front",
        battleback: "back",
        building: "dorm",
        build: "dorm",
        battle_front: "front",
        battle_back: "back",
        front: "front",
        back: "back",
        dorm: "dorm",
        // Add mapping for potential dynIllust structures if needed
        dyn_illust: "dynamic", // Example mapping
    };

    let hasProperSpineData = false;

    files.forEach((file) => {
        if (file.contentType !== "file" || !file.path) return;

        // Extract animation type from the file's display name (e.g., "BattleFront/char_xxx.png")
        const nameParts = file.name.split("/");
        const animFolder = nameParts[0].toLowerCase();
        let animType = animationTypeMapping[nameParts[0]] || animationTypeMapping[animFolder]; // Check original and lowercased

        // Special handling for DynIllust if it doesn't use subfolders like BattleFront
        if (!animType && isDynIllust) {
            // Attempt to map based on file naming convention or assume a default type
            if (file.name.toLowerCase().includes("dyn_illust")) {
                animType = "dynamic"; // Assign a type for dynamic illustrations
            }
        }
        if (!animType && file.name.toLowerCase().includes("dyn_illust")) {
            animType = "dynamic";
        }

        if (animType) {
            let processedPath = "";
            const fileName = file.path.split("/").pop() || file.name;
            const fileBaseName = fileName.split(".").slice(0, -1).join(".");
            const cleanedFileBaseName = cleanFolderName(fileBaseName);
            const fileNameLower = fileName.toLowerCase();

            // Format the path based on skin type
            if (isBaseSkin) {
                // Base skin path: /chararts/<animTypeFolder>/<cleanedFileBaseName>/<fileName>
                // e.g., /chararts/BattleFront/char_350_surtr/char_350_surtr.png
                const animTypeFolder = nameParts[0]; // Use the original case folder name
                processedPath = `/chararts/${animTypeFolder}/${cleanedFileBaseName}/${fileName}`;
            } else if (isDynIllust) {
                // Dynamic illustration path: /arts/dynchars/DynIllust/<subfolder>/<cleanedFileBaseName>/<fileName>
                // Path in JSON: char_350_surtr/dynIllust/dyn_illust_char_350_surtr_summer_9.png
                // We need the subfolder structure if it exists, otherwise just the file.
                // The crawl currently seems flat, let's assume flat for now:
                // Might need adjustment based on actual DynIllust file structure on disk.
                // Assuming the crawler puts files directly in dynIllust folder:
                const dynIllustBase = `/arts/dynchars/DynIllust`;
                // Try to extract potential subfolder from the filename if convention exists
                const illustMatch = cleanedFileBaseName.match(/dyn_illust_(char_\d+_[a-zA-Z0-9]+)/i);
                const opSubfolder = illustMatch ? illustMatch[1] : operatorCode; // Use operatorCode as fallback
                processedPath = `${dynIllustBase}/${opSubfolder}/${cleanedFileBaseName}/${fileName}`;
            } else {
                // Regular skin path: /skinpack/<animTypeFolder>/<cleanedFileBaseName>/<fileName>
                // e.g., /skinpack/BattleFront/char_350_surtr_it_1/char_350_surtr_it_1.png
                const animTypeFolder = nameParts[0]; // Use the original case folder name (e.g., BattleFront)
                processedPath = `/skinpack/${animTypeFolder}/${cleanedFileBaseName}/${fileName}`;
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
                // We consider it having data even if only PNG is present for some basic display
                hasProperSpineData = true;
            }
        }
    });

    skin.hasSpineData = hasProperSpineData;
}

/**
 * Extract just a list of operator codes
 */
export function extractOperatorList(items: RepoItem[]) {
    // With our new structure, each top-level item is already an operator
    return items.map((item) => cleanFolderName(item.name));
}
