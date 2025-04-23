import type { RepoItem, SpineFiles, CharacterSkin, CharacterData } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";

/**
 * Process characters into a more frontend-friendly format
 */
export function processCharsForFrontend(items: RepoItem[]): CharacterData[] {
    // The items are already organized by operator code in our new structure
    return items
        .map((operatorDir) => {
            // Each top-level directory is an operator
            const operatorCode = operatorDir.name;

            // Process skins for this operator
            const skins: CharacterSkin[] = [];

            // Process base and skin directories
            if (operatorDir.children) {
                // The base skin - which now contains front and back views
                const baseDir = operatorDir.children.find((child) => child.contentType === "dir" && child.name === "base");

                if (baseDir && baseDir.children) {
                    // Process animation views - mapping folder names to animation types
                    // BattleFront -> front
                    // BattleBack -> back
                    // Building -> dorm
                    const frontDir = baseDir.children.find((child) => child.contentType === "dir" && child.name === "BattleFront");
                    const backDir = baseDir.children.find((child) => child.contentType === "dir" && child.name === "BattleBack");
                    const dormDir = baseDir.children.find((child) => child.contentType === "dir" && child.name === "Building");

                    // Extract spine files for different animation types
                    const frontSpineFiles = frontDir ? extractSpineFiles(frontDir) : { atlas: null, skel: null, png: null };
                    const backSpineFiles = backDir ? extractSpineFiles(backDir) : { atlas: null, skel: null, png: null };
                    const dormSpineFiles = dormDir ? extractSpineFiles(dormDir) : { atlas: null, skel: null, png: null };

                    // Only add if at least one view has complete spine files
                    const hasFrontComplete = hasCompleteSpineFiles(frontSpineFiles);
                    const hasBackComplete = hasCompleteSpineFiles(backSpineFiles);
                    const hasDormComplete = hasCompleteSpineFiles(dormSpineFiles);

                    if (hasFrontComplete || hasBackComplete || hasDormComplete) {
                        const baseSkin: CharacterSkin = {
                            name: "default",
                            path: operatorCode,
                            hasSpineData: true,
                            animationTypes: {},
                        };

                        // Add front view if complete
                        if (hasFrontComplete) {
                            // Update paths to match actual file structure
                            if (frontSpineFiles.atlas) {
                                frontSpineFiles.atlas = `chararts/BattleFront/${frontSpineFiles.atlas.split('/').pop()}`;
                            }
                            if (frontSpineFiles.skel) {
                                frontSpineFiles.skel = `chararts/BattleFront/${frontSpineFiles.skel.split('/').pop()}`;
                            }
                            if (frontSpineFiles.png) {
                                frontSpineFiles.png = `chararts/BattleFront/${frontSpineFiles.png.split('/').pop()}`;
                            }
                            baseSkin.animationTypes.front = frontSpineFiles;
                        }

                        // Add back view if complete
                        if (hasBackComplete) {
                            // Update paths to match actual file structure
                            if (backSpineFiles.atlas) {
                                backSpineFiles.atlas = `chararts/BattleBack/${backSpineFiles.atlas.split('/').pop()}`;
                            }
                            if (backSpineFiles.skel) {
                                backSpineFiles.skel = `chararts/BattleBack/${backSpineFiles.skel.split('/').pop()}`;
                            }
                            if (backSpineFiles.png) {
                                backSpineFiles.png = `chararts/BattleBack/${backSpineFiles.png.split('/').pop()}`;
                            }
                            baseSkin.animationTypes.back = backSpineFiles;
                        }

                        // Add dorm view if complete
                        if (hasDormComplete) {
                            // Update paths to match actual file structure
                            if (dormSpineFiles.atlas) {
                                dormSpineFiles.atlas = `chararts/Building/${dormSpineFiles.atlas.split('/').pop()}`;
                            }
                            if (dormSpineFiles.skel) {
                                dormSpineFiles.skel = `chararts/Building/${dormSpineFiles.skel.split('/').pop()}`;
                            }
                            if (dormSpineFiles.png) {
                                dormSpineFiles.png = `chararts/Building/${dormSpineFiles.png.split('/').pop()}`;
                            }
                            baseSkin.animationTypes.dorm = dormSpineFiles;
                        }

                        skins.push(baseSkin);
                    }
                }

                // The skin variants
                const skinDir = operatorDir.children.find((child) => child.contentType === "dir" && child.name === "skin");

                if (skinDir && skinDir.children) {
                    // Ensure we're handling all possible file types, especially in non-standard formats
                    const allFiles = skinDir.children.filter(file => file.contentType === "file");
                    
                    // Check for spine files directly in the skin directory (not in subdirectories)
                    const directSpineFiles = extractSpineFiles({ name: "direct", path: skinDir.path, contentType: "dir", children: allFiles });
                    if (hasCompleteSpineFiles(directSpineFiles)) {
                        // Add as default skin if no default exists and no skin name can be extracted
                        const directSkin: CharacterSkin = {
                            name: "default",
                            path: operatorCode,
                            hasSpineData: true,
                            animationTypes: {
                                // If no specific type is detected, default to front
                                front: {
                                    atlas: directSpineFiles.atlas ? `skinpack/${directSpineFiles.atlas.split('/').pop()}` : null,
                                    skel: directSpineFiles.skel ? `skinpack/${directSpineFiles.skel.split('/').pop()}` : null,
                                    png: directSpineFiles.png ? `skinpack/${directSpineFiles.png.split('/').pop()}` : null,
                                }
                            },
                        };
                        
                        // Only add if we don't already have a default skin
                        if (!skins.some(skin => skin.name === "default")) {
                            skins.push(directSkin);
                        }
                    }
                    
                    // Group files by animation type
                    const frontFiles = allFiles.filter(file => file.name.includes("BattleFront"));
                    const backFiles = allFiles.filter(file => file.name.includes("BattleBack"));
                    const dormFiles = allFiles.filter(file => file.name.includes("Building"));
                    
                    // Handle files that don't have explicit type in their name
                    const unclassifiedFiles = allFiles.filter(file => 
                        !file.name.includes("BattleFront") && 
                        !file.name.includes("BattleBack") && 
                        !file.name.includes("Building"));
                    
                    // Try to detect file types based on naming patterns
                    const additionalFrontFiles = unclassifiedFiles.filter(file => !file.name.startsWith("build_"));
                    const additionalDormFiles = unclassifiedFiles.filter(file => file.name.startsWith("build_"));
                    
                    // Add these to our classified files
                    frontFiles.push(...additionalFrontFiles);
                    dormFiles.push(...additionalDormFiles);
                    
                    // Process each group separately to identify skins
                    const skinGroups = new Map<string, {
                        front: RepoItem[],
                        back: RepoItem[],
                        dorm: RepoItem[]
                    }>();
                    
                    // Helper function to add files to skin groups
                    const processFilesForType = (files: RepoItem[], type: 'front' | 'back' | 'dorm') => {
                        for (const file of files) {
                            // Extract full skin identifier from filename (including char_XXX_YYY part)
                            const skinIdentifier = extractFullSkinIdentifier(file.name);
                            if (!skinGroups.has(skinIdentifier)) {
                                skinGroups.set(skinIdentifier, {
                                    front: [],
                                    back: [],
                                    dorm: []
                                });
                            }
                            skinGroups.get(skinIdentifier)?.[type].push(file);
                        }
                    };
                    
                    // Process each animation type
                    processFilesForType(frontFiles, 'front');
                    processFilesForType(backFiles, 'back');
                    processFilesForType(dormFiles, 'dorm');
                    
                    // Now process each skin group
                    for (const [skinIdentifier, files] of skinGroups.entries()) {
                        // Skip if this is a default skin and we already have one
                        if (skinIdentifier === "default" && skins.some(skin => skin.name === "default")) {
                            continue;
                        }
                        
                        // Create virtual directory entries for each animation type
                        const frontDir: RepoItem | null = files.front.length > 0 ? {
                            name: "front",
                            path: `${skinDir.path}/${skinIdentifier}/front`,
                            contentType: "dir",
                            children: files.front
                        } : null;
                        
                        const backDir: RepoItem | null = files.back.length > 0 ? {
                            name: "back",
                            path: `${skinDir.path}/${skinIdentifier}/back`,
                            contentType: "dir",
                            children: files.back
                        } : null;
                        
                        const dormDir: RepoItem | null = files.dorm.length > 0 ? {
                            name: "dorm",
                            path: `${skinDir.path}/${skinIdentifier}/dorm`,
                            contentType: "dir",
                            children: files.dorm
                        } : null;
                        
                        // Extract spine files for each animation type
                        const frontSpineFiles = frontDir ? extractSpineFiles(frontDir) : { atlas: null, skel: null, png: null };
                        const backSpineFiles = backDir ? extractSpineFiles(backDir) : { atlas: null, skel: null, png: null };
                        const dormSpineFiles = dormDir ? extractSpineFiles(dormDir) : { atlas: null, skel: null, png: null };
                        
                        // Check if any animation type has complete spine files
                        const hasFrontComplete = hasCompleteSpineFiles(frontSpineFiles);
                        const hasBackComplete = hasCompleteSpineFiles(backSpineFiles);
                        const hasDormComplete = hasCompleteSpineFiles(dormSpineFiles);
                        
                        if (hasFrontComplete || hasBackComplete || hasDormComplete) {
                            // Use the full identifier as the skin name
                            const skin: CharacterSkin = {
                                name: skinIdentifier === "default" ? "default" : skinIdentifier,
                                path: operatorCode,
                                hasSpineData: true,
                                animationTypes: {},
                            };
                            
                            // Add animation types if complete
                            if (hasFrontComplete) {
                                // Update paths to use the proper format
                                if (frontSpineFiles.atlas) {
                                    frontSpineFiles.atlas = `skinpack/BattleFront/${frontSpineFiles.atlas.split('/').pop()}`;
                                }
                                if (frontSpineFiles.skel) {
                                    frontSpineFiles.skel = `skinpack/BattleFront/${frontSpineFiles.skel.split('/').pop()}`;
                                }
                                if (frontSpineFiles.png) {
                                    frontSpineFiles.png = `skinpack/BattleFront/${frontSpineFiles.png.split('/').pop()}`;
                                }
                                skin.animationTypes.front = frontSpineFiles;
                            }
                            
                            if (hasBackComplete) {
                                // Update paths to use the proper format
                                if (backSpineFiles.atlas) {
                                    backSpineFiles.atlas = `skinpack/BattleBack/${backSpineFiles.atlas.split('/').pop()}`;
                                }
                                if (backSpineFiles.skel) {
                                    backSpineFiles.skel = `skinpack/BattleBack/${backSpineFiles.skel.split('/').pop()}`;
                                }
                                if (backSpineFiles.png) {
                                    backSpineFiles.png = `skinpack/BattleBack/${backSpineFiles.png.split('/').pop()}`;
                                }
                                skin.animationTypes.back = backSpineFiles;
                            }
                            
                            if (hasDormComplete) {
                                // Update paths to use the proper format
                                if (dormSpineFiles.atlas) {
                                    dormSpineFiles.atlas = `skinpack/Building/${dormSpineFiles.atlas.split('/').pop()}`;
                                }
                                if (dormSpineFiles.skel) {
                                    dormSpineFiles.skel = `skinpack/Building/${dormSpineFiles.skel.split('/').pop()}`;
                                }
                                if (dormSpineFiles.png) {
                                    dormSpineFiles.png = `skinpack/Building/${dormSpineFiles.png.split('/').pop()}`;
                                }
                                skin.animationTypes.dorm = dormSpineFiles;
                            }
                            
                            skins.push(skin);
                        }
                    }
                }
            }

            // Only return operators that have at least one valid skin OR
            // return all operators to ensure we don't lose any
            return {
                operatorCode,
                name: operatorCodeToName(operatorCode),
                path: operatorDir.path,
                skins, // Even if empty, we'll return it to preserve the operator
            };
        })
        .filter(item => item !== null) as CharacterData[]; // Just filter out null entries, allow empty skins
}

/**
 * Extract the full skin identifier from a file name, including the char_XXX_YYY part
 */
function extractFullSkinIdentifier(fileName: string): string {
    // For default skins
    if (!fileName.includes("_") && !fileName.includes("#")) {
        return "default";
    }
    
    // Building/dorm files with build_ prefix
    if (fileName.startsWith("build_")) {
        const match = fileName.match(/build_(char_\d+_[a-z0-9]+(?:_[a-z0-9]+(?:#\d+)?)?)(?:\.|\$|_)/i);
        if (match && match[1]) {
            return match[1].toLowerCase();
        }
    }
    
    // For files with char_ prefix (most operators)
    const charMatch = fileName.match(/(char_\d+_[a-z0-9]+(?:_[a-z0-9]+(?:#\d+)?)?)(?:\.|\$|_)/i);
    if (charMatch && charMatch[1]) {
        return charMatch[1].toLowerCase();
    }
    
    // Special handling for specific patterns like sale#X
    const specialMatch = fileName.match(/(char_\d+_[a-z0-9]+)_([a-z0-9]+#\d+)/i);
    if (specialMatch) {
        return `${specialMatch[1].toLowerCase()}_${specialMatch[2].toLowerCase()}`;
    }
    
    // If no pattern matched, return default
    return "default";
}

/**
 * Extract just a list of operator codes
 */
export function extractOperatorList(items: RepoItem[]) {
    // With our new structure, each top-level item is already an operator
    return items.map((item) => item.name);
}

/**
 * Extract spine files information from a directory or collection of files
 */
function extractSpineFiles(dir: RepoItem): SpineFiles {
    const children = dir.children || [];

    // Find the spine files with more flexible matching
    const atlas = children.find((file) => file.name.endsWith(".atlas"));
    const skel = children.find((file) => file.name.endsWith(".skel"));
    
    // Some files might have variants with suffixes, try to find the best match
    let png = children.find((file) => file.name.endsWith(".png") && !file.name.includes("$"));
    if (!png) {
        // If no direct match, try with $ suffix (common in exported files)
        png = children.find((file) => file.name.endsWith(".png"));
    }

    return {
        atlas: atlas?.path || null,
        skel: skel?.path || null,
        png: png?.path || null,
    };
}

/**
 * Check if spine files are complete (has all required files)
 */
function hasCompleteSpineFiles(spineFiles: SpineFiles): boolean {
    return Boolean(spineFiles.atlas && spineFiles.skel && spineFiles.png);
}

/**
 * Convert operator code to a more readable name
 */
function operatorCodeToName(code: string) {
    // For now, we just return the code as the name
    // In a real implementation, you might want to look up the actual operator name
    return code;
}
