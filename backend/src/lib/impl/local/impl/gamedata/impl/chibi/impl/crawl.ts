import type { RepoItem } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";
import colors from "colors";
import fs from "fs";
import path from "path";
import { env } from "../../../../../../../../env";

// Base assets path - use environment variable if available or fallback to default
const BASE_ASSETS_PATH = env.UNPACKED_DIR 
    ? path.resolve(env.UNPACKED_DIR) 
    : path.resolve(process.cwd(), "..", "assets", "Unpacked");

// Paths for different types of chibi assets
export const CHARARTS_PATH = path.resolve(BASE_ASSETS_PATH, "chararts");
export const SKINPACK_PATH = path.resolve(BASE_ASSETS_PATH, "skinpack");
export const DYNILLUST_PATH = path.resolve(BASE_ASSETS_PATH, "arts", "dynchars", "DynIllust");

// Subdirectories for different sprite types - ensure these match the ones used in process.ts
// BattleFront maps to front animation type
// BattleBack maps to back animation type
// Building maps to dorm animation type
const SUBDIRS = ["BattleBack", "BattleFront", "Building"];

// Path to store the chibi data
const CHIBI_DATA_PATH = path.resolve(process.cwd(), "data", "chibi-data.json");

// Ensure the data directory exists
const ensureDataDir = () => {
    const dataDir = path.dirname(CHIBI_DATA_PATH);
    if (!fs.existsSync(dataDir)) {
        fs.mkdirSync(dataDir, { recursive: true });
    }
};

// Save chibi data to file
const saveChibiData = (data: RepoItem[]): void => {
    ensureDataDir();
    try {
        fs.writeFileSync(CHIBI_DATA_PATH, JSON.stringify(data, null, 2));
        console.log(colors.green(`✅ Saved chibi data to ${CHIBI_DATA_PATH}`));
    } catch (error) {
        console.error(colors.red(`❌ Error saving chibi data: ${error}`));
    }
};

// Check if the paths exist and warn if not
const checkPaths = () => {
    const paths = [CHARARTS_PATH, SKINPACK_PATH, DYNILLUST_PATH];
    paths.forEach(p => {
        if (!fs.existsSync(p)) {
            console.log(colors.yellow(`⚠️ Directory not found: ${p}`));
        }
    });
};

// Main crawl function that crawls all chibi asset directories
export const crawlLocalChibis = async (): Promise<RepoItem[]> => {
    try {
        console.log(colors.cyan(`ℹ️ Crawling local chibi assets...`));
        checkPaths();

        const operatorMap = new Map<string, RepoItem>();
        let totalFiles = 0;
        let matchedFiles = 0;

        // Track complete file sets by operator and animation type
        const fileTracker = new Map<
            string,
            {
                base: { [subdir: string]: boolean };
                skin: Map<string, { [subdir: string]: boolean }>;
                dynIllust: Map<string, boolean>;
            }
        >();

        // Function to process a file
        const processFile = (filePath: string, fileName: string, type: 'base' | 'skin' | 'dynIllust', subdir?: string) => {
            // Skip monobehaviour files
            if (fileName.toLowerCase().startsWith("monobehaviour_")) {
                return;
            }

            // Skip files with $1, $2, etc. (but allow $0)
            if (fileName.match(/\$[1-9]\d*\./)) {
                return;
            }

            totalFiles++;

            // Log progress
            if (totalFiles < 50 || totalFiles % 1000 === 0) {
                console.log(colors.gray(`🔍 Processing file: ${fileName}`));
            }

            // Match operator ID
            let match;
            let opId = "";
            let skinName = "";
            
            if (type === 'dynIllust') {
                // Handle dynamic illustrations
                match = fileName.match(/dyn_illust_(char_\d+_[a-zA-Z0-9]+)/i);
                if (match) {
                    opId = match[1].toLowerCase();
                    
                    // Extract skin name if present
                    const skinMatch = fileName.match(/dyn_illust_char_\d+_[a-zA-Z0-9]+_(.+?)(?:\.|$)/i);
                    if (skinMatch && skinMatch[1]) {
                        skinName = skinMatch[1].toLowerCase();
                    }
                }
            } else {
                // Handle regular chibis and skins
                match = fileName.match(/(build_char_|char_)(\d+_[a-zA-Z0-9]+)/i);
                if (match) {
                    const prefix = "char_";
                    const id = match[2].toLowerCase();
                    opId = `${prefix}${id}`;
                    
                    // Extract skin name for skin animations
                    if (type === 'skin') {
                        const skinMatch = fileName.match(/(?:char|build_char)_\d+_[a-z0-9]+_(.+?)(?:\.|#|\$|_Atlas|_SkeletonData|$)/i);
                        if (skinMatch && skinMatch[1]) {
                            skinName = skinMatch[1].toLowerCase();
                        }
                    }
                }
            }

            if (match && opId) {
                matchedFiles++;

                // Initialize tracker for this operator if needed
                if (!fileTracker.has(opId)) {
                    fileTracker.set(opId, {
                        base: {},
                        skin: new Map(),
                        dynIllust: new Map()
                    });
                }

                const tracker = fileTracker.get(opId)!;

                // Update tracker based on file type
                if (type === 'base' && subdir) {
                    tracker.base[subdir] = true;
                } else if (type === 'skin' && subdir) {
                    if (!tracker.skin.has(skinName)) {
                        tracker.skin.set(skinName, {});
                    }
                    const skinData = tracker.skin.get(skinName);
                    if (skinData && subdir) {
                        skinData[subdir] = true;
                    }
                } else if (type === 'dynIllust') {
                    tracker.dynIllust.set(skinName || 'base', true);
                }

                // Create operator if it doesn't exist
                if (!operatorMap.has(opId)) {
                    operatorMap.set(opId, {
                        name: opId,
                        path: opId,
                        contentType: "dir",
                        children: [
                            // Create directories for different asset types
                            {
                                name: "base",
                                path: `${opId}/base`,
                                contentType: "dir",
                                children: [],
                            },
                            {
                                name: "skin",
                                path: `${opId}/skin`,
                                contentType: "dir",
                                children: [],
                            },
                            {
                                name: "dynIllust",
                                path: `${opId}/dynIllust`,
                                contentType: "dir",
                                children: [],
                            }
                        ],
                    });
                }

                // Get the operator
                const operator = operatorMap.get(opId)!;

                // Get the correct folder (base, skin, or dynIllust)
                const folderName = type;
                const folder = operator.children!.find((c) => c.name === folderName)!;

                // Add subdirectory information to the file name if available
                const displayName = subdir ? `${subdir}/${fileName}` : fileName;

                // Add the file to the folder
                folder.children!.push({
                    name: displayName,
                    path: `${opId}/${folderName}/${displayName}`,
                    contentType: "file",
                });
            }
        };

        // Function to crawl a directory and process files
        const crawlDir = async (baseDir: string, type: 'base' | 'skin' | 'dynIllust') => {
            if (!fs.existsSync(baseDir)) {
                console.log(colors.yellow(`⚠️ Directory not found: ${baseDir}`));
                return;
            }

            if (type === 'dynIllust') {
                // Process dynamic illustrations directory
                const readDynIllustDir = (dirPath: string) => {
                    if (!fs.existsSync(dirPath)) return;
                    
                    const items = fs.readdirSync(dirPath);
                    for (const item of items) {
                        const fullPath = path.join(dirPath, item);
                        const stats = fs.statSync(fullPath);
                        
                        if (stats.isDirectory()) {
                            // Process files inside dynIllust subdirectories
                            const subFiles = fs.readdirSync(fullPath);
                            for (const file of subFiles) {
                                processFile(path.join(fullPath, file), file, 'dynIllust');
                            }
                        } else if (stats.isFile()) {
                            // Process dynIllust files directly in the main directory
                            processFile(fullPath, item, 'dynIllust');
                        }
                    }
                };
                
                readDynIllustDir(baseDir);
            } else {
                // Process normal chibi and skin directories with their respective subdirectories
                for (const subdir of SUBDIRS) {
                    const dirPath = path.join(baseDir, subdir);
                    if (!fs.existsSync(dirPath)) {
                        console.log(colors.yellow(`⚠️ Subdirectory not found: ${dirPath}`));
                        continue;
                    }
                    
                    const readSubDir = (dirPath: string) => {
                        const files = fs.readdirSync(dirPath);
                        for (const file of files) {
                            const fullPath = path.join(dirPath, file);
                            const stats = fs.statSync(fullPath);
                            
                            if (stats.isDirectory()) {
                                readSubDir(fullPath);
                            } else {
                                processFile(fullPath, file, type, subdir);
                            }
                        }
                    };
                    
                    readSubDir(dirPath);
                }
            }
        };

        // Crawl all directories
        await crawlDir(CHARARTS_PATH, 'base');
        await crawlDir(SKINPACK_PATH, 'skin');
        await crawlDir(DYNILLUST_PATH, 'dynIllust');

        // Print stats
        console.log(colors.cyan(`📊 File analysis: ${matchedFiles}/${totalFiles} files matched to operators (${Math.round((matchedFiles / totalFiles) * 100)}%)`));
        console.log(colors.cyan(`🔍 Found ${operatorMap.size} unique operators`));

        // Count operators with assets
        let operatorsWithBase = 0;
        let operatorsWithSkins = 0;
        let operatorsWithDynIllust = 0;
        let totalSkins = 0;
        let totalDynIllusts = 0;

        for (const [opId, data] of fileTracker.entries()) {
            // Check if operator has base assets
            if (Object.keys(data.base).length > 0) {
                operatorsWithBase++;
            } else {
                // If no base assets, clear the base directory
                const operator = operatorMap.get(opId);
                if (operator) {
                    const baseDir = operator.children!.find(c => c.name === "base");
                    if (baseDir) {
                        baseDir.children = [];
                    }
                }
            }

            // Check skins
            const skinCount = data.skin.size;
            if (skinCount > 0) {
                operatorsWithSkins++;
                totalSkins += skinCount;
            } else {
                // If no skins, clear the skin directory
                const operator = operatorMap.get(opId);
                if (operator) {
                    const skinDir = operator.children!.find(c => c.name === "skin");
                    if (skinDir) {
                        skinDir.children = [];
                    }
                }
            }

            // Check dynamic illustrations
            const dynIllustCount = data.dynIllust.size;
            if (dynIllustCount > 0) {
                operatorsWithDynIllust++;
                totalDynIllusts += dynIllustCount;
            } else {
                // If no dynamic illustrations, clear the dynIllust directory
                const operator = operatorMap.get(opId);
                if (operator) {
                    const dynIllustDir = operator.children!.find(c => c.name === "dynIllust");
                    if (dynIllustDir) {
                        dynIllustDir.children = [];
                    }
                }
            }
        }

        console.log(colors.cyan(`👤 Base sprites: ${operatorsWithBase}/${operatorMap.size} operators have base sprites`));
        console.log(colors.cyan(`🎭 Skin sprites: ${operatorsWithSkins}/${operatorMap.size} operators with skins (${totalSkins} total skins)`));
        console.log(colors.cyan(`✨ Dynamic illustrations: ${operatorsWithDynIllust}/${operatorMap.size} operators with dynamic illustrations (${totalDynIllusts} total)`));

        // Log top 10 operators by asset count
        const operatorCounts = new Map<string, number>();
        for (const [opId, operator] of operatorMap.entries()) {
            let count = 0;
            operator.children?.forEach((folder) => {
                count += folder.children?.length || 0;
            });
            operatorCounts.set(opId, count);
        }

        const topOperators = [...operatorCounts.entries()].sort((a, b) => b[1] - a[1]).slice(0, 10);

        if (topOperators.length > 0) {
            console.log(colors.cyan("📋 Top operators by asset count:"));
            topOperators.forEach(([opId, count]) => {
                console.log(colors.gray(`   ${opId}: ${count} files`));
            });
        }

        // Convert map to array
        const result = Array.from(operatorMap.values());

        // Save the data
        saveChibiData(result);

        return result;
    } catch (error) {
        console.error(colors.red(`❌ Error crawling local chibi assets: ${error}`));
        return [];
    }
};

// For backwards compatibility
export const crawlDirectory = crawlLocalChibis;
