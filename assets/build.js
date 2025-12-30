import { fileURLToPath } from "node:url";
import path from "node:path";
import { execSync } from "node:child_process";
import os from "node:os";
import readline from "node:readline";
import fs, { existsSync } from "node:fs";

const RED = "\x1b[0;31m";
const GREEN = "\x1b[0;32m";
const YELLOW = "\x1b[1;33m";
const BLUE = "\x1b[0;34m";
const NC = "\x1b[0m";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const SCRIPT_DIR = __dirname;
const RUST_MIN_VERSION = "1.70.0";
let BUILD_MODE = process.env.BUILD_MODE ?? "release";

let CLEAN_BUILD = false;
let RUN_TESTS = false;
let SKIP_CHECKS = false;
let SKIP_BUILD = false;
let BUILD_UNITY_RS = true;
let BUILD_DOWNLOADER = true;
let BUILD_UNPACKER = true;

function printMsg(color, ...msg) {
    console.log(color + msg.join(" ") + NC);
}

function printHeader(title) {
    console.log("");
    printMsg(BLUE, "TPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPW");
    printMsg(BLUE, `Q  ${title}`);
    printMsg(BLUE, "ZPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPP]");
    console.log("");
}

function printStep(text) {
    printMsg(GREEN, "�", text);
}

function printError(text) {
    printMsg(RED, " ERROR:", text);
}

function printWarning(text) {
    printMsg(YELLOW, "� WARNING:", text);
}

function printSuccess(text) {
    printMsg(GREEN, "", text);
}

function versionCompare(ver1, ver2) {
    if (ver1 === ver2) return 0;

    const ver1Arr = ver1.split(".");
    const ver2Arr = ver2.split(".");

    for (let i = 0; i < ver1Arr.length; i++) {
        const a = Number.parseInt(ver1Arr[i], 10);
        const b = Number.parseInt(ver2Arr[i] ?? "0", 10);

        if (a > b) return 1;
        if (a < b) return 2;
    }

    return 0;
}

async function ask(question) {
    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout
    });
    return new Promise(resolve => rl.question(question, ans => {
        rl.close();
        resolve(ans);
    }));
}

function checkRust() {
    printStep("Checking rust installation...");

    let rustVersion;
    try {
        const output = execSync("rustc --version", { encoding: "utf8" }).toString().trim();
        rustVersion = output.split(" ")[1].trim();
    } catch (_) {
        printError("Rust is not installed");
        return 1;
    }

    printMsg("", `  Found Rust version: ${rustVersion}`);

    const result = versionCompare(rustVersion, RUST_MIN_VERSION);

    if (result === 2) {
        printError(
            `Rust version ${rustVersion} is too old. Minimum required: ${RUST_MIN_VERSION}`
        );
        return 1;
    }

    printSuccess("Rust version is sufficient");
    return 0;
}

function installRust() {
    printStep("Installing Rust...");

    const platform = os.platform();

    if (platform === "darwin" || platform === "linux") {
        printMsg(NC, "Running rustup installer");

        try {
            // Run rustup installer
            execSync(
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
                { stdio: "inherit", shell: "/bin/bash" }
            );

            // Source ~/.cargo/env
            const cargoEnv = path.join(os.homedir(), ".cargo/env");
            if (fs.existsSync(cargoEnv)) {
                // Apply cargo env to current process
                const content = fs.readFileSync(cargoEnv, "utf8");
                // Very light parser: applies lines like `export PATH=...`
                content.split("\n").forEach(line => {
                    if (line.startsWith("export")) {
                        const [, key, value] = line.match(/export\s+(\w+)=(.*)/) ?? [];
                        if (key && value) process.env[key] = value.replace(/"/g, "");
                    }
                });
            }

            printSuccess("Rust installed successfully");
            return 0;
        } catch (err) {
            printError("Rust installation failed:" + err.message);
            return 1;
        }
    } else {
        printError("Automatic Rust installation is only supported on macOS and Linux.");
        printMsg(NC, "Please install Rust manually from: https://rustup.rs/");
        return 1;
    }
}

function checkCargo() {
    printStep("Installing Rust...");

    const platform = os.platform();
    const checkCommand = platform === "win32" ? "where cargo" : "command -v cargo";

    try {
        execSync(checkCommand, { stdio: "ignore" });
    } catch {
        printError("Cargo is not installed (should come with Rust)");
        return 1;
    }

    let cargoVersion;
    try {
        const output = execSync("cargo --version", { encoding: "utf8" }).toString().trim();
        cargoVersion = output.split(" ")[1].trim();
    } catch (_) {
        printError("Failed to read Cargo version!");
        return 1;
    }

    printMsg(NC, `  Found Cargo version: ${cargoVersion}`);
    printSuccess("Cargo is available");

    return 0;
}

async function checkFMOD() {
    printStep("Checking FMOD library...");

    let fmodFound = false;
    let fmodPath = null;
    let platform = os.platform();

    const exists = p => {
        try { return fs.existsSync(p); }
        catch { return false; }
    };

    if (platform === "darwin") {
        if (exists("/usr/local/lib/libfmod.dylib") || exists("/opt/homebrew/lib/libfmod.dylib")) {
            fmodFound = true;
            try {
                fmodPath = execSync(
                    "find /usr/local/lib /opt/homebrew/lib -name \"libfmod.dylib\" 2>/dev/null | head -1",
                    { encoding: "utf8" }
                ).trim();
            } catch {}
            printMsg(NC, `  Found FMOD at: ${fmodPath}`);
        }
    } else if (platform === "linux") {
        if (exists("/usr/local/lib/libfmod.so") || exists("/usr/lib/libfmod.so")) {
            fmodFound = true;
            try {
                fmodPath = execSync(
                    "find /usr/local/lib /usr/lib -name \"libfmod.so*\" 2>/dev/null | head -1",
                    { encoding: "utf8" }
                ).trim();
            } catch {}
            printMsg(NC, `  Found FMOD at: ${fmodPath}`);
        }
    } else if (platform === "win32") {
        // Check for FMOD in the unity-rs resources directory
        const is64bit = process.arch === "x64" || process.env.PROCESSOR_ARCHITECTURE === "AMD64";
        const fmodResourceDir = path.join(SCRIPT_DIR, "unity-rs", "src", "resources", "FMOD", "Windows", is64bit ? "x64" : "x86");

        if (exists(path.join(fmodResourceDir, "fmod.dll"))) {
            fmodFound = true;
            fmodPath = fmodResourceDir;
            printMsg(NC, `  Found FMOD at: ${fmodPath}`);
        } else {
            // Check system paths
            const systemPaths = [
                path.join(process.env.PROGRAMFILES || "C:\\Program Files", "FMOD SoundSystem", "FMOD Studio API Windows", "api", "core", "lib", is64bit ? "x64" : "x86"),
                path.join(process.env["PROGRAMFILES(X86)"] || "C:\\Program Files (x86)", "FMOD SoundSystem", "FMOD Studio API Windows", "api", "core", "lib", is64bit ? "x64" : "x86"),
            ];

            for (const sysPath of systemPaths) {
                if (exists(path.join(sysPath, "fmod.dll"))) {
                    fmodFound = true;
                    fmodPath = sysPath;
                    printMsg(NC, `  Found FMOD at: ${fmodPath}`);
                    break;
                }
            }
        }
    }

    if (fmodFound) {
        printSuccess("FMOD library is available.");
        return 0;
    } else {
        printWarning("FMOD library not found!");
        printMsg(NC, "  Audio extraction in unpacker may not work without FMOD");
        printMsg(NC, "");
        printMsg(NC, "  To install FMOD:");

        if (platform === "darwin") {
            printMsg(NC, "    1. Download from: https://www.fmod.com/download");
            printMsg(NC, "    2. Install FMOD Studio API");
            printMsg(NC, "    3. Copy libfmod.dylib to /usr/local/lib/");
        } else if (platform === "linux") {
            printMsg(NC, "    1. Download from: https://www.fmod.com/download");
            printMsg(NC, "    2. Extract and run: sudo cp libfmod.so.* /usr/local/lib/");
            printMsg(NC, "    3. Run: sudo ldconfig");
        } else if (platform === "win32") {
            printMsg(NC, "    1. Download FMOD Engine 2.02.22 (or any 2.02.xx) from: https://www.fmod.com/download");
            printMsg(NC, "    2. For MinGW builds, create import libraries using create-fmod-libs.bat");
            printMsg(NC, "    3. Copy DLLs and .a files to:");
            printMsg(NC, `       ${path.join(SCRIPT_DIR, "unity-rs", "src", "resources", "FMOD", "Windows", process.arch === "x64" ? "x64" : "x86")}`);
            printMsg(NC, "");
            printMsg(NC, "    See FMOD_INSTALLATION_GUIDE.md for detailed instructions");
        }
        printMsg(NC, "");

        const reply = await ask("  Continue without FMOD? (y/N): ");
        if (!/^y$/i.test(reply.trim())) {
            return 1;
        }

        return 0;
    }
}

function buildUnityRs() {
    printStep("Building unity-rs library...");

    const libDir = path.join(SCRIPT_DIR, "unity-rs");
    process.chdir(libDir);

    if (BUILD_MODE === "release") {
        printMsg(NC, "  Building in release mode (optimized)...");
        execSync("cargo build --release", { stdio: "inherit" });
    } else {
        printMsg(NC, "  Building in debug mode (faster compilation)...");
        execSync("cargo build", { stdio: "inherit" });
    }

    printSuccess("unity-rs library built successfully");
}

function buildDownloader() {
    printStep("Building arknights-downloader...");

    const downloaderDir = path.join(SCRIPT_DIR, "downloader");
    process.chdir(downloaderDir);

    if (BUILD_MODE === "release") {
        printMsg(NC, "  Building in release mode (optimized)...");
        execSync("cargo build --release", { stdio: "inherit" });
    } else {
        printMsg(NC, "  Building in debug mode (faster compilation)...");
        execSync("cargo build", { stdio: "inherit" });
    }

    printSuccess("arknights-downloader built successfully");
}

function buildUnpacker() {
    printStep("Building assets-unpacker...");

    const unpackerDir = path.join(SCRIPT_DIR, "unpacker");
    process.chdir(unpackerDir);

    // Check if fb_json_auto.rs exists - if not, generate it
    const fbJsonAutoPath = path.join(unpackerDir, "src", "fb_json_auto.rs");
    if (!existsSync(fbJsonAutoPath)) {
        printMsg(NC, "  Generating FlatBuffer JSON implementations (CN schemas)...");
        try {
            execSync("python3 generate_fb_json_impls.py", { stdio: "inherit" });
        } catch (err) {
            printWarning("Failed to generate FlatBuffer JSON impls: " + err.message);
            printMsg(NC, "  Run ./regenerate_fbs.sh manually if FlatBuffer decoding doesn't work");
        }
    }

    // Check if fb_json_auto_yostar.rs exists - if not, generate it
    const fbJsonAutoYostarPath = path.join(unpackerDir, "src", "fb_json_auto_yostar.rs");
    if (!existsSync(fbJsonAutoYostarPath)) {
        printMsg(NC, "  Generating FlatBuffer JSON implementations (Yostar/EN schemas)...");
        try {
            execSync("python3 generate_fb_json_impls_yostar.py", { stdio: "inherit" });
        } catch (err) {
            printWarning("Failed to generate Yostar FlatBuffer JSON impls: " + err.message);
            printMsg(NC, "  Yostar-specific tables (character_table, etc.) may not decode correctly");
        }
    }

    if (BUILD_MODE === "release") {
        printMsg(NC, "  Building in release mode (optimized)...");
        execSync("cargo build --release", { stdio: "inherit" });
    } else {
        printMsg(NC, "  Building in debug mode (faster compilation)...");
        execSync("cargo build", { stdio: "inherit" });
    }

    printSuccess("assets-unpacker built successfully");
}

function fixFMODPath() {
    const platform = os.platform();
    if (platform !== "darwin") {
        return 0;
    }

    printStep("Fixing FMOD library path for macOS...");

    const binaryPath = path.join(
        SCRIPT_DIR,
        "unpacker",
        "target",
        BUILD_MODE,
        "assets-unpacker"
    );

    if (!existsSync(binaryPath)) {
        printWarning(`Unpacker binary not found at: ${binaryPath}`);
        return 0;
    }

    const tryExec = (cmd) => {
        try {
            execSync(cmd, { stdio: "ignore" });
        } catch {
            // ignore failures, shell script used `|| true`
        }
    };

    tryExec(`install_name_tool -add_rpath /usr/local/lib "${binaryPath}"`);
    tryExec(`install_name_tool -add_rpath /opt/homebrew/lib "${binaryPath}"`);

    printSuccess("FMOD library path configured");

    return 1;
}

function runTests() {
    if (RUN_TESTS) {
        printHeader("Running Tests");

        printStep("Testing unity-rs...");
        process.chdir(path.join(SCRIPT_DIR, "unity-rs"));
        execSync("cargo test", { stdio: "inherit" });

        printStep("Testing downloader...");
        process.chdir(path.join(SCRIPT_DIR, "downloader"));
        execSync("cargo test", { stdio: "inherit" });

        printStep("Testing unpacker...");
        process.chdir(path.join(SCRIPT_DIR, "unpacker"));
        execSync("cargo test", { stdio: "inherit" });

        printSuccess("All tests passed");
    }
}

function cleanBuild() {
    printHeader("Cleaning Build Artifacts");

    printStep("Cleaning unity-rs...");
    process.chdir(path.join(SCRIPT_DIR, "lib"));
    execSync("cargo clean", { stdio: "inherit" });

    printStep("Cleaning downloader...");
    process.chdir(path.join(SCRIPT_DIR, "downloader"));
    execSync("cargo clean", { stdio: "inherit" });

    printStep("Cleaning unpacker...");
    process.chdir(path.join(SCRIPT_DIR, "unpacker"));
    execSync("cargo clean", { stdio: "inherit" });

    printSuccess("All build artifacts cleaned");
}

function printUsage(scriptName = process.argv[1]) {
    console.log("Usage: " + scriptName + " [OPTIONS]");
    console.log("");
    console.log("Build script for Arknights Assets toolchain");
    console.log("");
    console.log("Options:");
    console.log("  -h, --help              Show this help message");
    console.log("  -c, --clean             Clean build artifacts before building");
    console.log("  -d, --debug             Build in debug mode (faster compilation)");
    console.log("  -t, --test              Run tests after building");
    console.log("  --no-check              Skip dependency checks");
    console.log("  --skip-build            Skip the build process entirely");
    console.log("  --lib-only              Only build the unity-rs library");
    console.log("  --downloader-only       Only build the downloader");
    console.log("  --unpacker-only         Only build the unpacker");
    console.log("");
    console.log("Environment Variables:");
    console.log("  BUILD_MODE              Set to \"debug\" for debug builds (default: release)");
    console.log("  RUN_TESTS               Set to \"1\" to run tests after building");
    console.log("");
    console.log("Examples:");
    console.log("  " + scriptName + "                      # Build everything in release mode");
    console.log("  " + scriptName + " --debug              # Build everything in debug mode");
    console.log("  " + scriptName + " --clean              # Clean and rebuild");
    console.log("  " + scriptName + " --lib-only           # Only build unity-rs");
    console.log("  BUILD_MODE=debug " + scriptName + "     # Build in debug mode via env var");
    console.log("");
    console.log("FlatBuffer Schemas:");
    console.log("  If FlatBuffer schemas change, regenerate with:");
    console.log("    cd unpacker && ./regenerate_fbs.sh");
}

function printSummary() {
    printHeader("Build Summary");

    const modeText = BUILD_MODE === "debug" ? "debug (fast compilation)" : "release (optimized)";
    printMsg(GREEN, `Build mode: ${modeText}`);
    console.log("");

    printMsg(GREEN, "Built components:");

    if (BUILD_UNITY_RS) {
        const unityRsBinary = `${SCRIPT_DIR}/unity-rs/target/${BUILD_MODE}/libunity_rs.rlib`
        if (existsSync(unityRsBinary)) {
            printMsg(GREEN, `  • unity-rs library: ${unityRsBinary}`);
        } else {
            printMsg(RED, `  • unity-rs library: NOT FOUND`);
        }
    }
    if (BUILD_DOWNLOADER) {
        const downloaderBinary = `${SCRIPT_DIR}/downloader/target/${BUILD_MODE}/arknights-downloader`
        if (existsSync(downloaderBinary)) {
            printMsg(GREEN, `  • downloader: ${downloaderBinary}`);
        } else {
            printMsg(RED, `  • downloader: NOT FOUND`);
        }
    }
    if (BUILD_UNPACKER) {
        const unpackerBinary = `${SCRIPT_DIR}/unpacker/target/${BUILD_MODE}/assets-unpacker`
        if (existsSync(unpackerBinary)) {
            printMsg(GREEN, `  • unpacker: ${unpackerBinary}`);
        } else {
            printMsg(RED, `  • unpacker: NOT FOUND`);
        }
    }

    console.log("");
    printMsg(BLUE, "Next steps:");

    if (BUILD_DOWNLOADER === 1) {
        printMsg(NC, "  Run downloader:");
        printMsg(NC, "    cd downloader");
        printMsg(NC, `    cargo run --${BUILD_MODE} -- --help`);
    }

    if (BUILD_UNPACKER === 1) {
        printMsg(NC, "  Run unpacker:");
        printMsg(NC, "    cd unpacker");
        printMsg(NC, `    cargo run --${BUILD_MODE} -- --help`);
    }

    console.log("");
}

async function main() {
    const args = process.argv.slice(2);

    while (args.length > 0) {
        const arg = args.shift();
        switch (arg) {
            case "-h":
            // @biome-ignore biome lint/suspicious/noFallthroughSwitchClause
            case "--help":
                printUsage("");
                process.exit(0);

            case "-c":
            case "--clean":
                CLEAN_BUILD = 1;
                break;

            case "-d":
            case "--debug":
                BUILD_MODE = "debug";
                break;

            case "-t":
            case "--test":
                RUN_TESTS = 1;
                break;

            case "--no-check":
                SKIP_CHECKS = 1;
                break;

            case "--skip-build":
                SKIP_BUILD = true;
                break;

            case "--lib-only":
                BUILD_LIB = 1;
                BUILD_DOWNLOADER = 0;
                BUILD_UNPACKER = 0;
                break;

            case "--downloader-only":
                BUILD_LIB = 0;
                BUILD_DOWNLOADER = 1;
                BUILD_UNPACKER = 0;
                break;

            case "--unpacker-only":
                BUILD_LIB = 0;
                BUILD_DOWNLOADER = 0;
                BUILD_UNPACKER = 1;
                break;

            default:
                printError(`Unknown option: ${arg}`);
                printUsage("");
                process.exit(1);
        }
    }

    printHeader("Arknights Assets Build System");

    if (CLEAN_BUILD) {
        cleanBuild();
        console.log("");
    }

    if (!SKIP_CHECKS) {
        printHeader("Checking Dependencies");

        if (checkRust() === 1) {
            printMsg(NC, "");
            const reply = await ask("Would you like to install Rust automatically? (y/N):");
            if (!/^y$/i.test(reply.trim())) {
                installRust();
            } else {
                printError("Rust is required. Please install from https://rustup.rs/");
                process.exit(1);
            }
        }

        checkCargo();
        await checkFMOD();
        console.log("");
    }

    if (!SKIP_BUILD) {
        printHeader("Building Components");

        process.chdir(path.join(SCRIPT_DIR));

        if (BUILD_UNITY_RS) {
            buildUnityRs();
        }

        if (BUILD_DOWNLOADER) {
            buildDownloader();
        }

        if (BUILD_UNPACKER) {
            buildUnpacker();
            fixFMODPath();
        }

        runTests();
    } else {
        printMsg(YELLOW, "Build process skipped (--skip-build specified)");
    }

    process.chdir(path.join(SCRIPT_DIR));

    console.log("");
    printSummary();

    printSuccess("Build completed successfully.");
}

main().then("");
