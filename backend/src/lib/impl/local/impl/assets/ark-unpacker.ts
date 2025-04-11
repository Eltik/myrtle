import { spawn } from "child_process";
import { resolve } from "path";
import { existsSync } from "fs";

interface UnpackerOptions {
    inputDir?: string;
    outputDir?: string;
    directories?: string[];
    force?: boolean;
    debug?: boolean;
    all?: boolean;
    listDirs?: boolean;
}

/**
 * Runs the Arknights asset unpacker Python script
 * @param options Configuration options for the unpacker
 * @returns Promise resolving to success status and output message
 */
export async function runArkUnpacker(options: UnpackerOptions = {}): Promise<{ success: boolean; message: string }> {
    const scriptPath = resolve(__dirname, "ark-unpacker.py");

    // Verify the Python script exists
    if (!existsSync(scriptPath)) {
        return { success: false, message: `Error: Python script not found at ${scriptPath}` };
    }

    // Build command arguments
    const args: string[] = [];

    if (options.inputDir) {
        args.push("-i", options.inputDir);
    }

    if (options.outputDir) {
        args.push("-o", options.outputDir);
    }

    // Handle directories or --all flag
    if (options.all) {
        args.push("--all");
    } else if (options.directories && options.directories.length > 0) {
        args.push("-d");
        // The Python script expects directories as separate arguments after -d
        // not as a comma-separated string
        args.push(...options.directories);
    }

    if (options.force) {
        args.push("-f");
    }

    if (options.debug) {
        args.push("--debug");
    }

    if (options.listDirs) {
        args.push("--list-dirs");
    }

    // For safety, ensure Python is available
    try {
        const pythonCommand = process.platform === "win32" ? "python" : "python3";

        return new Promise((resolve) => {
            console.log(`Running command: ${pythonCommand} ${scriptPath} ${args.join(" ")}`);

            const pythonProcess = spawn(pythonCommand, [scriptPath, ...args]);

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
                        // Extract success statistics
                        let stats = "";
                        const statsMatch = stdout.match(/Processing completed![\s\S]*?(?:={10,})/);
                        if (statsMatch) {
                            stats = statsMatch[0];
                        }

                        // Check if assets were actually extracted
                        if (stdout.includes("Total assets extracted: 0")) {
                            resolve({
                                success: false,
                                message: "Process completed but no assets were extracted. Please check your input directory.",
                            });
                        } else {
                            resolve({
                                success: true,
                                message: stats || "Unpacking completed successfully",
                            });
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
