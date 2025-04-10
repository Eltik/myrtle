/**
 * Simplified Arknights Asset Downloader
 * This script downloads a small subset of Arknights assets with cleaner logging
 */

import { existsSync, mkdirSync } from "fs";
import ArkAssets from "../lib/impl/local/impl/handler/impl/ark-downloader";

// Setup variables
const downloadDir = process.argv[2] || "./downloads";
const singleThreadMode = true;
const focusPackage = process.env.FOCUS_PACKAGE || "other"; // 'other' tends to have more reliable assets

// Set log level to INFO by default
process.env.ARKNIGHTS_LOG_LEVEL = process.env.ARKNIGHTS_LOG_LEVEL || "INFO";

// Create download directory if it doesn't exist
if (!existsSync(downloadDir)) {
    mkdirSync(downloadDir, { recursive: true });
    console.log(`Created download directory: ${downloadDir}`);
}

async function run() {
    console.clear(); // Clear the terminal for better visibility

    console.log("\n--------------------------------------------------");
    console.log("🚀 Simplified Arknights Asset Downloader");
    console.log("--------------------------------------------------\n");
    console.log(`📁 Download directory: ${downloadDir}`);
    console.log(`🧵 Using single thread mode: ${singleThreadMode ? "Yes" : "No"}`);
    console.log(`📦 Focusing on package: ${focusPackage}`);
    console.log(`🔊 Log level: ${process.env.ARKNIGHTS_LOG_LEVEL}`);
    console.log("\n--------------------------------------------------\n");

    // Initialize downloader
    console.log("Initializing downloader...");
    const arkAssets = new ArkAssets();
    await arkAssets.initialize();

    // Get available packages
    const availablePackages = arkAssets.getHotUpdateListKeys();

    // Check if our focus package exists
    if (!availablePackages.includes(focusPackage)) {
        console.error(`❌ Package '${focusPackage}' not found!`);
        console.log("Available packages:", availablePackages.join(", "));
        process.exit(1);
    }

    console.log("\n--------------------------------------------------");
    console.log(`Starting download of package: ${focusPackage}`);
    console.log("--------------------------------------------------\n");

    try {
        // Download only the specified package with a single thread for cleaner logs
        await arkAssets.downloadFromList([focusPackage], downloadDir, singleThreadMode ? 1 : 4);

        console.log("\n--------------------------------------------------");
        console.log("✅ Download completed successfully!");
        console.log(`💾 Assets downloaded to: ${downloadDir}`);
        console.log("--------------------------------------------------\n");

        // List files to scan for LZ4 compression
        console.log("\n📦 To scan the downloaded files for LZ4 compression, run:");
        console.log(`   bun run scan-lz4 ${downloadDir}`);
        console.log("\n--------------------------------------------------\n");
    } catch (error) {
        console.error("\n--------------------------------------------------");
        console.error("❌ Download failed with error:");
        console.error(error);
        console.error("--------------------------------------------------\n");
        process.exit(1);
    }
}

// Run the script
run().catch((error) => {
    console.error("Fatal error:", error);
    process.exit(1);
});
