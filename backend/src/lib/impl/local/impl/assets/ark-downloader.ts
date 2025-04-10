import { spawn } from "child_process";
import { resolve } from "path";
import { existsSync } from "fs";

interface DownloaderOptions {
    outputDir?: string;
    packages?: string[];
    all?: boolean;
    server?: 0 | 1; // 0=Official, 1=Bilibili
    interactive?: boolean;
    listOnly?: boolean; // New option to only list packages without downloading
}

/**
 * Runs the Arknights asset downloader Python script
 * @param options Configuration options for the downloader
 * @returns Promise resolving to success status and output message
 */
export async function runArkDownloader(options: DownloaderOptions = {}): Promise<{ success: boolean; message: string }> {
    const scriptPath = resolve(__dirname, "ark-downloader.py");

    // Verify the Python script exists
    if (!existsSync(scriptPath)) {
        return { success: false, message: `Error: Python script not found at ${scriptPath}` };
    }

    // Build command arguments
    const args: string[] = [scriptPath];

    if (options.outputDir) {
        args.push("-o", options.outputDir);
    }

    // List packages only without downloading if listOnly is true
    if (options.listOnly) {
        // Don't add other download-related flags
    }
    // Handle packages or --all flag
    else if (options.all || (!options.packages && !options.interactive)) {
        // Default to downloading all packages if not specified otherwise
        args.push("--all");
    } else if (options.packages && options.packages.length > 0) {
        args.push("-p", options.packages.join(","));
    }

    // Set server if specified
    if (options.server !== undefined) {
        args.push("-s", options.server.toString());
    }

    // Use interactive mode if specifically requested
    if (options.interactive) {
        args.push("--interactive");
    }

    // For safety, ensure Python is available
    try {
        const pythonCommand = process.platform === "win32" ? "python" : "python3";

        return new Promise((resolve) => {
            console.log(`Running command: ${pythonCommand} ${args.join(" ")}`);

            const pythonProcess = spawn(pythonCommand, args);

            let stdout = "";
            let stderr = "";

            // Collect stdout data
            pythonProcess.stdout.on("data", (data) => {
                const output = data.toString();
                stdout += output;

                // Print output for monitoring
                process.stdout.write(output);
            });

            // Collect stderr data
            pythonProcess.stderr.on("data", (data) => {
                const output = data.toString();
                stderr += output;

                // Print error output in red
                process.stderr.write(`\x1b[31m${output}\x1b[0m`);
            });

            // Handle process completion
            pythonProcess.on("close", (code) => {
                if (code !== 0) {
                    // Check for specific error patterns
                    const errorMessage = stderr || "Unknown error occurred";
                    resolve({
                        success: false,
                        message: `Process exited with code ${code}: ${errorMessage}`,
                    });
                } else {
                    // Check for error patterns in output
                    const hasError = stderr.includes("Error:") || stderr.includes("Exception:") || stdout.includes("Error:") || stdout.includes("Failed to process");

                    if (hasError) {
                        let errorDetails = "";

                        // Extract relevant error sections
                        if (stderr.includes("Error:")) {
                            const match = stderr.match(/Error:.*(?:\n.*?)*/);
                            if (match) errorDetails += match[0] + "\n";
                        }

                        if (stdout.includes("Error:") || stdout.includes("Failed to process")) {
                            const matches = stdout.match(/(?:Error:|Failed to process).*(?:\n.*?)*/g);
                            if (matches) errorDetails += matches.join("\n");
                        }

                        resolve({
                            success: false,
                            message: `Completed with errors: ${errorDetails || stderr || stdout}`,
                        });
                    } else {
                        // If it's just listing packages, it's a success even without downloads
                        if (options.listOnly) {
                            // Extract list of packages
                            const packageList = stdout.match(/Available packages:[\s\S]*?(?=--|$)/);
                            resolve({
                                success: true,
                                message: packageList ? packageList[0] : "Package list retrieved successfully",
                            });
                        } else {
                            // Extract success statistics for downloads
                            let stats = "";
                            const statsMatch = stdout.match(/Processing completed![\s\S]*?(?:={10,})/);
                            if (statsMatch) {
                                stats = statsMatch[0];
                            }

                            // Check if assets were actually downloaded
                            if (stdout.includes("Total assets downloaded: 0")) {
                                resolve({
                                    success: false,
                                    message: "Process completed but no assets were downloaded. Please check your settings.",
                                });
                            } else {
                                resolve({
                                    success: true,
                                    message: stats || "Downloading completed successfully",
                                });
                            }
                        }
                    }
                }
            });

            // Handle process errors (e.g., Python not found)
            pythonProcess.on("error", (err) => {
                resolve({
                    success: false,
                    message: `Failed to start process: ${err.message}`,
                });
            });
        });
    } catch (error) {
        return {
            success: false,
            message: `Failed to execute Python script: ${error instanceof Error ? error.message : String(error)}`,
        };
    }
}
