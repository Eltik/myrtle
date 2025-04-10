// Credit to https://github.com/ChaomengOrion/ArkAssetsTool
// TypeScript conversion of the Python Arknights asset downloader, adapted for Bun

import * as crypto from "crypto";
import * as path from "path";
import { existsSync, mkdirSync, readdirSync, statSync, unlinkSync, rmdirSync, appendFileSync } from "fs";
import * as readline from "readline";
import { Buffer } from "buffer";
import lz4 from "lz4js";

// Logger utility for better debugging
enum LogLevel {
    DEBUG = 0,
    INFO = 1,
    WARN = 2,
    ERROR = 3,
}

// Global progress bar manager that can be accessed by the Logger
let globalProgressManager: MultiProgressBar | null = null;

class Logger {
    private static logLevel: LogLevel = LogLevel.INFO;
    private static readonly colors = {
        debug: "\x1b[90m", // Gray
        info: "\x1b[36m", // Cyan
        warn: "\x1b[33m", // Yellow
        error: "\x1b[31m", // Red
        reset: "\x1b[0m", // Reset
    };
    private static logFile: string | null = null;
    private static logDir: string = "./logs";

    static setLogLevel(level: LogLevel): void {
        Logger.logLevel = level;
    }

    static setProgressManager(manager: MultiProgressBar): void {
        globalProgressManager = manager;
    }

    static initializeFileLogging(): void {
        try {
            // Ensure logs directory exists
            if (!existsSync(Logger.logDir)) {
                mkdirSync(Logger.logDir, { recursive: true });
            }

            // Create log file with timestamp
            const now = new Date();
            const timestamp = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}_${String(now.getHours()).padStart(2, "0")}-${String(now.getMinutes()).padStart(2, "0")}-${String(now.getSeconds()).padStart(2, "0")}`;
            Logger.logFile = `${Logger.logDir}/arkassets_${timestamp}.log`;

            // Log initialization
            Logger.info("Logger", `File logging initialized to ${Logger.logFile}`);
        } catch (error) {
            console.error(`Failed to initialize file logging: ${error}`);
        }
    }

    static debug(component: string, message: string, ...args: any[]): void {
        if (Logger.logLevel <= LogLevel.DEBUG) {
            const logMsg = `${Logger.colors.debug}[DEBUG][${component}]${Logger.colors.reset} ${message}`;
            Logger.log(logMsg, args);
        }
    }

    static info(component: string, message: string, ...args: any[]): void {
        if (Logger.logLevel <= LogLevel.INFO) {
            const logMsg = `${Logger.colors.info}[INFO][${component}]${Logger.colors.reset} ${message}`;
            Logger.log(logMsg, args);
        }
    }

    static warn(component: string, message: string, ...args: any[]): void {
        if (Logger.logLevel <= LogLevel.WARN) {
            const logMsg = `${Logger.colors.warn}[WARN][${component}]${Logger.colors.reset} ${message}`;
            Logger.log(logMsg, args);
        }
    }

    static error(component: string, message: string, ...args: any[]): void {
        if (Logger.logLevel <= LogLevel.ERROR) {
            const logMsg = `${Logger.colors.error}[ERROR][${component}]${Logger.colors.reset} ${message}`;
            Logger.log(logMsg, args);
        }
    }

    private static log(message: string, args: any[]): void {
        const fullMessage = args.length > 0 ? `${message} ${args.join(" ")}` : message;
        const timestamp = new Date().toISOString();
        const logEntry = `[${timestamp}] ${fullMessage}`;

        // Console output with colors
        if (globalProgressManager) {
            globalProgressManager.log(fullMessage);
        } else {
            console.log(fullMessage);
        }

        // File output without colors (strip ANSI codes)
        if (Logger.logFile) {
            try {
                const plainText = logEntry.replace(/\x1b\[[0-9;]*m/g, "");
                const logMessage = plainText + "\n";

                // Use the imported fs module
                appendFileSync(Logger.logFile, logMessage);
            } catch (error) {
                console.error(`Failed to write to log file: ${error}`);
            }
        }
    }
}

// Simple progress bar class
class ProgressBar {
    private width: number;
    public total: number;
    public current: number = 0;
    private description: string;
    private lastRender: string = "";
    private fixed: boolean;

    constructor(total: number, description: string = "", width: number = 40, fixed: boolean = false) {
        this.total = total;
        this.description = description;
        this.width = width;
        this.fixed = fixed;
    }

    update(current: number, description?: string): void {
        this.current = current;
        if (description) {
            this.description = description;
        }
        this.render();
    }

    increment(value: number = 1, description?: string): void {
        this.update(this.current + value, description);
    }

    render(): void {
        const percent = Math.min(Math.floor((this.current / this.total) * 100), 100);
        const filledWidth = Math.floor((this.current / this.total) * this.width);
        const bar = "█".repeat(filledWidth) + "-".repeat(this.width - filledWidth);

        const renderText = `${this.description.padEnd(30)} |${bar}| ${percent}% (${this.current}/${this.total})`;

        // Only redraw if the output has changed
        if (renderText !== this.lastRender) {
            if (this.fixed) {
                // Just print a new line with the updated progress
                console.log(renderText);
            } else {
                // Simply log the updated progress
                console.log(renderText);
            }
            this.lastRender = renderText;
        }
    }
}

// Multi-progress bar manager
class MultiProgressBar {
    private bars: ProgressBar[] = [];
    private logBuffer: string[] = [];
    private maxLogs: number = 20;
    private initialized: boolean = false;

    createBar(total: number, description: string = ""): ProgressBar {
        const bar = new ProgressBar(total, description, 40, false);
        this.bars.push(bar);

        if (!this.initialized) {
            // Simple initialization - no need to create blank space
            this.initialized = true;
        }

        return bar;
    }

    log(message: string): void {
        // Add to log buffer
        const timestamp = new Date().toISOString();
        const logEntry = `[${timestamp}] ${message}`;
        this.logBuffer.push(logEntry);

        // Trim log buffer if needed
        if (this.logBuffer.length > this.maxLogs) {
            this.logBuffer = this.logBuffer.slice(this.logBuffer.length - this.maxLogs);
        }

        // Simply log the message
        console.log(logEntry);
    }

    stop(): void {
        // No need to move cursor or clear screen
        this.initialized = false;
        console.log("--- Progress Complete ---");
    }
}

// Unity asset handling utility
// This is a detailed implementation to parse Unity asset bundles
class UnityAssetHandler {
    private buffer: Buffer;
    private header: UnityBundleHeader | null = null;
    private assets: UnityAsset[] = [];

    constructor(buffer: Buffer) {
        this.buffer = buffer;
        this.parseHeader();
    }

    private parseHeader(): void {
        // Unity bundle signature check - typically starts with "UnityFS"
        const signature = this.buffer.slice(0, 8).toString("utf8");

        if (signature === "UnityFS") {
            Logger.debug("UnityAssetHandler", "Detected UnityFS format signature");
            this.parseUnityFSFormat();
        } else if (signature.startsWith("Unity")) {
            Logger.debug("UnityAssetHandler", "Detected legacy Unity format signature");
            this.parseLegacyFormat();
        } else {
            Logger.warn("UnityAssetHandler", `Unknown Unity asset format signature: "${signature.substring(0, Math.min(signature.length, 20))}"`);
            this.attemptBasicParsing();
        }
    }

    private parseUnityFSFormat(): void {
        // Unity FS format (Unity 5+)
        // Format: UnityFS signature, format version, unity version, file size, compressed data size,
        // uncompressed data size, flags, data hash (optional)
        try {
            const view = new DataView(this.buffer.buffer);

            this.header = {
                signature: this.buffer.slice(0, 8).toString("utf8"),
                version: view.getUint32(8, true),
                unityVersion: this.buffer.slice(12, 28).toString("utf8").replace(/\0/g, ""),
                unityRevision: this.buffer.slice(28, 44).toString("utf8").replace(/\0/g, ""),
                size: view.getBigUint64(44, true),
                compressedBlocksInfoSize: view.getUint32(52, true),
                uncompressedBlocksInfoSize: view.getUint32(56, true),
                flags: view.getUint32(60, true),
            };

            // Log additional debug info for compression
            const compressionType = this.header.flags & 0x3f;
            if (compressionType > 0) {
                const types = ["None", "LZMA", "LZ4", "LZ4HC", "LZHAM"];
                Logger.debug("UnityAssetHandler", `Unity asset compression detected: ${types[compressionType] || "Unknown"} (${compressionType})`);
                Logger.debug("UnityAssetHandler", `Compression details: flags=${this.header.flags}, compressedSize=${this.header.compressedBlocksInfoSize}, uncompressedSize=${this.header.uncompressedBlocksInfoSize}`);
            }

            // Extract assets based on the header
            this.extractAssetsFromBundle();
        } catch (error) {
            console.error("Error parsing Unity FS format:", error);
        }
    }

    private parseLegacyFormat(): void {
        // Legacy Unity format (pre-Unity 5)
        console.warn("Legacy Unity format detected, limited support available");

        try {
            const view = new DataView(this.buffer.buffer);

            // Simplified legacy header - actual format is more complex
            this.header = {
                signature: this.buffer.slice(0, 8).toString("utf8"),
                version: view.getUint32(8, true),
                unityVersion: "Pre-Unity 5",
                unityRevision: "Unknown",
                size: BigInt(this.buffer.length),
                compressedBlocksInfoSize: 0,
                uncompressedBlocksInfoSize: 0,
                flags: 0,
            };

            // Attempt to extract assets
            this.attemptBasicParsing();
        } catch (error) {
            console.error("Error parsing legacy Unity format:", error);
        }
    }

    private extractAssetsFromBundle(): void {
        if (!this.header) return;

        try {
            // Skip header (64 bytes) and parse blocks info

            // Check if data is compressed
            const isCompressed = (this.header.flags & 0x3f) !== 0;

            if (isCompressed) {
                // Handle different compression types
                const compressionType = this.header.flags & 0x3f;

                switch (compressionType) {
                    case 1: // LZMA
                        console.warn("LZMA compression not implemented in this version");
                        this.attemptBasicParsing();
                        return;
                    case 2: // LZ4
                    case 3: // LZ4HC
                        // Start at byte 64 (after header)
                        const compressedData = this.buffer.slice(64);

                        try {
                            // Get compressed and uncompressed block size from header
                            const compressedSize = this.header.compressedBlocksInfoSize;
                            const uncompressedSize = this.header.uncompressedBlocksInfoSize;

                            // Log for debugging
                            Logger.info("UnityAssetHandler", `Decompressing LZ4 data: compressed size ${compressedSize}, uncompressed size ${uncompressedSize}`);

                            // Perform LZ4 decompression using lz4js
                            let decompressedData: Buffer;
                            try {
                                const result = lz4.decompress(compressedData.slice(0, compressedSize));
                                decompressedData = Buffer.from(result);

                                if (decompressedData.length !== uncompressedSize) {
                                    Logger.warn("UnityAssetHandler", `Decompression size mismatch: expected ${uncompressedSize}, got ${decompressedData.length}. Continuing anyway.`);
                                }
                            } catch (err) {
                                Logger.error("UnityAssetHandler", "LZ4 decompression failed:", err);
                                this.attemptBasicParsing();
                                return;
                            }

                            if (!decompressedData || decompressedData.length === 0) {
                                Logger.error("UnityAssetHandler", "LZ4 decompression failed, falling back to basic parsing");
                                this.attemptBasicParsing();
                                return;
                            }

                            Logger.info("UnityAssetHandler", `LZ4 decompression successful: decoded ${decompressedData.length} bytes`);

                            // Full implementation that handles blocks info and data blocks properly
                            try {
                                // The decompressed data is the blocks info
                                const blocksInfoData = decompressedData;

                                // Parse the blocks info
                                // Format typically:
                                // - 16 bytes hash (uncompressed blocks info hash)
                                // - Block data (blocks count, uncompressed/compressed sizes, flags)

                                // Skip the 16 bytes hash at the beginning of blocks info
                                const blocksInfoView = new DataView(blocksInfoData.buffer);
                                let offset = 16; // Skip hash

                                // Read the number of blocks
                                const blocksCount = blocksInfoView.getUint32(offset, true);
                                offset += 4;

                                Logger.debug("UnityAssetHandler", `Asset bundle has ${blocksCount} data blocks`);

                                // Create arrays to store block metadata
                                const uncompressedSizes: number[] = [];
                                const compressedSizes: number[] = [];
                                const flags: number[] = [];

                                // Read block metadata
                                for (let i = 0; i < blocksCount; i++) {
                                    uncompressedSizes.push(blocksInfoView.getUint32(offset, true));
                                    offset += 4;
                                }

                                for (let i = 0; i < blocksCount; i++) {
                                    compressedSizes.push(blocksInfoView.getUint32(offset, true));
                                    offset += 4;
                                }

                                for (let i = 0; i < blocksCount; i++) {
                                    flags.push(blocksInfoView.getUint16(offset, true));
                                    offset += 2;
                                }

                                // Calculate total uncompressed size to allocate buffer
                                const totalUncompressedSize = uncompressedSizes.reduce((sum, size) => sum + size, 0);
                                const dataResult = Buffer.alloc(totalUncompressedSize);

                                // The actual compressed data starts after the header (64 bytes) and the blocks info
                                let dataOffset = 64 + this.header.compressedBlocksInfoSize;
                                let outputOffset = 0;

                                // Process each data block
                                for (let i = 0; i < blocksCount; i++) {
                                    const uncompressedSize = uncompressedSizes[i];
                                    const compressedSize = compressedSizes[i];
                                    const blockFlag = flags[i];

                                    // Get the current data block
                                    const blockData = this.buffer.slice(dataOffset, dataOffset + compressedSize);

                                    // Check if the block is compressed (flag bit 0x3F != 0)
                                    const isCompressed = (blockFlag & 0x3f) !== 0;

                                    if (isCompressed) {
                                        // Handle different compression types based on flag
                                        const compressionType = blockFlag & 0x3f;

                                        if (compressionType === 2 || compressionType === 3) {
                                            // LZ4 decompression
                                            try {
                                                const decompressed = lz4.decompress(blockData);

                                                if (decompressed.length !== uncompressedSize) {
                                                    Logger.warn("UnityAssetHandler", `Block ${i} decompression size mismatch: expected ${uncompressedSize}, got ${decompressed.length}`);
                                                }

                                                // Copy the decompressed data to the result buffer
                                                Buffer.from(decompressed).copy(dataResult, outputOffset, 0, Math.min(decompressed.length, uncompressedSize));
                                            } catch (err) {
                                                Logger.warn("UnityAssetHandler", `Failed to decompress block ${i}:`, err);
                                                // Copy as-is (although this will likely result in corrupt data)
                                                blockData.copy(dataResult, outputOffset, 0, Math.min(blockData.length, uncompressedSize));
                                            }
                                        } else {
                                            Logger.warn("UnityAssetHandler", `Unsupported compression type in block ${i}: ${compressionType}`);
                                            // Copy as-is (although this will likely result in corrupt data)
                                            blockData.copy(dataResult, outputOffset, 0, Math.min(blockData.length, uncompressedSize));
                                        }
                                    } else {
                                        // Block is not compressed, just copy the data
                                        blockData.copy(dataResult, outputOffset, 0, uncompressedSize);
                                    }

                                    // Move to next block
                                    dataOffset += compressedSize;
                                    outputOffset += uncompressedSize;
                                }

                                Logger.info("UnityAssetHandler", `Successfully processed ${blocksCount} blocks, total size: ${this.formatSize(totalUncompressedSize)}`);

                                // Replace the buffer with the fully processed data for further processing
                                this.buffer = dataResult;

                                // Now identify assets in the decompressed data
                                this.identifyAssetTypes();
                                return;
                            } catch (error) {
                                Logger.error("UnityAssetHandler", `Error in full asset bundle processing: ${error}`);

                                // Fallback to the simplified approach
                                Logger.warn("UnityAssetHandler", "Falling back to simplified decompression approach");
                                this.buffer = decompressedData;
                                this.identifyAssetTypes();
                                return;
                            }
                        } catch (error) {
                            Logger.error("UnityAssetHandler", "Error during LZ4 decompression:", error);
                            this.attemptBasicParsing();
                            return;
                        }
                    default:
                        Logger.warn("UnityAssetHandler", "Unknown compression format:", compressionType);
                        this.attemptBasicParsing();
                        return;
                }
            } else {
                // Uncompressed data - we would process the blocks info here
                // but for this implementation we'll just use pattern detection
            }

            // Since we don't have full decompression libraries, we'll try to identify
            // common asset patterns in the data
            this.identifyAssetTypes();
        } catch (error) {
            console.error("Error extracting assets from bundle:", error);
            this.attemptBasicParsing();
        }
    }

    private identifyAssetTypes(): void {
        // Scan through the buffer to find common asset type signatures
        this.assets = [];

        // Look for texture data
        this.findTextureAssets();

        // Look for text data
        this.findTextAssets();

        // Look for audio data
        this.findAudioAssets();

        // Look for mesh data
        this.findMeshAssets();

        console.log(`Identified ${this.assets.length} assets in bundle`);
    }

    private findTextureAssets(): void {
        // Common texture formats:
        // PNG: 89 50 4E 47
        // JPEG: FF D8 FF
        // DXT: various DDS signatures

        // Simplistic approach: scan for PNG/JPEG signatures
        for (let i = 0; i < this.buffer.length - 16; i++) {
            if (
                (this.buffer[i] === 0x89 && this.buffer[i + 1] === 0x50 && this.buffer[i + 2] === 0x4e && this.buffer[i + 3] === 0x47) || // PNG
                (this.buffer[i] === 0xff && this.buffer[i + 1] === 0xd8 && this.buffer[i + 2] === 0xff) // JPEG
            ) {
                // Found a texture signature
                // In a real implementation, we would determine the size of the texture
                // and extract the full data
                const format = this.buffer[i] === 0x89 ? "PNG" : "JPEG";

                // Simple size estimation (basic approach)
                let size = 0;
                if (format === "PNG") {
                    // Look for IEND chunk
                    for (let j = i; j < this.buffer.length - 8; j++) {
                        if (this.buffer[j] === 0x49 && this.buffer[j + 1] === 0x45 && this.buffer[j + 2] === 0x4e && this.buffer[j + 3] === 0x44) {
                            size = j + 8 - i;
                            break;
                        }
                    }
                } else if (format === "JPEG") {
                    // Look for JPEG EOI marker (FF D9)
                    for (let j = i; j < this.buffer.length - 2; j++) {
                        if (this.buffer[j] === 0xff && this.buffer[j + 1] === 0xd9) {
                            size = j + 2 - i;
                            break;
                        }
                    }
                }

                if (size > 0) {
                    this.assets.push({
                        type: "Texture2D",
                        format: format,
                        offset: i,
                        size: size,
                        name: `texture_${this.assets.length}`,
                    });

                    // Skip ahead to avoid finding the same texture again
                    i += size;
                }
            }
        }
    }

    private findTextAssets(): void {
        // Text asset detection is challenging, but we can look for:
        // 1. JSON patterns: { and " characters
        // 2. XML/HTML patterns: < and > characters
        // 3. Plain text: look for runs of ASCII text

        for (let i = 0; i < this.buffer.length - 16; i++) {
            // Check for Unity TextAsset markers
            // Unity often prefixes TextAssets with specific markers
            const possibleTextRun = this.buffer.slice(i, i + 100).toString("utf8");

            if (possibleTextRun.includes('{"') || possibleTextRun.includes("<?xml")) {
                // Potential JSON or XML content
                let endOffset = i;
                const format = possibleTextRun.includes('{"') ? "JSON" : "XML";

                // Try to find end of text - this is a simplified approach
                if (format === "JSON") {
                    let braceCount = 0;
                    let inString = false;

                    for (let j = i; j < Math.min(i + 10000, this.buffer.length); j++) {
                        const char = String.fromCharCode(this.buffer[j]);

                        if (char === '"' && (j === i || String.fromCharCode(this.buffer[j - 1]) !== "\\")) {
                            inString = !inString;
                        }

                        if (!inString) {
                            if (char === "{") braceCount++;
                            if (char === "}") braceCount--;

                            if (braceCount === 0 && j > i + 10) {
                                endOffset = j + 1;
                                break;
                            }
                        }
                    }
                } else if (format === "XML") {
                    // Simple XML end detection
                    for (let j = i + 100; j < Math.min(i + 10000, this.buffer.length - 10); j++) {
                        if (
                            this.buffer
                                .slice(j, j + 10)
                                .toString("utf8")
                                .includes("</")
                        ) {
                            for (let k = j; k < Math.min(j + 1000, this.buffer.length - 1); k++) {
                                if (this.buffer[k] === 0x3e) {
                                    // '>'
                                    endOffset = k + 1;
                                    break;
                                }
                            }
                            break;
                        }
                    }
                }

                if (endOffset > i + 10) {
                    this.assets.push({
                        type: "TextAsset",
                        format: format,
                        offset: i,
                        size: endOffset - i,
                        name: `text_${this.assets.length}`,
                    });

                    i = endOffset;
                }
            }
        }
    }

    private findAudioAssets(): void {
        // Common audio signatures:
        // WAV: 52 49 46 46 (RIFF)
        // OGG: 4F 67 67 53 (OggS)
        // MP3: 49 44 33 or FF FB

        for (let i = 0; i < this.buffer.length - 16; i++) {
            let format = "";
            let size = 0;

            if (this.buffer[i] === 0x52 && this.buffer[i + 1] === 0x49 && this.buffer[i + 2] === 0x46 && this.buffer[i + 3] === 0x46) {
                // WAV file (RIFF header)
                format = "WAV";

                // WAV size is at bytes 4-7 after "RIFF" (little-endian)
                const wavSize = this.buffer.readUInt32LE(i + 4) + 8; // + 8 for header
                if (wavSize > 0 && wavSize < 100 * 1024 * 1024) {
                    // Sanity check: < 100MB
                    size = wavSize;
                }
            } else if (this.buffer[i] === 0x4f && this.buffer[i + 1] === 0x67 && this.buffer[i + 2] === 0x67 && this.buffer[i + 3] === 0x53) {
                // OGG file (OggS header)
                format = "OGG";

                // OGG files don't have a simple size field
                // Look for end of file by scanning for multiple "OggS" markers
                let lastOggS = i;
                for (let j = i + 4; j < this.buffer.length - 4; j++) {
                    if (this.buffer[j] === 0x4f && this.buffer[j + 1] === 0x67 && this.buffer[j + 2] === 0x67 && this.buffer[j + 3] === 0x53) {
                        lastOggS = j;
                    }
                }

                // Scan past the last OggS to find end of file
                for (let j = lastOggS; j < Math.min(lastOggS + 10000, this.buffer.length); j++) {
                    // Simple heuristic: look for non-OGG data
                    if (
                        j + 50 < this.buffer.length &&
                        this.buffer
                            .slice(j, j + 50)
                            .toString("hex")
                            .includes("00000000000000000000")
                    ) {
                        size = j - i;
                        break;
                    }
                }

                if (size === 0 && lastOggS > i) {
                    // Estimate size based on last OggS marker
                    size = lastOggS - i + 1000; // Add some padding
                }
            } else if (
                (this.buffer[i] === 0x49 && this.buffer[i + 1] === 0x44 && this.buffer[i + 2] === 0x33) || // ID3
                (this.buffer[i] === 0xff && this.buffer[i + 1] === 0xfb) // MP3 frame sync
            ) {
                // MP3 file
                format = "MP3";

                // MP3 size is harder to determine
                // Look for end by scanning for non-MP3 data
                for (let j = i + 1000; j < Math.min(i + 100000, this.buffer.length - 10); j++) {
                    // Simple approach: look for zero-filled sections or new file headers
                    if (
                        this.buffer.slice(j, j + 10).every((b) => b === 0) ||
                        (this.buffer[j] === 0x89 && this.buffer[j + 1] === 0x50) || // PNG
                        (this.buffer[j] === 0xff && this.buffer[j + 1] === 0xd8) || // JPEG
                        (this.buffer[j] === 0x52 && this.buffer[j + 1] === 0x49) || // RIFF
                        (this.buffer[j] === 0x4f && this.buffer[j + 1] === 0x67) || // OGG
                        (this.buffer[j] === 0x49 && this.buffer[j + 1] === 0x44 && j > i + 1000) // ID3
                    ) {
                        size = j - i;
                        break;
                    }
                }
            }

            if (format && size > 1000) {
                // Minimum size check to avoid false positives
                this.assets.push({
                    type: "AudioClip",
                    format: format,
                    offset: i,
                    size: size,
                    name: `audio_${this.assets.length}`,
                });

                i += size - 1;
            }
        }
    }

    private findMeshAssets(): void {
        // Unity mesh assets are harder to detect by raw signatures
        // In a real implementation, we would parse the asset structure
        // For this example, we'll use some simplified detection

        // Common mesh data might include sequences of float32 values
        // that represent vertices/normals/UVs
        // This is a highly simplified approach and may have false positives

        // Skip for now as it requires more detailed Unity format knowledge
        return;
    }

    private attemptBasicParsing(): void {
        // Fallback method if we can't parse the bundle format
        // Just try to identify some common asset types based on patterns
        this.identifyAssetTypes();
    }

    // Main method to extract assets into separate files
    async extractAssets(outputPath: string): Promise<void> {
        if (!this.header) {
            // Don't treat this as a fatal error, just try basic extraction
            Logger.warn("UnityAssetHandler", "No Unity header parsed, attempting basic extraction");
            return this.extractUsingBasicTypes(outputPath);
        }

        try {
            // Create output directory if it doesn't exist
            if (!existsSync(outputPath)) {
                mkdirSync(outputPath, { recursive: true });
            }

            Logger.info("UnityAssetHandler", `Extracting assets to ${outputPath}`);
            Logger.info("UnityAssetHandler", `Bundle format: ${this.header.signature}, version: ${this.header.version}, Unity: ${this.header.unityVersion}`);
            Logger.info("UnityAssetHandler", `Assets found: ${this.assets.length}`);

            // Group assets by type for logging
            const assetTypes: Record<string, number> = {};
            for (const asset of this.assets) {
                assetTypes[asset.type] = (assetTypes[asset.type] || 0) + 1;
            }

            // Log asset type distribution
            Logger.debug("UnityAssetHandler", "Asset types distribution:");
            for (const [type, count] of Object.entries(assetTypes)) {
                Logger.debug("UnityAssetHandler", `  - ${type}: ${count} assets`);
            }

            // If no assets were found via type identification, try basic type detection
            if (this.assets.length === 0) {
                Logger.warn("UnityAssetHandler", "No assets identified by type, attempting basic extraction");
                return this.extractUsingBasicTypes(outputPath);
            }

            // Extract assets by type
            for (const asset of this.assets) {
                try {
                    const assetOutputPath = path.join(outputPath, asset.name + this.getFileExtension(asset));

                    // Extract based on asset type
                    if (asset.type === "TextAsset") {
                        await this.extractTextAsset(assetOutputPath);
                    } else if (asset.type === "Texture2D" || asset.type === "Sprite") {
                        await this.extractImageAsset(assetOutputPath);
                    } else if (asset.type === "AudioClip") {
                        await this.extractAudioAsset(assetOutputPath);
                    } else if (asset.type === "Mesh") {
                        // Mesh extraction would be implemented here
                        Logger.warn("UnityAssetHandler", `Skipping unsupported asset type: ${asset.type}`);
                    } else {
                        Logger.warn("UnityAssetHandler", `Skipping unsupported asset type: ${asset.type}`);
                    }
                } catch (error) {
                    Logger.error("UnityAssetHandler", `Error extracting asset ${asset.name} (${asset.type}):`, error);
                }
            }
        } catch (error) {
            Logger.error("UnityAssetHandler", "Error during asset extraction:", error);
        }
    }

    private getFileExtension(asset: UnityAsset): string {
        switch (asset.type) {
            case "Texture2D":
                return asset.format?.toLowerCase() || "bin";
            case "TextAsset":
                return asset.format === "JSON" ? "json" : asset.format === "XML" ? "xml" : "txt";
            case "AudioClip":
                return asset.format?.toLowerCase() || "bin";
            default:
                return "bin";
        }
    }

    // Fallback method using simplified type detection
    private async extractUsingBasicTypes(outputPath: string): Promise<void> {
        try {
            // Create output directory if it doesn't exist
            if (!existsSync(outputPath)) {
                mkdirSync(outputPath, { recursive: true });
            }

            // Try to detect if this is a text asset
            if (this.isTextAsset()) {
                await this.extractTextAsset(outputPath);
                return;
            } else if (this.isImageAsset()) {
                const extension = this.determineImageExtension();
                const fileName = path.basename(outputPath);
                await Bun.write(path.join(path.dirname(outputPath), `${fileName}.${extension}`), this.buffer);
                Logger.info("UnityAssetHandler", `Extracted image asset: ${fileName}.${extension}`);
                return;
            } else if (this.isAudioAsset()) {
                const extension = this.determineAudioExtension();
                const fileName = path.basename(outputPath);
                await Bun.write(path.join(path.dirname(outputPath), `${fileName}.${extension}`), this.buffer);
                Logger.info("UnityAssetHandler", `Extracted audio asset: ${fileName}.${extension}`);
                return;
            }

            // If can't identify type, save with original name but .bin extension
            const fileName = path.basename(outputPath);
            await Bun.write(path.join(path.dirname(outputPath), `${fileName}.bin`), this.buffer);
            Logger.info("UnityAssetHandler", `Extracted unknown asset type as: ${fileName}.bin`);
        } catch (error) {
            Logger.error("UnityAssetHandler", `Error in basic extraction: ${error}`);
        }
    }

    private isTextAsset(): boolean {
        // Check if this is a text asset
        try {
            // Try to parse as JSON or check for text patterns
            const sample = this.buffer.slice(0, Math.min(1000, this.buffer.length)).toString("utf-8");

            // Check for JSON pattern
            if (sample.trim().startsWith("{") && sample.includes('"')) {
                try {
                    JSON.parse(sample);
                    return true;
                } catch {
                    // Not valid JSON
                }
            }

            // Check for XML pattern
            if (sample.trim().startsWith("<") && sample.includes(">")) {
                return true;
            }

            // Check for plain text (most characters are printable ASCII)
            const printableChars = sample.match(/[\x20-\x7E\n\r\t]/g);
            return printableChars !== null && printableChars.length > sample.length * 0.8;
        } catch {
            return false;
        }
    }

    private isImageAsset(): boolean {
        // Check for common image headers
        const header = this.buffer.slice(0, 8);

        // PNG signature: 89 50 4E 47 0D 0A 1A 0A
        if (header[0] === 0x89 && header[1] === 0x50 && header[2] === 0x4e && header[3] === 0x47) {
            return true;
        }

        // JPEG signature: FF D8 FF
        if (header[0] === 0xff && header[1] === 0xd8 && header[2] === 0xff) {
            return true;
        }

        // GIF signature: 47 49 46 38
        if (header[0] === 0x47 && header[1] === 0x49 && header[2] === 0x46 && header[3] === 0x38) {
            return true;
        }

        // DDS signature: 44 44 53 20
        if (header[0] === 0x44 && header[1] === 0x44 && header[2] === 0x53 && header[3] === 0x20) {
            return true;
        }

        return false;
    }

    private isAudioAsset(): boolean {
        // Check for common audio headers
        const header = this.buffer.slice(0, 8);

        // RIFF/WAV signature: 52 49 46 46
        if (header[0] === 0x52 && header[1] === 0x49 && header[2] === 0x46 && header[3] === 0x46) {
            return true;
        }

        // OGG signature: 4F 67 67 53
        if (header[0] === 0x4f && header[1] === 0x67 && header[2] === 0x67 && header[3] === 0x53) {
            return true;
        }

        // MP3 signature (ID3): 49 44 33
        if (header[0] === 0x49 && header[1] === 0x44 && header[2] === 0x33) {
            return true;
        }

        // MP3 frame sync: FF FB or FF FA
        if (header[0] === 0xff && (header[1] === 0xfb || header[1] === 0xfa)) {
            return true;
        }

        return false;
    }

    private determineImageExtension(): string {
        // Check for common image headers
        const header = this.buffer.slice(0, 8);

        // PNG signature: 89 50 4E 47 0D 0A 1A 0A
        if (header[0] === 0x89 && header[1] === 0x50 && header[2] === 0x4e && header[3] === 0x47) {
            return "png";
        }

        // JPEG signature: FF D8 FF
        if (header[0] === 0xff && header[1] === 0xd8 && header[2] === 0xff) {
            return "jpg";
        }

        // GIF signature: 47 49 46 38
        if (header[0] === 0x47 && header[1] === 0x49 && header[2] === 0x46 && header[3] === 0x38) {
            return "gif";
        }

        // DDS signature: 44 44 53 20
        if (header[0] === 0x44 && header[1] === 0x44 && header[2] === 0x53 && header[3] === 0x20) {
            return "dds";
        }

        return "bin";
    }

    private determineAudioExtension(): string {
        // Check for common audio headers
        const header = this.buffer.slice(0, 8);

        // RIFF/WAV signature: 52 49 46 46
        if (header[0] === 0x52 && header[1] === 0x49 && header[2] === 0x46 && header[3] === 0x46) {
            return "wav";
        }

        // OGG signature: 4F 67 67 53
        if (header[0] === 0x4f && header[1] === 0x67 && header[2] === 0x67 && header[3] === 0x53) {
            return "ogg";
        }

        // MP3 signature (ID3): 49 44 33
        if (header[0] === 0x49 && header[1] === 0x44 && header[2] === 0x33) {
            return "mp3";
        }

        // MP3 frame sync: FF FB or FF FA
        if (header[0] === 0xff && (header[1] === 0xfb || header[1] === 0xfa)) {
            return "mp3";
        }

        return "bin";
    }

    private async extractTextAsset(outputPath: string): Promise<void> {
        try {
            const data = this.buffer;
            let textContent = "";

            // Check if this is likely an encrypted text asset from Arknights
            const isLikelyEncrypted =
                data.length > 128 &&
                this.buffer[0] !== 0x7b && // Not starting with {
                this.buffer[0] !== 0x3c; // Not starting with <

            if (isLikelyEncrypted) {
                try {
                    Logger.info("UnityAssetHandler", "Attempting to decrypt TextAsset...");

                    // Check if this is a gamedata/levels file (special case in Python)
                    const pathLower = outputPath.toLowerCase();
                    const isLevelData = pathLower.includes("gamedata/levels") || pathLower.includes("gamedata\\levels");
                    const isEnemyData = pathLower.includes("enemydata");

                    // Apply appropriate decryption based on file type
                    let decryptedData: Buffer;

                    if (isLevelData && !isEnemyData) {
                        // Special case for level data: skip the first 128 bytes
                        decryptedData = data.slice(128);
                        Logger.debug("UnityAssetHandler", "Applied level data special case (skipping 128 bytes)");
                    } else {
                        // Regular case: apply the ArkAssets decryption
                        decryptedData = ArkAssets.textAssetDecrypt(data, !isLevelData);
                        Logger.debug("UnityAssetHandler", "Applied standard TextAsset decryption");
                    }

                    // Try to parse the decrypted data
                    try {
                        textContent = decryptedData.toString("utf-8");

                        // Check if it's valid JSON
                        try {
                            const jsonObj = JSON.parse(textContent);
                            textContent = JSON.stringify(jsonObj, null, 2);
                            const extension = ".json";

                            const fileName = path.basename(outputPath);
                            const outFile = path.join(path.dirname(outputPath), `${fileName}${extension}`);

                            await Bun.write(outFile, textContent);
                            Logger.info("UnityAssetHandler", `Extracted and decrypted JSON TextAsset: ${outFile}`);
                            return;
                        } catch {
                            // Not JSON, continue with other formats
                        }
                    } catch {
                        Logger.warn("UnityAssetHandler", "Failed to decode decrypted data as UTF-8, using raw buffer");

                        // If we can't decode as text, save the raw buffer
                        const fileName = path.basename(outputPath);
                        const outFile = path.join(path.dirname(outputPath), `${fileName}.bin`);

                        await Bun.write(outFile, decryptedData);
                        Logger.info("UnityAssetHandler", `Extracted decrypted binary TextAsset: ${outFile}`);
                        return;
                    }
                } catch (decryptError) {
                    Logger.warn("UnityAssetHandler", `Decryption failed, falling back to raw data: ${decryptError}`);
                    // Continue with raw data
                }
            }

            // If we reach here, either the file wasn't encrypted or decryption failed
            // Determine the content type from the buffer
            try {
                textContent = data.toString("utf-8");

                // Check for JSON
                let extension = ".txt";
                try {
                    const jsonObj = JSON.parse(textContent);
                    textContent = JSON.stringify(jsonObj, null, 2);
                    extension = ".json";
                } catch {
                    // Check for XML
                    if (textContent.trim().startsWith("<") && textContent.includes("</")) {
                        extension = ".xml";
                    } else if (textContent.includes("\0")) {
                        // Binary data with nulls
                        extension = ".bin";
                    }
                }

                const fileName = path.basename(outputPath);
                const outFile = path.join(path.dirname(outputPath), `${fileName}${extension}`);

                await Bun.write(outFile, textContent);
                Logger.info("UnityAssetHandler", `Extracted text asset: ${outFile}`);
            } catch {
                // If we can't parse as text, save as binary
                const fileName = path.basename(outputPath);
                const outFile = path.join(path.dirname(outputPath), `${fileName}.bin`);

                await Bun.write(outFile, data);
                Logger.info("UnityAssetHandler", `Extracted binary asset: ${outFile}`);
            }
        } catch (error) {
            Logger.error("UnityAssetHandler", `Error extracting text asset: ${error}`);
        }
    }

    private async extractImageAsset(outputPath: string): Promise<void> {
        try {
            // Determine image format
            let extension = "bin";

            if (this.buffer[0] === 0x89 && this.buffer[1] === 0x50) {
                extension = "png";
            } else if (this.buffer[0] === 0xff && this.buffer[1] === 0xd8) {
                extension = "jpg";
            } else if (this.buffer[0] === 0x47 && this.buffer[1] === 0x49) {
                extension = "gif";
            } else if (this.buffer[0] === 0x44 && this.buffer[1] === 0x44) {
                extension = "dds";
            }

            // Save the raw image data
            await Bun.write(path.join(outputPath, `image.${extension}`), this.buffer);
        } catch (error) {
            console.error("Error extracting image asset:", error);
        }
    }

    private async extractAudioAsset(outputPath: string): Promise<void> {
        try {
            // Determine audio format
            let extension = "bin";

            if (this.buffer[0] === 0x52 && this.buffer[1] === 0x49) {
                extension = "wav";
            } else if (this.buffer[0] === 0x4f && this.buffer[1] === 0x67) {
                extension = "ogg";
            } else if (this.buffer[0] === 0x49 && this.buffer[1] === 0x44) {
                extension = "mp3";
            } else if (this.buffer[0] === 0xff && (this.buffer[1] === 0xfb || this.buffer[1] === 0xfa)) {
                extension = "mp3";
            }

            // Save the raw audio data
            await Bun.write(path.join(outputPath, `audio.${extension}`), this.buffer);
        } catch (error) {
            console.error("Error extracting audio asset:", error);
        }
    }

    // Helper method to format file sizes in human-readable format
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

    // Method to check if the asset contains any extractable content
    async hasExtractableContent(): Promise<boolean> {
        // Check for common file signatures that we can extract
        if (this.isTextAsset() || this.isImageAsset() || this.isAudioAsset()) {
            return true;
        }

        // Check the header to see if it's a valid Unity asset
        if (this.header && this.header.signature && this.header.signature.startsWith("Unity")) {
            return true;
        }

        // Check if we identified any assets during parsing
        if (this.assets && this.assets.length > 0) {
            return true;
        }

        // If the file is very small, it might be empty or not contain useful data
        if (this.buffer.length < 1024) {
            return false;
        }

        // Additional checks for specific asset bundle markers
        const sample = this.buffer.slice(0, Math.min(1000, this.buffer.length)).toString("utf-8", 0, 100);
        if (sample.includes("UnityFS") || sample.includes("Unity Web")) {
            return true;
        }

        // Check for other common data formats
        const header = this.buffer.slice(0, 16);
        const commonFormats = [
            [0x89, 0x50, 0x4e, 0x47], // PNG
            [0xff, 0xd8, 0xff], // JPEG
            [0x52, 0x49, 0x46, 0x46], // RIFF/WAV
            [0x4f, 0x67, 0x67, 0x53], // OGG
            [0x49, 0x44, 0x33], // MP3 (ID3)
        ];

        for (const format of commonFormats) {
            let matches = true;
            for (let i = 0; i < format.length; i++) {
                if (header[i] !== format[i]) {
                    matches = false;
                    break;
                }
            }
            if (matches) return true;
        }

        return false;
    }
}

// Types for Unity asset bundle parsing
interface UnityBundleHeader {
    signature: string;
    version: number;
    unityVersion: string;
    unityRevision: string;
    size: bigint;
    compressedBlocksInfoSize: number;
    uncompressedBlocksInfoSize: number;
    flags: number;
}

interface UnityAsset {
    type: string;
    format?: string;
    offset: number;
    size: number;
    name: string;
}

enum Servers {
    OFFICIAL = 0,
    BILIBILI = 1,
}

interface AssetInfo {
    totalSize: number;
    abSize: number;
    md5: string;
}

interface PackageInfo {
    totalSize: number;
    files: Record<string, AssetInfo>;
}

interface HotUpdateList {
    [key: string]: PackageInfo;
}

class ArkAssets {
    private static readonly CHAT_MASK = Buffer.from("554954704169383270484157776e7a7148524d4377506f6e4a4c49423357436c", "hex").toString();
    private server: Servers;
    private assetVersion: string = "";
    private clientVersion: string = "";
    private hotUpdateList: HotUpdateList = { other: { totalSize: 0, files: {} } };
    private totalSize: number = 0;
    private abSize: number = 0;

    constructor(server: Servers = Servers.OFFICIAL) {
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

    static async getVersion(server: Servers = Servers.OFFICIAL): Promise<[string, string]> {
        const response = await fetch(`https://ak-conf.hypergryph.com/config/prod/${server === Servers.OFFICIAL ? "official" : "b"}/Android/version`);
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
        const response = await fetch(`https://ak.hycdn.cn/assetbundle/${this.server === Servers.OFFICIAL ? "official" : "bilibili"}/Android/assets/${this.assetVersion}/hot_update_list.json`);
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
        const url = `https://ak.hycdn.cn/assetbundle/${this.server === Servers.OFFICIAL ? "official" : "bilibili"}/Android/assets/${this.assetVersion}/${pathToAsset
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
