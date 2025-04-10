// Credit to https://github.com/ChaomengOrion/ArkAssetsTool
// TypeScript conversion of the Python Arknights asset downloader, adapted for Bun

import * as crypto from "crypto";
import * as path from "path";
import { existsSync, mkdirSync, readdirSync, statSync, unlinkSync, rmdirSync } from "fs";
import * as readline from "readline";
import { Buffer } from "buffer";
import { Logger } from "./impl/logger";
import { MultiProgressBar } from "./impl/multi-progress-bar";
import { ProgressBar } from "./impl/progress-bar";
import { UnityAssetHandler } from "./impl/unity-asset-handler";
import { type AssetInfo, AssetServers, type HotUpdateList, LogLevel } from "../../../../../types/impl/lib/impl/local/impl/assets";

class ArkAssets {
    private static readonly CHAT_MASK = Buffer.from("554954704169383270484157776e7a7148524d4377506f6e4a4c49423357436c", "hex").toString();
    private server: AssetServers;
    private assetVersion: string = "";
    private clientVersion: string = "";
    private hotUpdateList: HotUpdateList = { other: { totalSize: 0, files: {} } };
    private totalSize: number = 0;
    private abSize: number = 0;

    constructor(server: AssetServers = AssetServers.OFFICIAL) {
        this.server = server;
    }

    static textAssetDecrypt(stream: Buffer, hasRsa: boolean = true): Buffer {
        const aesKey = Buffer.from(ArkAssets.CHAT_MASK.slice(0, 16));
        const aesIv = Buffer.alloc(16);
        const data = hasRsa ? stream.slice(128) : stream;
        const buf = data.slice(0, 16);
        const mask = Buffer.from(ArkAssets.CHAT_MASK.slice(16));

        for (let i = 0; i < buf.length; i++) {
            aesIv[i] = buf[i] ^ mask[i];
        }

        const decipher = crypto.createDecipheriv("aes-128-cbc", aesKey, aesIv);
        const decrypted = Buffer.concat([decipher.update(data.slice(16)), decipher.final()]);

        // Remove PKCS#7 padding
        const padLength = decrypted[decrypted.length - 1];
        return decrypted.slice(0, decrypted.length - padLength);
    }

    static async getVersion(server: AssetServers = AssetServers.OFFICIAL): Promise<[string, string]> {
        const response = await fetch(`https://ak-conf.hypergryph.com/config/prod/${server === AssetServers.OFFICIAL ? "official" : "b"}/Android/version`);
        const data = (await response.json()) as { resVersion: string; clientVersion: string };
        return [data.resVersion, data.clientVersion];
    }

    async initialize(): Promise<void> {
        [this.assetVersion, this.clientVersion] = await ArkAssets.getVersion(this.server);
        console.log(`\x1b[1;32mGame Versions: ${this.clientVersion} Material Versions: ${this.assetVersion}\x1b[0m`);

        const [hotUpdateList, totalSize, abSize] = await this.getHotUpdateList();
        this.hotUpdateList = hotUpdateList;
        this.totalSize = totalSize;
        this.abSize = abSize;

        console.log(`\x1b[1;32mTotal Resource Size: ${this.formatSize(this.totalSize)} Unzipped Size: ${this.formatSize(this.abSize)}\x1b[0m`);
    }

    // Public method to get the hot update list keys
    getHotUpdateListKeys(): string[] {
        return Object.keys(this.hotUpdateList);
    }

    private async getHotUpdateList(): Promise<[HotUpdateList, number, number]> {
        const response = await fetch(`https://ak.hycdn.cn/assetbundle/${this.server === AssetServers.OFFICIAL ? "official" : "bilibili"}/Android/assets/${this.assetVersion}/hot_update_list.json`);
        const js = (await response.json()) as {
            packInfos: Array<{ name: string }>;
            abInfos: Array<{
                name: string;
                totalSize: number;
                abSize: number;
                md5: string;
                pid?: string;
            }>;
        };

        const out: HotUpdateList = { other: { totalSize: 0, files: {} } };
        let totalSize = 0;
        let abSize = 0;

        for (const item of js.packInfos) {
            // Replace ALL underscores with slashes (Python uses replace which replaces all occurrences)
            const k = item.name.replaceAll("_", "/");
            out[k] = { totalSize: 0, files: {} };
            Logger.debug("ArkAssets", `Registered package: ${k}`);
        }

        const addOther = (item: any) => {
            const size = item.totalSize;
            out.other.totalSize += size;
            out.other.files[item.name] = {
                totalSize: size,
                abSize: item.abSize,
                md5: item.md5,
            };
        };

        for (const item of js.abInfos) {
            const size = item.totalSize;
            totalSize += size;
            const itemAbSize = item.abSize;
            abSize += itemAbSize;

            if ("pid" in item) {
                // Replace ALL underscores with slashes
                const pid = item.pid!.replaceAll("_", "/");
                if (pid in out) {
                    out[pid].totalSize += size;
                    out[pid].files[item.name] = {
                        totalSize: size,
                        abSize: itemAbSize,
                        md5: item.md5,
                    };
                } else {
                    addOther(item);
                }
            } else {
                addOther(item);
            }
        }

        const totalPackages = Object.keys(out).length;
        const totalFiles = Object.values(out).reduce((count, pkg) => count + Object.keys(pkg.files).length, 0);
        Logger.info("ArkAssets", `Found ${totalPackages} packages with ${totalFiles} total files`);

        return [out, totalSize, abSize];
    }

    async download(savedir: string): Promise<void> {
        const options: [string, string, string[]][] = [];

        // Sort packages by name for consistent display
        const packageNames = Object.keys(this.hotUpdateList).sort();
        Logger.info("ArkAssets", `Found ${packageNames.length} packages available for download`);

        // Display options for each package
        for (const item of packageNames) {
            const size = this.hotUpdateList[item].totalSize;
            const per = size / this.totalSize;
            const barLength = Math.floor(per * (process.stdout.columns - 46));
            const bar = "█".repeat(barLength);

            // Format similar to Python version
            const displayText = `${item.padEnd(15)} Size: ${this.formatSize(size).padEnd(7)} ${(per * 100).toFixed(2)}% ${bar}`;
            options.push([item, displayText, ["1", "34"]]);

            console.log(`\x1b[1;34m${displayText}\x1b[0m`);
        }

        console.log("\x1b[1;36mDownload Options [All Download=A, Select Download=C, Cancel=Other]: \x1b[0m");

        // Setup readline
        const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout,
        });

        rl.question("", async (inp) => {
            if (inp === "A" || inp === "a") {
                console.log("\x1b[1;36mStart All Download\x1b[0m");
                await this.downloadFromList(packageNames, savedir);
                rl.close();
            } else if (inp === "C" || inp === "c") {
                console.log("\x1b[1;36mSelect packages to download\x1b[0m");
                console.log("\x1b[1;36mEnter package numbers separated by commas (e.g., 1,3,5) or package names:\x1b[0m");

                // Display numbered options for easier selection
                packageNames.forEach((name, index) => {
                    console.log(`\x1b[1;33m${index + 1}. ${name}\x1b[0m`);
                });

                rl.question("Selection: ", async (selection) => {
                    const selectedKeys: string[] = [];

                    // Try to parse as numbers first
                    const numberSelections = selection
                        .split(",")
                        .map((s) => s.trim())
                        .filter((s) => s.length > 0);

                    for (const sel of numberSelections) {
                        // If it's a number, use it as an index
                        const numMatch = sel.match(/^\d+$/);
                        if (numMatch) {
                            const index = parseInt(sel, 10) - 1;
                            if (index >= 0 && index < packageNames.length) {
                                selectedKeys.push(packageNames[index]);
                            }
                        }
                        // Otherwise, check if it's a package name
                        else if (packageNames.includes(sel)) {
                            selectedKeys.push(sel);
                        }
                    }

                    // Deduplicate
                    const uniqueKeys = [...new Set(selectedKeys)];

                    if (uniqueKeys.length === 0) {
                        console.log("\x1b[1;33mNo valid packages selected, download canceled\x1b[0m");
                    } else {
                        console.log(`\x1b[1;36mSelected packages: ${uniqueKeys.join(", ")}\x1b[0m`);
                        await this.downloadFromList(uniqueKeys, savedir);
                    }
                    rl.close();
                });
            } else {
                console.log("\x1b[1;33mDownload Canceled\x1b[0m");
                rl.close();
            }
        });
    }

    async downloadFromList(keys: string[], savedir: string, threadingCount: number = 4): Promise<void> {
        // Create progress manager for coordinated output
        const progressManager = new MultiProgressBar();
        Logger.setProgressManager(progressManager);

        // Create progress bars
        const totalBar = progressManager.createBar(0, "Total Progress");
        const unzipBar = progressManager.createBar(0, "Unzip Progress");
        const unpackBar = progressManager.createBar(0, "Unpack Progress");

        // Filter keys to only include those that exist and log info about all available packages
        const availablePackages = Object.keys(this.hotUpdateList);
        Logger.info("ArkAssets", `Available packages (${availablePackages.length}): ${availablePackages.join(", ")}`);

        const validKeys = keys.filter((key) => this.hotUpdateList[key]);
        if (validKeys.length === 0) {
            Logger.error("ArkAssets", "No valid packages found in the provided keys");
            progressManager.stop();
            return;
        }

        // Calculate the total number of files across ALL packages for reference
        const totalFilesInGame = Object.values(this.hotUpdateList).reduce((count, pkg) => count + Object.keys(pkg.files).length, 0);

        // Calculate the total number of files in the selected packages
        const totalFilesInSelection = validKeys.reduce((count, key) => count + Object.keys(this.hotUpdateList[key].files).length, 0);

        Logger.info("ArkAssets", `Total files in game: ${totalFilesInGame}`);
        Logger.info("ArkAssets", `Selected packages (${validKeys.length}/${availablePackages.length}): ${validKeys.join(", ")}`);
        Logger.info("ArkAssets", `Files in selected packages: ${totalFilesInSelection}/${totalFilesInGame} (${((totalFilesInSelection / totalFilesInGame) * 100).toFixed(1)}% of total)`);

        // Create directory if it doesn't exist
        if (!existsSync(savedir)) {
            mkdirSync(savedir, { recursive: true });
        }

        // Read persistent resource list
        const perPath = path.join(savedir, "persistent_res_list.json");
        let per: Record<string, string> = {};

        if (!existsSync(perPath)) {
            await Bun.write(perPath, "{}");
        } else {
            try {
                const perContent = await Bun.file(perPath).text();
                per = JSON.parse(perContent);
                const downloadedCount = Object.keys(per).length;
                Logger.info("ArkAssets", `Found record of ${downloadedCount} previously downloaded files (${((downloadedCount / totalFilesInGame) * 100).toFixed(1)}% of total game files)`);
            } catch {
                Logger.warn("ArkAssets", "Error reading persistent_res_list.json, creating new one");
                await Bun.write(perPath, "{}");
            }
        }

        // Count total files and collect files to download
        const files: Record<string, AssetInfo> = {};
        let totalFiles = 0;
        let upToDateFiles = 0;
        let updatedFiles = 0;

        for (const key of validKeys) {
            const filesInPackage = this.hotUpdateList[key].files;
            const packageFileCount = Object.keys(filesInPackage).length;
            Logger.info("ArkAssets", `Package ${key} has ${packageFileCount} files`);

            for (const file in filesInPackage) {
                totalFiles++;
                // Check if we already have the latest version
                if (file in per && per[file] === filesInPackage[file].md5) {
                    upToDateFiles++;
                    Logger.debug("ArkAssets", `[${file}] Already has latest version`);
                } else {
                    if (file in per) {
                        Logger.info("ArkAssets", `[${file}] Will be updated`);
                        const filePath = path.join(savedir, file);
                        if (existsSync(filePath)) {
                            unlinkSync(filePath);
                        }
                        const unpackDir = path.join(path.dirname(filePath), path.basename(filePath, path.extname(filePath)) + "_unpacked");
                        if (existsSync(unpackDir)) {
                            this.removeDirectory(unpackDir);
                        }
                    } else {
                        Logger.info("ArkAssets", `[${file}] New file to download`);
                    }

                    // Add to download list
                    files[file] = filesInPackage[file];
                    updatedFiles++;
                }
            }
        }

        // Update progress bar totals
        totalBar.total = updatedFiles || 1; // Prevent division by zero
        unzipBar.total = updatedFiles || 1;
        unpackBar.total = updatedFiles || 1;

        if (updatedFiles === 0) {
            if (upToDateFiles === totalFilesInGame) {
                Logger.info("ArkAssets", "All game files are up to date! Nothing to download.");
            } else {
                Logger.info("ArkAssets", `All selected packages are up to date! (${upToDateFiles}/${totalFilesInGame} total game files downloaded)`);
                Logger.info("ArkAssets", `To download remaining files, select other packages when prompted.`);
            }
            progressManager.stop();
            return;
        }

        Logger.info("ArkAssets", `Found ${totalFiles} files in selected packages:`);
        Logger.info("ArkAssets", `- ${upToDateFiles} files already up to date`);
        Logger.info("ArkAssets", `- ${updatedFiles} files to download/update`);

        // Setup download threads
        const startTime = Date.now();
        const downloadedSize = { value: 0 };

        // Process files in batches per thread
        const batches: string[][] = [];

        for (let i = 0; i < threadingCount; i++) {
            batches.push([]);
        }

        // Distribute files to batches
        const fileNames = Object.keys(files);
        for (let i = 0; i < fileNames.length; i++) {
            batches[i % threadingCount].push(fileNames[i]);
        }

        // Process each batch in a separate thread
        const promises = batches.map((batch, i) => this.processBatch(batch, files, savedir, per, perPath, i, startTime, downloadedSize, totalBar, unzipBar, unpackBar));

        await Promise.all(promises);

        // Count remaining files that weren't part of this download
        const totalDownloadedFiles = Object.keys(per).length;
        const remainingFiles = totalFilesInGame - totalDownloadedFiles;

        Logger.info("ArkAssets", `Download complete! Downloaded ${this.formatSize(downloadedSize.value)} in ${((Date.now() - startTime) / 1000).toFixed(1)}s`);
        Logger.info("ArkAssets", `Progress: ${totalDownloadedFiles}/${totalFilesInGame} game files downloaded (${((totalDownloadedFiles / totalFilesInGame) * 100).toFixed(1)}% complete)`);

        if (remainingFiles > 0) {
            Logger.info("ArkAssets", `${remainingFiles} files remaining. Run the downloader again and select additional packages to complete the download.`);
        } else {
            Logger.info("ArkAssets", `All game files have been downloaded! The download is complete.`);
        }

        progressManager.stop();
    }

    private async processBatch(
        batch: string[],
        files: Record<string, AssetInfo>,
        savedir: string,
        per: Record<string, string>,
        perPath: string,
        threadNum: number,
        startTime: number,
        downloadedSize: { value: number },
        totalBar: ProgressBar,
        unzipBar: ProgressBar,
        unpackBar: ProgressBar,
    ): Promise<void> {
        for (const file of batch) {
            try {
                totalBar.update(totalBar.current, `Thread ${threadNum} [${file}]`);

                // Download the asset
                const stream = await this.downloadAsset(file, threadNum);
                downloadedSize.value += stream.length;

                // Update progress
                totalBar.increment(1);

                // Save the asset to disk
                const filePath = path.join(savedir, file);
                const dirPath = path.dirname(filePath);
                if (!existsSync(dirPath)) {
                    mkdirSync(dirPath, { recursive: true });
                }

                // Save/unzip the asset
                unzipBar.update(unzipBar.current, `Unzipping [${file}]`);
                await this.unzipAsset(stream, file, files[file].md5, savedir, per, perPath);
                unzipBar.increment(1);

                // If it's an .ab file (Unity Asset Bundle), skip unpacking
                if (file.endsWith(".ab")) {
                    Logger.info("ArkAssets", `Skipping unpack for Unity Asset Bundle: ${file}`);
                    unpackBar.update(unpackBar.current, `Skipped unpack for [${file}]`);
                    unpackBar.increment(1);
                } else {
                    // Unpack the asset
                    unpackBar.update(unpackBar.current, `Unpacking [${file}]`);
                    await this.unpackAsset(filePath, unpackBar);
                    unpackBar.increment(1);
                }
            } catch (error) {
                Logger.error("ArkAssets", `Error processing ${file}: ${error}`);
                // Still increment progress bars to prevent hanging UI
                if (totalBar.current < totalBar.total) totalBar.increment(1);
                if (unzipBar.current < unzipBar.total) unzipBar.increment(1);
                if (unpackBar.current < unpackBar.total) unpackBar.increment(1);
            }
        }
    }

    private async downloadAsset(pathToAsset: string, threadNum: number = 0): Promise<Buffer> {
        const url = `https://ak.hycdn.cn/assetbundle/${this.server === AssetServers.OFFICIAL ? "official" : "bilibili"}/Android/assets/${this.assetVersion}/${pathToAsset
            .replace(/(?<=\.)((?!\.)[^/])*$/, "dat")
            .replaceAll("/", "_")
            .replace("#", "__")}`;

        console.log(`\x1b[33mThread ${threadNum}: Downloading ${pathToAsset}\x1b[0m`);

        const response = await fetch(url, {
            headers: {
                "User-Agent": "BestHTTP",
            },
        });

        if (!response.ok) {
            throw new Error(`Failed to download ${pathToAsset}: ${response.status} ${response.statusText}`);
        }

        return Buffer.from(await response.arrayBuffer());
    }

    private async unzipAsset(stream: Buffer, name: string, md5: string, savePath: string, per: Record<string, string>, perPath: string): Promise<void> {
        try {
            // Check if this is an AB file (Unity Asset Bundle)
            if (name.endsWith(".ab")) {
                // Asset bundles are not ZIP files, just save them directly
                Logger.info("ArkAssets", `Saving Unity Asset Bundle: ${name}`);
                const outputPath = path.join(savePath, name);
                await Bun.write(outputPath, stream);
            } else {
                // For other files, assume they might be ZIP files
                const outputPath = path.join(savePath, name);
                // Create parent directory if it doesn't exist
                const parentDir = path.dirname(outputPath);
                if (!existsSync(parentDir)) {
                    mkdirSync(parentDir, { recursive: true });
                }

                try {
                    // First try to detect if this is a ZIP file
                    if (stream.length > 4 && stream[0] === 0x50 && stream[1] === 0x4b && stream[2] === 0x03 && stream[3] === 0x04) {
                        Logger.info("ArkAssets", `Detected ZIP file: ${name}, extracting contents`);
                        // We would extract the ZIP here, but Bun doesn't have built-in ZIP support
                        // For now, we'll just save the file and note that a proper implementation would extract it
                        await Bun.write(outputPath, stream);
                        Logger.warn("ArkAssets", `ZIP extraction not implemented in this version, saving as-is: ${name}`);
                    } else {
                        // Not a ZIP file, save directly
                        Logger.info("ArkAssets", `Saving file: ${name}`);
                        await Bun.write(outputPath, stream);
                    }
                } catch (zipError) {
                    Logger.error("ArkAssets", `Error processing file ${name}: ${zipError}`);
                    // Save the raw file as fallback
                    await Bun.write(outputPath, stream);
                }
            }

            // Update persistent list
            per[name] = md5;
            await Bun.write(perPath, JSON.stringify(per, null, 2));
        } catch (error) {
            Logger.error("ArkAssets", `Error unzipping ${name}: ${error}`);
        }
    }

    private async unpackAsset(filePath: string, progressBar: ProgressBar): Promise<void> {
        try {
            // Skip processing for .ab files - these are Unity Asset Bundle files that should remain as files
            if (filePath.endsWith(".ab")) {
                Logger.info("ArkAssets", `Skipping unpack for Unity Asset Bundle: ${filePath}`);
                progressBar.increment();
                return;
            }

            Logger.info("ArkAssets", `Unpacking asset: ${filePath}`);

            // Create a buffer from the file
            const data = await Bun.file(filePath).arrayBuffer();
            const buffer = Buffer.from(data);

            // Create asset handler
            const assetHandler = new UnityAssetHandler(buffer);

            // Check if the asset contains extractable content before proceeding
            if (await assetHandler.hasExtractableContent()) {
                // Extract to a directory named after the file but with an _unpacked suffix
                const fileName = path.basename(filePath);
                const baseName = path.basename(filePath, path.extname(filePath));
                const outputDir = path.join(path.dirname(filePath), baseName + "_unpacked");

                Logger.info("ArkAssets", `Extracting to: ${outputDir}`);

                // Create the output directory if it doesn't exist
                if (!existsSync(outputDir)) {
                    mkdirSync(outputDir, { recursive: true });
                }

                // Extract assets, passing the original file name
                await assetHandler.extractAssets(path.join(outputDir, fileName));

                // Verify if extraction produced any files
                let fileCount = 0;
                try {
                    const files = readdirSync(outputDir);
                    fileCount = files.length;

                    // If no files were extracted, remove the empty directory
                    if (fileCount === 0) {
                        rmdirSync(outputDir);
                        Logger.info("ArkAssets", `No files were extracted from ${filePath}, removed empty directory`);
                    } else {
                        Logger.info("ArkAssets", `Asset unpacked successfully: ${filePath} -> ${outputDir} (${fileCount} files)`);
                    }
                } catch (err) {
                    Logger.warn("ArkAssets", `Error verifying extraction results: ${err}`);
                }
            } else {
                Logger.info("ArkAssets", `No extractable content found in ${filePath}, skipping unpacking`);
            }
        } catch (error) {
            Logger.error("ArkAssets", `Failed to unpack asset ${filePath}:`, error);
        }
    }

    private removeDirectory(dirPath: string): void {
        if (existsSync(dirPath)) {
            readdirSync(dirPath).forEach((file) => {
                const curPath = path.join(dirPath, file);
                if (statSync(curPath).isDirectory()) {
                    this.removeDirectory(curPath);
                } else {
                    unlinkSync(curPath);
                }
            });
            rmdirSync(dirPath);
        }
    }

    private formatSize(bytes: number, size: number = 1024, digits: number = 2): string {
        const units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
        let count = 0;

        const scale = (n: number): number => {
            if (n > size && count < units.length - 1) {
                count++;
                return scale(n / size);
            }
            return n;
        };

        const scaled = scale(bytes);
        return `${scaled.toFixed(digits)}${units[count]}`;
    }
}

// Usage example
const main = async () => {
    // Initialize logging based on environment variables or defaults
    const logLevelString = process.env.ARKNIGHTS_LOG_LEVEL?.toUpperCase() || "INFO";
    const logLevelMap: Record<string, LogLevel> = {
        DEBUG: LogLevel.DEBUG,
        INFO: LogLevel.INFO,
        WARN: LogLevel.WARN,
        ERROR: LogLevel.ERROR,
    };

    Logger.setLogLevel(logLevelMap[logLevelString] || LogLevel.INFO);

    // Initialize file logging
    Logger.initializeFileLogging();

    Logger.info("ArkAssets", `Starting with log level: ${logLevelString}`);
    Logger.info("ArkAssets", `This session will be logged to the logs directory`);

    const arkAssets = new ArkAssets();
    await arkAssets.initialize();
    await arkAssets.download(process.argv[2] || path.join(process.cwd(), "ArkAssets"));
};

if (import.meta.main) {
    main().catch(console.error);
}

export default ArkAssets;

// Include at the bottom of the file:

// Setup command-line entry point
async function runDownloader() {
    try {
        // Parse command line arguments
        const args = process.argv.slice(2);
        const downloadPath = args[0] || path.join(process.cwd(), "ArkAssets");

        // Print script information
        console.log("\x1b[1;36m==========================================\x1b[0m");
        console.log("\x1b[1;36mArknights Asset Downloader (TypeScript)\x1b[0m");
        console.log("\x1b[1;36m==========================================\x1b[0m");

        // Initialize logging
        const logLevelString = process.env.ARKNIGHTS_LOG_LEVEL?.toUpperCase() || "INFO";
        const logLevelMap: Record<string, LogLevel> = {
            DEBUG: LogLevel.DEBUG,
            INFO: LogLevel.INFO,
            WARN: LogLevel.WARN,
            ERROR: LogLevel.ERROR,
        };

        Logger.setLogLevel(logLevelMap[logLevelString] || LogLevel.INFO);
        Logger.initializeFileLogging();

        Logger.info("ArkAssets", `Starting Arknights asset downloader with log level: ${logLevelString}`);
        Logger.info("ArkAssets", `Download path: ${downloadPath}`);

        // Initialize the downloader
        const arkAssets = new ArkAssets();
        await arkAssets.initialize();

        // Download assets to the specified path
        await arkAssets.download(downloadPath);
    } catch (error) {
        console.error(`\x1b[31mFatal error: ${error}\x1b[0m`);
        process.exit(1);
    }
}

// Export the downloder function
export { runDownloader };

// If this script is run directly, execute the downloader
if (require.main === module) {
    runDownloader();
}
