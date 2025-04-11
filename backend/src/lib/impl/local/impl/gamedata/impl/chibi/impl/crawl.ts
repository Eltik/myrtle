import type { RepoItem } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";
import colors from "colors";
import fs from "fs";
import path from "path";

// Local paths for chibi assets - using paths relative to backend directory
export const CHARARTS_PATH = path.resolve(process.cwd(), "unpacked", "chararts");
export const SKINPACK_PATH = path.resolve(process.cwd(), "unpacked", "skinpack");

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

// Main crawl function that crawls both chararts and skinpack directories
export const crawlLocalChibis = async (): Promise<RepoItem[]> => {
    try {
        console.log(colors.cyan(`ℹ️ Crawling local chibi assets...`));

        const operatorMap = new Map<string, RepoItem>();
        let totalFiles = 0;
        let matchedFiles = 0;

        // Function to crawl a directory and process files
        const crawlDir = async (dirPath: string, isChararts: boolean) => {
            if (!fs.existsSync(dirPath)) {
                console.log(colors.yellow(`⚠️ Directory not found: ${dirPath}`));
                return;
            }

            const processFile = (filePath: string, fileName: string) => {
                // Skip monobehaviour files
                if (fileName.toLowerCase().startsWith("monobehaviour_")) {
                    return;
                }

                totalFiles++;

                // Debug output
                if (totalFiles < 50 || totalFiles % 1000 === 0) {
                    console.log(colors.gray(`🔍 Processing file: ${fileName}`));

                    // Test regex match
                    const match = fileName.match(/(build_char_|char_)(\d+_[a-zA-Z0-9]+)/i);
                    if (match) {
                        console.log(colors.green(`  ✅ Regex match: ${match[0]} -> ${match[2]}`));
                    } else {
                        console.log(colors.red(`  ❌ No regex match`));
                    }
                }

                // Match operator ID
                const match = fileName.match(/(build_char_|char_)(\d+_[a-zA-Z0-9]+)/i);
                if (match) {
                    // Extract the operator ID
                    const prefix = "char_";
                    const id = match[2].toLowerCase();
                    const opId = `${prefix}${id}`;

                    matchedFiles++;
                    console.log(colors.green(`✓ Matched file ${fileName} to operator ID: ${opId}`));

                    // Create operator if it doesn't exist
                    if (!operatorMap.has(opId)) {
                        operatorMap.set(opId, {
                            name: opId,
                            path: opId,
                            contentType: "dir",
                            children: [
                                // Create base and skin directories
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
                            ],
                        });
                    }

                    // Get the operator
                    const operator = operatorMap.get(opId)!;

                    // Get the correct folder (base or skin)
                    const folderName = isChararts ? "base" : "skin";
                    const folder = operator.children!.find((c) => c.name === folderName)!;

                    // Add the file to the folder
                    folder.children!.push({
                        name: fileName,
                        path: `${opId}/${folderName}/${fileName}`,
                        contentType: "file",
                    });
                }
            };

            // Read all files recursively
            const readDirRecursive = (dirPath: string) => {
                const files = fs.readdirSync(dirPath);
                for (const file of files) {
                    const fullPath = path.join(dirPath, file);
                    const stats = fs.statSync(fullPath);
                    if (stats.isDirectory()) {
                        readDirRecursive(fullPath);
                    } else {
                        processFile(fullPath, file);
                    }
                }
            };

            // Start crawling
            readDirRecursive(dirPath);
        };

        // Crawl both directories
        await crawlDir(CHARARTS_PATH, true);
        await crawlDir(SKINPACK_PATH, false);

        // Print stats
        console.log(colors.cyan(`📊 File analysis: ${matchedFiles}/${totalFiles} files matched to operators (${Math.round((matchedFiles / totalFiles) * 100)}%)`));
        console.log(colors.cyan(`🔍 Found ${operatorMap.size} unique operators`));

        // Log top 10 operators by file count
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
            console.log(colors.cyan("📋 Top operators by file count:"));
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
