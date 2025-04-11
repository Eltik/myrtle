/**
 * Simplified Arknights Asset Unpacker
 * This script unpacks Arknights assets with cleaner logging
 */

import { existsSync, mkdirSync } from "fs";
import { resolve } from "path";
import { runArkUnpacker } from "../lib/impl/local/impl/assets/ark-unpacker";

// Parse command line arguments properly
const args = process.argv.slice(2);
const flagArgs = args.filter((arg) => arg.startsWith("-"));
const positionalArgs = args.filter((arg) => !arg.startsWith("-"));

// Setup variables - only use non-flag arguments as positional parameters
const inputDir = positionalArgs[0] || "./downloads";
const outputDir = positionalArgs[1] || "./unpacked";
const forceReprocess = flagArgs.includes("--force") || flagArgs.includes("-f");
const debugMode = flagArgs.includes("--debug");
const listDirs = flagArgs.includes("--list-dirs");
const processAll = flagArgs.includes("--all");

// Directory selection
let selectedDirs: string[] = [];
const dirArg = flagArgs.find((arg) => arg.startsWith("--dirs=") || arg.startsWith("-d="));
if (dirArg) {
    const dirVal = dirArg.split("=")[1];
    if (dirVal) {
        selectedDirs = dirVal.split(",").map((dir) => dir.trim());
    }
}

// If no specific options are provided (no --all, no dirs, and not listing), default to some common dirs
if (!processAll && selectedDirs.length === 0 && !listDirs) {
    selectedDirs = ["arts", "chararts", "skinpack", "ui"];
    console.log(`No directories specified. Using default directories: ${selectedDirs.join(", ")}`);
}

async function run() {
    console.clear(); // Clear the terminal for better visibility

    console.log("\n--------------------------------------------------");
    console.log("🧩 Simplified Arknights Asset Unpacker");
    console.log("--------------------------------------------------\n");
    console.log(`📁 Input directory: ${resolve(inputDir)}`);

    // Check if input directory exists
    if (!existsSync(inputDir)) {
        console.error("\n--------------------------------------------------");
        console.error(`❌ Input directory does not exist: ${resolve(inputDir)}`);
        console.error("Please specify a valid input directory or download assets first.");
        console.error("--------------------------------------------------\n");
        process.exit(1);
    }

    // Show compatibility notice
    console.log("\n--------------------------------------------------");
    console.log("⚠️  IMPORTANT NOTICE: ASSET COMPATIBILITY ISSUES");
    console.log("--------------------------------------------------");
    console.log("The downloaded Arknights assets are in a custom compression format");
    console.log("that the current unpacker cannot decompress properly.");
    console.log("");
    console.log("While the script can list the available directories, extracting");
    console.log("assets will not work due to how Arknights packages their assets.");
    console.log("");
    console.log("Options:");
    console.log("1. Use the Arknights Wiki/Gamepress for game assets");
    console.log("2. Extract assets directly from the game on a rooted device");
    console.log("3. Use specialized decompression tools designed for Arknights");
    console.log("--------------------------------------------------\n");

    if (listDirs) {
        console.log("\n--------------------------------------------------");
        console.log("📋 Listing available directories...");
        console.log("--------------------------------------------------\n");

        const result = await runArkUnpacker({
            inputDir,
            listDirs: true,
            debug: debugMode,
        });

        if (!result.success) {
            console.error("\n--------------------------------------------------");
            console.error("❌ Failed to list directories:");
            console.error(result.message);
            console.error("--------------------------------------------------\n");
            process.exit(1);
        }

        process.exit(0);
    }

    // If user wants to try unpacking anyway
    const proceedAnyway = flagArgs.includes("--force-try") || flagArgs.includes("--proceed-anyway");

    if (!proceedAnyway) {
        console.log("To attempt unpacking anyway, run with the --proceed-anyway flag.");
        console.log("However, note that it's unlikely to extract any assets.");
        process.exit(0);
    }

    console.log("\n--------------------------------------------------");
    console.log("Proceeding with unpacking attempt as requested...");
    console.log("--------------------------------------------------\n");

    console.log(`📂 Output directory: ${resolve(outputDir)}`);
    console.log(`🔄 Force reprocess: ${forceReprocess ? "Yes" : "No"}`);
    console.log(`🔍 Debug mode: ${debugMode ? "Yes" : "No"}`);

    if (processAll) {
        console.log(`📦 Processing: All directories`);
    } else {
        console.log(`📦 Processing directories: ${selectedDirs.join(", ")}`);
    }

    // Create output directory
    if (!existsSync(outputDir)) {
        mkdirSync(outputDir, { recursive: true });
        console.log(`Created output directory: ${outputDir}`);
    }

    console.log("\n--------------------------------------------------\n");

    try {
        console.log("Starting unpacking process...\n");

        const result = await runArkUnpacker({
            inputDir,
            outputDir,
            directories: processAll ? undefined : selectedDirs,
            all: processAll,
            force: forceReprocess,
            debug: debugMode,
        });

        if (result.success) {
            console.log("\n--------------------------------------------------");
            console.log("✅ Unpacking completed successfully!");
            console.log(`💾 Assets unpacked to: ${resolve(outputDir)}`);
            console.log("--------------------------------------------------\n");

            // Display result statistics
            if (result.message) {
                console.log(result.message);
                console.log("\n--------------------------------------------------\n");
            }
        } else {
            console.error("\n--------------------------------------------------");
            console.error("❌ Unpacking failed with errors:");
            console.error(result.message);
            console.error("--------------------------------------------------\n");

            if (result.message.includes("no assets were extracted")) {
                console.log("💡 As expected, no assets were extracted due to Arknights' custom asset format.");
                console.log("The best options for getting game assets are:");
                console.log("1. Use the Arknights Wiki or Gamepress");
                console.log("2. Extract directly from the game using specialized tools");
                console.log("--------------------------------------------------\n");
            }

            process.exit(1);
        }
    } catch (error) {
        console.error("\n--------------------------------------------------");
        console.error("❌ Unpacking failed with error:");
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
