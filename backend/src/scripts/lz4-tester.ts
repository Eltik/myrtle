/**
 * LZ4 Test Script
 * This script tests the LZ4 implementation by scanning a directory of
 * downloaded Unity assets and trying to decompress any LZ4 compressed data.
 */

import { existsSync, readdirSync } from "fs";
import { join } from "path";
import { Buffer } from "buffer";
import lz4 from "lz4js";

// Unity asset handling
class UnityAssetScanner {
    private filePath: string;
    private fileBuffer: Buffer;

    constructor(filePath: string, fileBuffer: Buffer) {
        this.filePath = filePath;
        this.fileBuffer = fileBuffer;
    }

    scan(): boolean {
        try {
            // Check if the file starts with the Unity signature
            const signature = this.fileBuffer.slice(0, 8).toString("utf8");
            if (signature !== "UnityFS") {
                // Not a Unity asset bundle
                return false;
            }

            // Parse the header to check for LZ4 compression
            const view = new DataView(this.fileBuffer.buffer);

            const flags = view.getUint32(60, true);
            const compressionType = flags & 0x3f;

            if (compressionType === 2 || compressionType === 3) {
                console.log("\n-----------------------------------------");
                console.log(`🎯 Found LZ4 compressed asset: ${this.filePath}`);
                console.log(`   Compression type: ${compressionType === 2 ? "LZ4" : "LZ4HC"}`);

                // Get compressed and uncompressed sizes
                const compressedSize = view.getUint32(52, true);
                const uncompressedSize = view.getUint32(56, true);
                console.log(`   Compressed size: ${compressedSize} bytes`);
                console.log(`   Uncompressed size: ${uncompressedSize} bytes`);

                // Try to decompress
                console.log("\n   Attempting LZ4 decompression...");

                try {
                    // Get the compressed data (after header)
                    const compressedData = this.fileBuffer.slice(64, 64 + compressedSize);

                    // Decompress
                    const startTime = Date.now();
                    const decompressedData = lz4.decompress(compressedData);
                    const endTime = Date.now();

                    console.log(`   ✅ Decompression successful!`);
                    console.log(`   Decompressed size: ${decompressedData.length} bytes`);
                    console.log(`   Time taken: ${endTime - startTime}ms`);
                    console.log("-----------------------------------------\n");

                    return true;
                } catch (err: any) {
                    console.log(`   ❌ Decompression failed: ${err.message}`);
                    console.log("-----------------------------------------\n");
                    return false;
                }
            }

            return false;
        } catch {
            // Not a valid Unity asset or other error
            return false;
        }
    }
}

// Main script
async function main() {
    const assetDir = process.argv[2];

    if (!assetDir || !existsSync(assetDir)) {
        console.error("Please provide a valid directory of downloaded assets");
        console.error("Usage: bun run lz4-tester.ts <path-to-assets>");
        process.exit(1);
    }

    console.log("\n==============================================");
    console.log("🔍 LZ4 Compression Tester");
    console.log("==============================================\n");

    console.log(`Scanning directory: ${assetDir}`);

    // Function to recursively scan directories
    const scanDirectory = async (dir: string) => {
        const entries = readdirSync(dir, { withFileTypes: true });

        let totalFiles = 0;
        let totalLz4Files = 0;

        for (const entry of entries) {
            const fullPath = join(dir, entry.name);

            if (entry.isDirectory()) {
                // Recursively scan subdirectories
                const [subFiles, subLz4] = await scanDirectory(fullPath);
                totalFiles += subFiles;
                totalLz4Files += subLz4;
            } else if (entry.isFile() && entry.name.endsWith(".ab")) {
                // Found a Unity asset bundle file (.ab)
                totalFiles++;

                try {
                    // Read the file
                    const fileBuffer = await Bun.file(fullPath).arrayBuffer();
                    const buffer = Buffer.from(fileBuffer);

                    // Scan for LZ4 compression
                    const scanner = new UnityAssetScanner(fullPath, buffer);
                    const hasLz4 = scanner.scan();

                    if (hasLz4) {
                        totalLz4Files++;
                    }
                } catch (err: any) {
                    console.error(`Error reading file ${fullPath}: ${err.message || err}`);
                }
            }
        }

        return [totalFiles, totalLz4Files];
    };

    try {
        const [totalFiles, totalLz4Files] = await scanDirectory(assetDir);

        console.log("\n==============================================");
        console.log(`Scan complete! Found ${totalLz4Files} LZ4 compressed files out of ${totalFiles} Unity asset bundles.`);
        console.log("==============================================\n");

        if (totalLz4Files === 0) {
            console.log("No LZ4 compressed files found.");
            console.log("This could be because:");
            console.log("1. The assets in this directory don't use LZ4 compression");
            console.log("2. The asset format has changed and detection isn't working");
            console.log("\nTry downloading different packages or assets.");
        }
    } catch (err: any) {
        console.error("Scan failed:", err.message || err);
        process.exit(1);
    }
}

main();
