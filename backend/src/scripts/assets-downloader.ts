/**
 * Simplified Arknights Asset Downloader
 * This script downloads Arknights assets using the Python downloader script
 */

import { existsSync, mkdirSync } from "fs";
import { spawn } from "child_process";
import { runArkDownloader } from "../lib/impl/local/impl/assets/ark-downloader";

// Parse command line arguments
const args = process.argv.slice(2);

// Setup variables
const downloadDir = process.env.DOWNLOAD_DIR || "./ArkAssets";
const pythonCommand = process.platform === "win32" ? "python" : "python3";

// Set debugging flags
const quietMode = args.includes("--quiet");

// Help flag
if (args.includes("--help") || args.includes("-h")) {
    console.log("\n--------------------------------------------------");
    console.log("🚀 Arknights Asset Downloader - Help");
    console.log("--------------------------------------------------");
    console.log("Usage: bun download [options]");
    console.log("\nOptions:");
    console.log("  --quiet         Less verbose output");
    console.log("  --help, -h      Show this help message");
    console.log("\nExamples:");
    console.log("  bun download");
    process.exit(0);
}

// Create download directory if it doesn't exist
if (!existsSync(downloadDir)) {
    mkdirSync(downloadDir, { recursive: true });
    console.log(`Created download directory: ${downloadDir}`);
}

async function run() {
    if (!quietMode) console.clear(); // Clear the terminal for better visibility

    console.log("\n--------------------------------------------------");
    console.log("🚀 Simplified Arknights Asset Downloader (Python-powered)");
    console.log("--------------------------------------------------\n");
    console.log(`📁 Download directory: ${downloadDir}`);
    console.log(`🐍 Using Python: ${pythonCommand}`);

    console.log("\n--------------------------------------------------\n");

    // Check if Python is available
    try {
        const pythonVersionCheck = spawn(pythonCommand, ["--version"]);
        let versionOutput = "";

        pythonVersionCheck.stdout.on("data", (data) => {
            versionOutput += data.toString();
        });

        pythonVersionCheck.stderr.on("data", (data) => {
            versionOutput += data.toString();
        });

        await new Promise<void>((resolve) => {
            pythonVersionCheck.on("close", (code) => {
                if (code !== 0) {
                    console.error(`\n❌ ${pythonCommand} is not available. Please install Python 3.`);
                    process.exit(1);
                } else {
                    console.log(`✅ Python detected: ${versionOutput.trim()}`);
                    resolve();
                }
            });
        });

        // Check for required Python packages
        // Note: pycryptodome is imported as 'Crypto', not 'pycryptodome'
        const requiredPackages = [
            { name: "UnityPy", importName: "UnityPy" },
            { name: "requests", importName: "requests" },
            { name: "tqdm", importName: "tqdm" },
            { name: "pycryptodome", importName: "Crypto" },
            { name: "bson", importName: "bson" },
            { name: "keyboard", importName: "keyboard" },
        ];
        console.log("Checking for required Python packages...");

        for (const pkg of requiredPackages) {
            try {
                const pkgCheck = spawn(pythonCommand, ["-c", `import ${pkg.importName}`]);
                await new Promise<void>((resolve, reject) => {
                    pkgCheck.on("close", (code) => {
                        if (code !== 0) {
                            console.error(`\n❌ Python package '${pkg.name}' is not installed.`);
                            console.error(`Please install it with: pip install ${pkg.name}`);
                            reject(new Error(`Missing Python package: ${pkg.name}`));
                        } else {
                            console.log(`✅ Python package detected: ${pkg.name}`);
                            resolve();
                        }
                    });
                });
            } catch {
                process.exit(1);
            }
        }
    } catch (err) {
        console.error(`\n❌ Failed to check Python environment: ${err}`);
        process.exit(1);
    }

    console.log("\n--------------------------------------------------");
    console.log("Starting download of Arknights assets");
    console.log("Press Ctrl+C to cancel. Download can be resumed later.");
    console.log("--------------------------------------------------\n");

    try {
        // Download using the Python script via our wrapper
        const result = await runArkDownloader({
            outputDir: downloadDir,
        });

        if (result.success) {
            console.log("\n--------------------------------------------------");
            console.log("✅ Download completed successfully!");
            console.log(`💾 Assets downloaded to: ${downloadDir}`);
            console.log("--------------------------------------------------\n");

            // Show asset compatibility notice
            console.log("\n--------------------------------------------------");
            console.log("⚠️  IMPORTANT: ASSET COMPATIBILITY");
            console.log("--------------------------------------------------");
            console.log("The downloaded Arknights assets are in a custom compression format");
            console.log("that can be difficult to extract. To view asset files, you can:");
            console.log("");
            console.log("1. List the downloaded directories:");
            console.log(`   bun unpack --list-dirs`);
            console.log("");
            console.log("2. Try the LZ4 compression scanner:");
            console.log(`   bun run scan-lz4 ${downloadDir}`);
            console.log("");
            console.log("For best results, consider using Arknights Wiki/Gamepress");
            console.log("for game assets rather than trying to extract them directly.");
            console.log("--------------------------------------------------\n");
        } else {
            console.error("\n--------------------------------------------------");
            console.error("❌ Download failed:");
            console.error(result.message);
            console.error("--------------------------------------------------\n");
            process.exit(1);
        }
    } catch (err) {
        console.error("\n--------------------------------------------------");
        console.error("❌ Download failed with error:");
        console.error(err);
        console.error("--------------------------------------------------\n");
        process.exit(1);
    }
}

// Run the script
run().catch((err) => {
    console.error("Fatal error:", err);
    process.exit(1);
});
