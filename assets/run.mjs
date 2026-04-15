#!/usr/bin/env node

import { execFileSync, spawn } from "node:child_process";
import {
	copyFileSync,
	existsSync,
	mkdirSync,
	readdirSync,
	readFileSync,
	statSync,
	writeFileSync,
} from "node:fs";
import { readdir, stat } from "node:fs/promises";
import { dirname, join, sep } from "node:path";
import { fileURLToPath } from "node:url";

import boxen from "boxen";
import chalk from "chalk";
import inquirer from "inquirer";
import ora from "ora";
import { WebSocketServer } from "ws";

// ─── Constants ──────────────────────────────────────────────────────────────

const __dirname = dirname(fileURLToPath(import.meta.url));
const isWindows = process.platform === "win32";
const isMac = process.platform === "darwin";
const exe = isWindows ? ".exe" : "";
const BINARIES_DIR = join(__dirname, "binaries");
const DOWNLOADER_BIN = join(BINARIES_DIR, `downloader${exe}`);
const UNPACKER_BIN = join(BINARIES_DIR, `unpacker${exe}`);
const DOWNLOADER_BUILD = join(
	__dirname,
	"downloader",
	"target",
	"release",
	`downloader${exe}`,
);
const UNPACKER_BUILD = join(
	__dirname,
	"unpacker",
	"target",
	"release",
	`unpacker${exe}`,
);
const DEFAULT_THREADS = 2;
// Built via RegExp constructor to avoid a literal ESC control character in source
const ANSI_RE = new RegExp(`${String.fromCharCode(0x1b)}\\[[0-9;]*m`, "g");

// ─── Server Version URL Map ─────────────────────────────────────────────────
// Source: downloader/src/server.rs

const SERVERS = {
	en: {
		label: "Global/EN (Yostar)",
		versionUrl:
			"https://ark-us-static-online.yo-star.com/assetbundle/official/Android/version",
	},
	jp: {
		label: "Japan (Yostar)",
		versionUrl:
			"https://ark-jp-static-online.yo-star.com/assetbundle/official/Android/version",
	},
	kr: {
		label: "Korea (Yostar)",
		versionUrl:
			"https://ark-kr-static-online.yo-star.com/assetbundle/official/Android/version",
	},
	tw: {
		label: "Taiwan (Gryphline)",
		versionUrl:
			"https://ark-tw-static-online.yo-star.com/assetbundle/official/Android/version",
	},
	cn: {
		label: "CN Official (Hypergryph)",
		versionUrl:
			"https://ak-conf.hypergryph.com/config/prod/official/Android/version",
	},
	bilibili: {
		label: "CN Bilibili",
		versionUrl: "https://ak-conf.hypergryph.com/config/prod/b/Android/version",
	},
};

// ─── Shared Utilities ───────────────────────────────────────────────────────

function tryExec(cmd, args) {
	try {
		return execFileSync(cmd, args, {
			encoding: "utf-8",
			stdio: ["pipe", "pipe", "pipe"],
		}).trim();
	} catch {
		return null;
	}
}

function parseVersion(versionStr) {
	const match = versionStr?.match(/(\d+)\.(\d+)\.(\d+)/);
	if (!match) return null;
	return { major: +match[1], minor: +match[2], patch: +match[3] };
}

function versionGte(v, min) {
	if (v.major !== min.major) return v.major > min.major;
	if (v.minor !== min.minor) return v.minor > min.minor;
	return v.patch >= min.patch;
}

async function fetchServerVersion(serverKey) {
	const server = SERVERS[serverKey];
	if (!server) throw new Error(`Unknown server: ${serverKey}`);
	const res = await fetch(server.versionUrl);
	if (!res.ok) throw new Error(`HTTP ${res.status} from ${server.versionUrl}`);
	const data = await res.json();
	return { resVersion: data.resVersion, clientVersion: data.clientVersion };
}

function readStoredVersion(savedir) {
	const versionFile = join(savedir, ".version");
	try {
		return readFileSync(versionFile, "utf-8").trim();
	} catch {
		return null;
	}
}

function writeStoredVersion(savedir, resVersion) {
	mkdirSync(savedir, { recursive: true });
	writeFileSync(join(savedir, ".version"), resVersion, "utf-8");
}

/**
 * Check if the unpacker binary is newer than the last extraction timestamp.
 * Returns true if re-extraction is needed (binary was rebuilt since last extract).
 */
function unpackerIsNewer(savedir) {
	const stampFile = join(savedir, ".last_extract");
	try {
		const binMtime = statSync(UNPACKER_BIN).mtimeMs;
		const stampMtime = statSync(stampFile).mtimeMs;
		return binMtime > stampMtime;
	} catch {
		// Stamp doesn't exist → never extracted, or binary missing
		return existsSync(UNPACKER_BIN);
	}
}

/**
 * Check if the extraction output directory is missing or empty.
 * Returns true if re-extraction is needed because output doesn't exist.
 */
function outputMissingOrEmpty(outputDir) {
	try {
		const entries = readdirSync(outputDir);
		return entries.length === 0;
	} catch {
		// Directory doesn't exist
		return true;
	}
}

/** Touch the extraction timestamp file after a successful unpack. */
function touchExtractStamp(savedir) {
	mkdirSync(savedir, { recursive: true });
	writeFileSync(join(savedir, ".last_extract"), new Date().toISOString(), "utf-8");
}

function binariesExist() {
	return existsSync(DOWNLOADER_BIN) && existsSync(UNPACKER_BIN);
}

function formatBytes(bytes) {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
	if (bytes < 1024 * 1024 * 1024)
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

// ─── Progress Bar ───────────────────────────────────────────────────────────

function createProgressBar(label, { width = 30 } = {}) {
	let finished = false;
	let spinnerActive = false;
	let spinner = null;

	function startSpinner(text) {
		if (finished) return;
		spinner = ora(text).start();
		spinnerActive = true;
	}

	function render(completed, total) {
		if (finished) return;
		// Stop spinner on first real progress render
		if (spinnerActive && spinner) {
			spinner.stop();
			spinnerActive = false;
		}
		const percent = total > 0 ? completed / total : 0;
		const filled = Math.round(width * percent);
		const empty = width - filled;
		const bar =
			chalk.cyan("\u2588".repeat(filled)) + chalk.dim("\u2591".repeat(empty));
		const pct = (percent * 100).toFixed(1).padStart(5);
		const counts = `${String(completed).padStart(String(total).length)}/${total}`;
		process.stdout.write(
			`\r  ${chalk.bold(label)}  ${bar}  ${counts}  ${pct}%`,
		);
	}

	function clearLine() {
		process.stdout.write(`\r${" ".repeat(process.stdout.columns || 120)}\r`);
	}

	function succeed(message) {
		if (finished) return;
		finished = true;
		if (spinnerActive && spinner) spinner.stop();
		clearLine();
		console.log(`  ${chalk.green("\u2713")} ${message}`);
	}

	function fail(message) {
		if (finished) return;
		finished = true;
		if (spinnerActive && spinner) spinner.stop();
		clearLine();
		console.log(`  ${chalk.red("\u2717")} ${message}`);
	}

	return { startSpinner, render, succeed, fail };
}

// ─── Binary Progress Helpers ────────────────────────────────────────────────

/**
 * Spawn the downloader binary and track progress.
 *
 * indicatif suppresses all stderr output when not connected to a TTY,
 * so we track progress by polling the persistent_res_list.json manifest
 * which the downloader updates after each file completes.
 *
 * @param {object} opts
 * @param {string} opts.serverKey - Server region key
 * @param {string} opts.savedir - Save directory
 * @param {number} [opts.threads] - Concurrent download threads
 * @param {(p: {completed: number, total: number, percent: number}) => void} [opts.onProgress]
 * @returns {Promise<{downloaded: number, failed: number, totalBytes: number}>}
 */
function runDownload({
	serverKey,
	savedir,
	threads = DEFAULT_THREADS,
	onProgress,
}) {
	return new Promise((resolve, reject) => {
		const child = spawn(
			DOWNLOADER_BIN,
			[
				"--server",
				serverKey,
				"-d",
				savedir,
				"-t",
				String(threads),
				"download",
				"--all",
			],
			{
				cwd: __dirname,
				stdio: ["ignore", "pipe", "pipe"],
			},
		);

		activeChildren.add(child);
		child.on("close", () => activeChildren.delete(child));

		let stdoutBuf = "";
		let stderrRaw = "";
		let totalFiles = 0;
		let initialManifestCount = 0;
		let pollTimer = null;

		// Count entries in the manifest file to track progress
		function countManifestEntries() {
			try {
				const manifestPath = join(savedir, "persistent_res_list.json");
				const content = readFileSync(manifestPath, "utf-8");
				const entries = JSON.parse(content);
				return Object.keys(entries).length;
			} catch {
				return 0;
			}
		}

		// Snapshot the manifest before download starts
		initialManifestCount = countManifestEntries();

		child.stdout.on("data", (chunk) => {
			stdoutBuf += chunk.toString();
			// Parse total from "N files to download (M skipped)"
			if (totalFiles === 0) {
				const m = stdoutBuf.match(/(\d+) files to download/);
				if (m) {
					totalFiles = parseInt(m[1], 10);
					onProgress?.({ completed: 0, total: totalFiles, percent: 0 });

					// Start polling manifest for progress
					pollTimer = setInterval(() => {
						const currentCount = countManifestEntries();
						const completed = currentCount - initialManifestCount;
						if (completed > 0 && totalFiles > 0) {
							onProgress?.({
								completed: Math.min(completed, totalFiles),
								total: totalFiles,
								percent: (Math.min(completed, totalFiles) / totalFiles) * 100,
							});
						}
					}, 1000);
				}
			}
		});

		child.stderr.on("data", (chunk) => {
			stderrRaw += chunk.toString();
		});

		child.on("close", (code) => {
			if (pollTimer) clearInterval(pollTimer);

			if (code !== 0) {
				const clean = `${stdoutBuf}\n${stderrRaw}`
					.replace(ANSI_RE, "")
					.replace(/\r/g, "\n");
				const lines = clean
					.split("\n")
					.map((l) => l.trim())
					.filter(Boolean);
				const errOutput = lines.slice(-10).join("\n");
				reject(new Error(`Downloader exited with code ${code}\n${errOutput}`));
				return;
			}
			// Parse "Done: X downloaded, Y failed, Z bytes"
			const doneMatch = stdoutBuf.match(
				/Done:\s*(\d+)\s*downloaded,\s*(\d+)\s*failed,\s*(\d+)\s*bytes/,
			);
			resolve({
				downloaded: doneMatch ? parseInt(doneMatch[1], 10) : 0,
				failed: doneMatch ? parseInt(doneMatch[2], 10) : 0,
				totalBytes: doneMatch ? parseInt(doneMatch[3], 10) : 0,
			});
		});

		child.on("error", (err) => {
			if (pollTimer) clearInterval(pollTimer);
			reject(err);
		});
	});
}

/**
 * Spawn the unpacker binary and track progress.
 *
 * indicatif suppresses all stderr output when not connected to a TTY.
 * The unpacker prints "Exported N ..." lines to stdout as each phase completes.
 * We parse stdout for these lines to track progress.
 *
 * @param {object} opts
 * @param {string} opts.inputDir - Input directory with bundles
 * @param {string} opts.outputDir - Output directory for extracted assets
 * @param {number} [opts.jobs] - Parallel extraction threads
 * @param {(p: {completed: number, total: number, percent: number}) => void} [opts.onProgress]
 * @returns {Promise<{exported: number}>}
 */
function runUnpack({
	inputDir,
	outputDir,
	jobs = DEFAULT_THREADS,
	onProgress,
}) {
	return new Promise((resolve, reject) => {
		const child = spawn(
			UNPACKER_BIN,
			["extract", "-i", inputDir, "-o", outputDir, "-j", String(jobs)],
			{
				cwd: __dirname,
				stdio: ["ignore", "pipe", "pipe"],
			},
		);

		activeChildren.add(child);
		child.on("close", () => activeChildren.delete(child));

		const MAX_TAIL = 256 * 1024; // bounded rolling buffer for error diagnostics
		let stdoutTail = "";
		let stderrTail = "";
		let stdoutPartial = ""; // incomplete trailing line between chunks
		let stderrPartial = "";
		let totalExported = 0;
		let exportedSum = 0;

		const appendTail = (tail, s) => {
			const combined = tail + s;
			return combined.length > MAX_TAIL ? combined.slice(-MAX_TAIL) : combined;
		};

		child.stdout.on("data", (chunk) => {
			const text = chunk.toString();
			stdoutTail = appendTail(stdoutTail, text);

			stdoutPartial += text;
			const lines = stdoutPartial.split("\n");
			stdoutPartial = lines.pop() ?? "";

			let latestProgress = totalExported;
			for (const line of lines) {
				const mp = line.match(/progress:\s*(\d+)\s+assets/);
				if (mp) {
					const n = parseInt(mp[1], 10);
					if (n > latestProgress) latestProgress = n;
				}
				const me = line.match(/Exported\s+(\d+)\s+/);
				if (me) {
					const n = parseInt(me[1], 10);
					exportedSum += n;
					if (n > latestProgress) latestProgress = n;
				}
			}
			if (latestProgress > totalExported) {
				totalExported = latestProgress;
				onProgress?.({ completed: totalExported, total: 0, percent: -1 });
			}
		});

		child.stderr.on("data", (chunk) => {
			stderrTail = appendTail(stderrTail, chunk.toString());
			stderrPartial += chunk.toString();
			if (stderrPartial.length > MAX_TAIL) {
				stderrPartial = stderrPartial.slice(-MAX_TAIL);
			}
		});

		child.on("close", (code) => {
			if (code !== 0) {
				const clean = `${stdoutTail}\n${stderrTail}`
					.replace(ANSI_RE, "")
					.replace(/\r/g, "\n");
				const lines = clean
					.split("\n")
					.map((l) => l.trim())
					.filter(Boolean);
				const errOutput = lines.slice(-10).join("\n");
				reject(new Error(`Unpacker exited with code ${code}\n${errOutput}`));
				return;
			}
			// Process any final line without trailing newline
			if (stdoutPartial) {
				const me = stdoutPartial.match(/Exported\s+(\d+)\s+/);
				if (me) exportedSum += parseInt(me[1], 10);
			}
			resolve({ exported: exportedSum });
		});

		child.on("error", reject);
	});
}

// ─── Option 1: Setup ───────────────────────────────────────────────────────

async function runSetup() {
	console.log(chalk.bold("\n─── Prerequisite Checks ───\n"));

	const RUST_MIN = { major: 1, minor: 85, patch: 0 };
	const checks = [];

	// Git
	const gitOut = tryExec("git", ["--version"]);
	checks.push({
		name: "Git",
		found: !!gitOut,
		version: gitOut ?? null,
		install: {
			darwin: "xcode-select --install  (or)  brew install git",
			linux: "sudo apt install git  (or)  sudo dnf install git",
			win32: "Download from https://git-scm.com/download/win",
		},
	});

	// Rust compiler
	const rustcOut = tryExec("rustc", ["--version"]);
	const rustcVer = parseVersion(rustcOut);
	const rustcOk = rustcVer ? versionGte(rustcVer, RUST_MIN) : false;
	checks.push({
		name: "Rust (rustc)",
		found: !!rustcOut && rustcOk,
		version: rustcOut ?? null,
		detail: rustcOut && !rustcOk ? "Need >= 1.85.0 for edition 2024" : null,
		install: {
			darwin: "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
			linux: "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
			win32: "Download rustup-init.exe from https://rustup.rs",
		},
	});

	// Cargo
	const cargoOut = tryExec("cargo", ["--version"]);
	checks.push({
		name: "Cargo",
		found: !!cargoOut,
		version: cargoOut ?? null,
		install: {
			darwin: "Installed with Rust (rustup)",
			linux: "Installed with Rust (rustup)",
			win32: "Installed with Rust (rustup)",
		},
	});

	// C compiler
	let ccFound = false;
	let ccVersion = null;
	if (isMac) {
		const xcodeOut = tryExec("xcode-select", ["-p"]);
		ccFound = !!xcodeOut;
		ccVersion = xcodeOut ? `Xcode CLT: ${xcodeOut}` : null;
	} else if (isWindows) {
		// Check which Rust toolchain is active (gnu vs msvc)
		const rustcHost = tryExec("rustc", ["-vV"]);
		const isMsvcToolchain = rustcHost?.includes("x86_64-pc-windows-msvc");

		if (isMsvcToolchain) {
			// For MSVC toolchain, check if cl.exe exists (even if not in PATH)
			// Try common MSVC installation paths
			const clOut = tryExec("where", ["cl"]);
			if (clOut) {
				ccFound = true;
				ccVersion = clOut.split("\n")[0];
			} else {
				// Check if MSVC is installed even if not in PATH
				const vswhereOut = tryExec("C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere.exe",
					["-latest", "-requires", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-property", "installationPath"]);
				if (vswhereOut) {
					ccFound = true;
					ccVersion = "MSVC (found via vswhere, not in PATH)";
				}
			}
		} else {
			// For GNU toolchain, check for gcc
			const gccOut = tryExec("gcc", ["--version"]);
			ccFound = !!gccOut;
			ccVersion = gccOut ? gccOut.split("\n")[0] : null;
		}
	} else {
		const gccOut = tryExec("gcc", ["--version"]);
		ccFound = !!gccOut;
		ccVersion = gccOut ? gccOut.split("\n")[0] : null;
	}
	checks.push({
		name: "C Compiler",
		found: ccFound,
		version: ccVersion ?? null,
		install: {
			darwin: "xcode-select --install",
			linux: "sudo apt install build-essential  (or)  sudo dnf install gcc",
			win32: "Install Visual Studio Build Tools (C++ workload)",
		},
	});

	// Display results
	const missing = [];
	for (const c of checks) {
		const icon = c.found ? chalk.green("✓") : chalk.red("✗");
		const ver = c.version ? chalk.dim(` (${c.version})`) : "";
		const detail = c.detail ? chalk.yellow(` — ${c.detail}`) : "";
		console.log(`  ${icon} ${c.name}${ver}${detail}`);
		if (!c.found) missing.push(c);
	}
	console.log();

	if (missing.length > 0) {
		const instructions = missing
			.map((c) => {
				const cmd = c.install[process.platform] ?? c.install.linux;
				return `${chalk.bold(c.name)}\n  ${cmd}`;
			})
			.join("\n\n");

		console.log(
			boxen(instructions, {
				title: "Missing Prerequisites",
				titleAlignment: "center",
				padding: 1,
				margin: { top: 0, bottom: 1, left: 1, right: 1 },
				borderStyle: "round",
				borderColor: "yellow",
			}),
		);
		console.log(
			chalk.yellow("Install the missing tools above, then re-run this script."),
		);
		return;
	}

	const { proceed } = await inquirer.prompt([
		{
			type: "confirm",
			name: "proceed",
			message: "All prerequisites found. Proceed with building?",
			default: true,
		},
	]);
	if (!proceed) return;

	// Check OpenArknightsFBS
	const fbsDir = join(__dirname, "OpenArknightsFBS", "FBS");
	let hasFbs = false;
	try {
		const files = readdirSync(fbsDir);
		hasFbs = files.some((f) => f.endsWith(".fbs"));
	} catch {
		// directory doesn't exist
	}
	if (!hasFbs) {
		console.log(
			chalk.yellow(
				"\n⚠  OpenArknightsFBS/FBS/ has no .fbs files.\n" +
					"   This is only needed for the generate-fbs binary.\n" +
					"   If you need it, clone the OpenArknightsFBS repo into this directory.\n",
			),
		);
	}

	// Build crates
	for (const crate of ["downloader", "unpacker"]) {
		await buildCrate(crate);
	}

	// Copy binaries to /binaries
	mkdirSync(BINARIES_DIR, { recursive: true });
	const copySpinner = ora("Copying binaries to ./binaries/…").start();
	copyFileSync(DOWNLOADER_BUILD, DOWNLOADER_BIN);
	copyFileSync(UNPACKER_BUILD, UNPACKER_BIN);
	copySpinner.succeed("Binaries copied to ./binaries/");

	// Results summary
	const displayDownloader = `.${sep}binaries${sep}downloader${exe}`;
	const displayUnpacker = `.${sep}binaries${sep}unpacker${exe}`;

	const summary = [
		chalk.bold.green("Build complete!\n"),
		`${chalk.bold("Downloader:")} ${displayDownloader}`,
		`${chalk.bold("Unpacker:")}   ${displayUnpacker}`,
		"",
		chalk.dim("Quick start:"),
		`  ${displayDownloader} --server en download --all`,
		`  ${displayUnpacker} extract -i ./ArkAssets -o ./output`,
	].join("\n");

	console.log(
		boxen(summary, {
			padding: 1,
			margin: { top: 1, bottom: 0, left: 1, right: 1 },
			borderStyle: "round",
			borderColor: "green",
		}),
	);
}

async function buildCrate(name) {
	const crateDir = join(__dirname, name);
	const spinner = ora(`Building ${name} (release)…`).start();
	const startTime = Date.now();
	let compiledCount = 0;

	return new Promise((resolve, reject) => {
		const child = spawn("cargo", ["build", "--release"], {
			cwd: crateDir,
			stdio: ["ignore", "pipe", "pipe"],
		});

		let stderr = "";
		child.stderr.on("data", (data) => {
			stderr += data.toString();
			const lines = data.toString().trim().split("\n");
			for (const line of lines) {
				if (line.includes("Compiling")) compiledCount++;
			}
			const last = lines[lines.length - 1].trim();
			if (last) {
				spinner.text = `Building ${name}: ${last}${compiledCount > 0 ? chalk.dim(` (${compiledCount} crates)`) : ""}`;
			}
		});

		child.on("close", (code) => {
			const elapsed = ((Date.now() - startTime) / 1000).toFixed(1);
			if (code === 0) {
				const detail =
					compiledCount > 0 ? `${compiledCount} crates compiled` : "up to date";
				spinner.succeed(`${name} built successfully (${elapsed}s, ${detail})`);
				resolve();
			} else {
				spinner.fail(`${name} build failed (${elapsed}s)`);
				console.error(chalk.red(stderr.slice(-500)));
				reject(new Error(`${name} build failed with code ${code}`));
			}
		});

		child.on("error", reject);
	});
}

// ─── Option 2: Check & Update ──────────────────────────────────────────────

async function runUpdate() {
	if (!binariesExist()) {
		console.log(chalk.red("\nBinaries not found. Run Setup first.\n"));
		return;
	}

	// Prompt for configuration
	const { serverKey, savedir, outputDir, threads } = await inquirer.prompt([
		{
			type: "list",
			name: "serverKey",
			message: "Server region:",
			choices: Object.entries(SERVERS).map(([key, s]) => ({
				name: `${key} — ${s.label}`,
				value: key,
			})),
			default: "en",
		},
		{
			type: "input",
			name: "savedir",
			message: "Asset download directory:",
			default: "./ArkAssets",
		},
		{
			type: "input",
			name: "outputDir",
			message: "Extraction output directory:",
			default: "./output",
		},
		{
			type: "number",
			name: "threads",
			message: "Concurrent threads (download & unpack):",
			default: DEFAULT_THREADS,
		},
	]);

	// Check version
	const versionSpinner = ora("Checking server version…").start();
	let serverVer;
	try {
		serverVer = await fetchServerVersion(serverKey);
		versionSpinner.succeed(
			`Server: client=${serverVer.clientVersion}  resources=${serverVer.resVersion}`,
		);
	} catch (err) {
		versionSpinner.fail(`Failed to fetch version: ${err.message}`);
		return;
	}

	const storedVer = readStoredVersion(savedir);
	if (storedVer) {
		console.log(chalk.dim(`  Local version: ${storedVer}`));
	} else {
		console.log(chalk.dim("  No local version found (first run)"));
	}

	const assetsUpToDate = storedVer === serverVer.resVersion;
	const needsReextract =
		assetsUpToDate && (unpackerIsNewer(savedir) || outputMissingOrEmpty(outputDir));

	if (assetsUpToDate && !needsReextract) {
		console.log(
			boxen(chalk.green("Assets are up to date!"), {
				padding: 1,
				margin: { top: 1, bottom: 0, left: 1, right: 1 },
				borderStyle: "round",
				borderColor: "green",
			}),
		);
		return;
	}

	if (needsReextract) {
		const reason = outputMissingOrEmpty(outputDir)
			? "but the output directory is missing or empty."
			: "but the unpacker binary has been rebuilt since last extraction.";
		console.log(
			boxen(
				[
					chalk.yellow.bold("Re-extraction needed"),
					"",
					`Assets version ${chalk.green(storedVer)} is current,`,
					reason,
				].join("\n"),
				{
					padding: 1,
					margin: { top: 1, bottom: 0, left: 1, right: 1 },
					borderStyle: "round",
					borderColor: "yellow",
				},
			),
		);

		const { proceed } = await inquirer.prompt([
			{
				type: "confirm",
				name: "proceed",
				message: "Re-extract with updated unpacker?",
				default: true,
			},
		]);
		if (!proceed) return;
	} else {
		// New assets available from server
		console.log(
			boxen(
				[
					chalk.yellow.bold("Update Available"),
					"",
					`Current: ${storedVer ?? chalk.dim("(none)")}`,
					`Server:  ${chalk.green(serverVer.resVersion)}`,
				].join("\n"),
				{
					padding: 1,
					margin: { top: 1, bottom: 0, left: 1, right: 1 },
					borderStyle: "round",
					borderColor: "yellow",
				},
			),
		);

		const { proceed } = await inquirer.prompt([
			{
				type: "confirm",
				name: "proceed",
				message: "Download and extract updates?",
				default: true,
			},
		]);
		if (!proceed) return;
	}

	// Download phase (skip if only re-extracting)
	if (!needsReextract) {
		console.log();
		const dlBar = createProgressBar("Downloading");
		dlBar.startSpinner("Fetching asset manifest…");
		try {
			const dlStats = await runDownload({
				serverKey,
				savedir,
				threads,
				onProgress: ({ completed, total }) => dlBar.render(completed, total),
			});
			dlBar.succeed(
				`Download complete: ${dlStats.downloaded} files, ${dlStats.failed} failed, ${formatBytes(dlStats.totalBytes)}`,
			);
		} catch (err) {
			dlBar.fail(`Download failed: ${err.message}`);
			return;
		}
	}

	// Unpack phase
	const upSpinner = ora("Extracting assets…").start();
	try {
		const upStats = await runUnpack({
			inputDir: savedir,
			outputDir,
			jobs: threads,
			onProgress: ({ completed }) => {
				upSpinner.text = `Extracting assets… (${completed.toLocaleString()} exported so far)`;
			},
		});
		upSpinner.succeed(
			`Extraction complete: ${upStats.exported.toLocaleString()} assets exported`,
		);
	} catch (err) {
		upSpinner.fail(`Extraction failed: ${err.message}`);
		return;
	}

	// Save version & extraction timestamp
	if (!assetsUpToDate) {
		writeStoredVersion(savedir, serverVer.resVersion);
	}
	touchExtractStamp(savedir);

	const msg = needsReextract
		? `Re-extracted with updated unpacker (${serverVer.resVersion})`
		: `Updated to ${serverVer.resVersion}`;
	console.log(
		boxen(chalk.green.bold(msg), {
			padding: 1,
			margin: { top: 1, bottom: 0, left: 1, right: 1 },
			borderStyle: "round",
			borderColor: "green",
		}),
	);
}

// ─── Option 3: WebSocket Server ─────────────────────────────────────────────

async function runWebSocketServer({ nonInteractive = false, cliArgs = {} } = {}) {
	if (!binariesExist()) {
		console.log(chalk.red("\nBinaries not found. Run Setup first.\n"));
		return;
	}

	const defaults = {
		serverKey: cliArgs.server ?? process.env.WS_SERVER ?? "en",
		savedir: cliArgs.savedir ?? process.env.WS_SAVEDIR ?? "./ArkAssets",
		outputDir: cliArgs.output ?? process.env.WS_OUTPUT ?? "./output",
		threads: Number(cliArgs.threads ?? process.env.WS_THREADS ?? DEFAULT_THREADS),
		port: Number(cliArgs.port ?? process.env.WS_PORT ?? 9160),
		intervalMin: Number(cliArgs.interval ?? process.env.WS_INTERVAL ?? 30),
	};

	const config = nonInteractive
		? defaults
		: await inquirer.prompt([
			{
				type: "list",
				name: "serverKey",
				message: "Server region:",
				choices: Object.entries(SERVERS).map(([key, s]) => ({
					name: `${key} — ${s.label}`,
					value: key,
				})),
				default: defaults.serverKey,
			},
			{ type: "input", name: "savedir", message: "Asset download directory:", default: defaults.savedir },
			{ type: "input", name: "outputDir", message: "Extraction output directory:", default: defaults.outputDir },
			{ type: "number", name: "threads", message: "Concurrent threads (download & unpack):", default: defaults.threads },
			{ type: "number", name: "port", message: "WebSocket port:", default: defaults.port },
			{ type: "number", name: "intervalMin", message: "Check interval (minutes):", default: defaults.intervalMin },
		]);

	if (!SERVERS[config.serverKey]) {
		console.log(chalk.red(`\nUnknown server: ${config.serverKey}. Valid: ${Object.keys(SERVERS).join(", ")}\n`));
		return;
	}

	const intervalMs = config.intervalMin * 60 * 1000;

	// State
	let currentState = "idle";
	let updating = false;
	let currentVersion = readStoredVersion(config.savedir);

	// WebSocket server
	const wss = new WebSocketServer({ port: config.port });

	// Listen for CTRL+C directly on raw stdin. Libraries like signal-exit
	// (used by inquirer/ora) patch process.emit and install their own SIGINT
	// handlers, which can swallow the signal. Reading raw stdin for 0x03 is
	// the most reliable way to detect CTRL+C regardless of what other
	// libraries do.
	if (process.stdin.isTTY && process.stdin.setRawMode) {
		process.stdin.setRawMode(true);
		process.stdin.resume();
		process.stdin.on("data", (key) => {
			// 0x03 = CTRL+C
			if (key[0] === 0x03) {
				for (const child of activeChildren) {
					try {
						child.kill("SIGKILL");
					} catch {}
				}
				wss.close();
				console.log(chalk.dim("\nInterrupted."));
				process.exit(1);
			}
		});
	}
	const clients = new Set();

	function broadcast(msg) {
		const data = JSON.stringify(msg);
		for (const ws of clients) {
			if (ws.readyState === ws.OPEN) ws.send(data);
		}
	}

	function sendTo(ws, msg) {
		if (ws.readyState === ws.OPEN) ws.send(JSON.stringify(msg));
	}

	function statusMessage() {
		return {
			type: "status",
			state: currentState,
			version: { current: currentVersion ?? null },
		};
	}

	// List resources in output directory
	async function listResources(ws) {
		const dir = config.outputDir;
		if (!existsSync(dir)) {
			sendTo(ws, { type: "resource_list", files: [], totalSize: 0 });
			return;
		}

		const files = [];
		let totalSize = 0;

		try {
			const entries = await readdir(dir, { withFileTypes: true });
			for (const entry of entries) {
				const fullPath = join(dir, entry.name);
				const st = await stat(fullPath);

				if (entry.isDirectory()) {
					// Summarize directory: count files and total size one level deep
					let dirSize = 0;
					let fileCount = 0;
					try {
						const subEntries = await readdir(fullPath, { withFileTypes: true });
						for (const sub of subEntries) {
							if (sub.isFile()) {
								const subSt = await stat(join(fullPath, sub.name));
								dirSize += subSt.size;
								fileCount++;
							}
						}
					} catch {
						// skip unreadable subdirs
					}
					files.push({
						name: entry.name,
						path: entry.name,
						size: dirSize,
						fileCount,
						modified: st.mtime.toISOString(),
						created: st.birthtime.toISOString(),
						type: "directory",
					});
					totalSize += dirSize;
				} else {
					files.push({
						name: entry.name,
						path: entry.name,
						size: st.size,
						modified: st.mtime.toISOString(),
						created: st.birthtime.toISOString(),
						type: "file",
					});
					totalSize += st.size;
				}
			}
		} catch (err) {
			sendTo(ws, {
				type: "error",
				message: `Failed to list resources: ${err.message}`,
			});
			return;
		}

		sendTo(ws, {
			type: "resource_list",
			files,
			totalSize,
			totalSizeFormatted: formatBytes(totalSize),
		});
	}

	// Perform download + unpack cycle
	async function performUpdate() {
		if (updating) return;
		updating = true;

		try {
			// Download phase
			currentState = "downloading";
			broadcast(statusMessage());
			console.log(chalk.blue(`[${new Date().toLocaleTimeString()}] Downloading assets...`));

			const dlStats = await runDownload({
				serverKey: config.serverKey,
				savedir: config.savedir,
				threads: config.threads,
				onProgress: (p) => broadcast({ type: "download_progress", ...p }),
				onStatus: (msg) =>
					broadcast({ type: "status", state: "downloading", message: msg }),
			});

			console.log(chalk.blue(`[${new Date().toLocaleTimeString()}] Download complete: ${dlStats.downloaded} files, ${dlStats.failed} failed, ${formatBytes(dlStats.totalBytes)}`));
			broadcast({
				type: "download_complete",
				downloaded: dlStats.downloaded,
				failed: dlStats.failed,
				totalBytes: dlStats.totalBytes,
				totalBytesFormatted: formatBytes(dlStats.totalBytes),
			});

			// Unpack phase
			currentState = "unpacking";
			broadcast(statusMessage());
			console.log(chalk.blue(`[${new Date().toLocaleTimeString()}] Unpacking assets...`));

			const upStats = await runUnpack({
				inputDir: config.savedir,
				outputDir: config.outputDir,
				jobs: config.threads,
				onProgress: (p) => broadcast({ type: "unpack_progress", ...p }),
			});

			// Update stored version and extraction timestamp
			const serverVer = await fetchServerVersion(config.serverKey);
			writeStoredVersion(config.savedir, serverVer.resVersion);
			touchExtractStamp(config.savedir);
			currentVersion = serverVer.resVersion;

			currentState = "idle";
			console.log(chalk.green(`[${new Date().toLocaleTimeString()}] Update complete: v${currentVersion}, ${dlStats.downloaded} downloaded, ${upStats.exported} exported`));
			broadcast({
				type: "update_complete",
				version: currentVersion,
				downloaded: dlStats.downloaded,
				failed: dlStats.failed,
				exported: upStats.exported,
			});
			broadcast(statusMessage());
		} catch (err) {
			currentState = "idle";
			console.log(chalk.red(`[${new Date().toLocaleTimeString()}] Update failed: ${err.message}`));
			broadcast({ type: "error", message: `Update failed: ${err.message}` });
			broadcast(statusMessage());
		} finally {
			updating = false;
		}
	}

	// Check for updates and trigger download if needed
	async function checkAndUpdate() {
		if (updating) return;

		try {
			currentState = "checking";
			broadcast(statusMessage());
			console.log(chalk.dim(`[${new Date().toLocaleTimeString()}] Checking for updates...`));

			const serverVer = await fetchServerVersion(config.serverKey);
			const storedVer = readStoredVersion(config.savedir);

			currentState = "idle";

			const needsReextract =
				unpackerIsNewer(config.savedir) || outputMissingOrEmpty(config.outputDir);
			if (storedVer === serverVer.resVersion && !needsReextract) {
				console.log(chalk.dim(`[${new Date().toLocaleTimeString()}] Up to date (${storedVer})`));
				broadcast(statusMessage());
				return;
			}

			if (needsReextract) {
				console.log(chalk.yellow(`[${new Date().toLocaleTimeString()}] Re-extraction needed (assets current but output stale)`));
			} else {
				console.log(chalk.yellow(`[${new Date().toLocaleTimeString()}] Update available: ${storedVer ?? "(none)"} → ${serverVer.resVersion}`));
			}

			// Update available (new assets or unpacker rebuild)
			broadcast({
				type: "update_available",
				currentVersion: storedVer ?? null,
				newVersion: serverVer.resVersion,
				clientVersion: serverVer.clientVersion,
			});

			await performUpdate();
		} catch (err) {
			currentState = "idle";
			console.log(chalk.red(`[${new Date().toLocaleTimeString()}] Version check failed: ${err.message}`));
			broadcast({
				type: "error",
				message: `Version check failed: ${err.message}`,
			});
			broadcast(statusMessage());
		}
	}

	// Handle client connections
	wss.on("connection", (ws) => {
		console.log(chalk.dim(`[${new Date().toLocaleTimeString()}] Client connected (${clients.size + 1} total)`));
		clients.add(ws);
		sendTo(ws, statusMessage());

		ws.on("message", async (raw) => {
			let msg;
			try {
				msg = JSON.parse(raw.toString());
			} catch {
				sendTo(ws, { type: "error", message: "Invalid JSON" });
				return;
			}

			switch (msg.type) {
				case "force_update":
					if (updating) {
						sendTo(ws, {
							type: "error",
							message: "Update already in progress",
						});
					} else {
						performUpdate();
					}
					break;

				case "list_resources":
					await listResources(ws);
					break;

				default:
					sendTo(ws, {
						type: "error",
						message: `Unknown command: ${msg.type}`,
					});
			}
		});

		ws.on("close", () => {
			clients.delete(ws);
			console.log(chalk.dim(`[${new Date().toLocaleTimeString()}] Client disconnected (${clients.size} remaining)`));
		});
		ws.on("error", () => clients.delete(ws));
	});

	// Start periodic checking
	setInterval(checkAndUpdate, intervalMs);

	console.log(
		boxen(
			[
				chalk.bold.green("WebSocket Server Running"),
				"",
				`${chalk.bold("Address:")}  ws://localhost:${config.port}`,
				`${chalk.bold("Server:")}   ${config.serverKey} — ${SERVERS[config.serverKey].label}`,
				`${chalk.bold("Savedir:")}  ${config.savedir}`,
				`${chalk.bold("Output:")}   ${config.outputDir}`,
				`${chalk.bold("Threads:")}  ${config.threads}`,
				`${chalk.bold("Interval:")} ${config.intervalMin} minutes`,
				"",
				chalk.dim("Press Ctrl+C to stop"),
			].join("\n"),
			{
				padding: 1,
				margin: { top: 1, bottom: 0, left: 1, right: 1 },
				borderStyle: "round",
				borderColor: "green",
			},
		),
	);

	// Initial check
	await checkAndUpdate();
}

// ─── Global SIGINT ──────────────────────────────────────────────────────────
// Track active child processes so CTRL+C can kill them and exit cleanly
const activeChildren = new Set();

// NOTE: Do NOT use readline.createInterface here — it puts stdin into raw mode
// which intercepts CTRL+C (0x03) and prevents the process-level SIGINT from
// firing. Instead, rely on process.on("SIGINT") which works when stdin is in
// normal (cooked) mode.
process.on("SIGINT", () => {
	for (const child of activeChildren) {
		try {
			child.kill("SIGKILL");
		} catch {}
	}
	console.log(chalk.dim("\nInterrupted."));
	process.exit(1);
});

// ─── Main Menu ──────────────────────────────────────────────────────────────

// ─── CLI argv parsing (non-interactive entry) ──────────────────────────────
// Usage: node run.mjs ws [--server en] [--savedir ./ArkAssets] [--output ./output]
//                        [--threads N] [--port 9160] [--interval 30]
const argv = process.argv.slice(2);
const cliAction = argv[0] && !argv[0].startsWith("--") ? argv[0] : null;
const cliArgs = {};
for (let i = cliAction ? 1 : 0; i < argv.length; i++) {
	const a = argv[i];
	if (!a.startsWith("--")) continue;
	const key = a.slice(2);
	const next = argv[i + 1];
	if (next !== undefined && !next.startsWith("--")) {
		cliArgs[key] = next;
		i++;
	} else {
		cliArgs[key] = true;
	}
}

if (cliAction) {
	const platformLabel = isMac ? "macOS" : isWindows ? "Windows" : "Linux";
	console.log(chalk.dim(`Platform: ${platformLabel} (${process.arch})`));
	switch (cliAction) {
		case "setup":
			await runSetup();
			break;
		case "update":
			await runUpdate();
			break;
		case "ws":
			await runWebSocketServer({ nonInteractive: true, cliArgs });
			break;
		default:
			console.log(chalk.red(`Unknown command: ${cliAction}`));
			console.log("Valid commands: setup, update, ws");
			process.exit(1);
	}
} else {
	console.log(
		boxen(chalk.bold.cyan("Arknights Asset Pipeline"), {
			padding: 1,
			margin: 1,
			borderStyle: "round",
			borderColor: "cyan",
		}),
	);

	const platformLabel = isMac ? "macOS" : isWindows ? "Windows" : "Linux";
	console.log(chalk.dim(`Platform: ${platformLabel} (${process.arch})\n`));

	const hasBinaries = binariesExist();
	if (!hasBinaries) {
		console.log(
			chalk.yellow("Binaries not found. Run Setup first to build them.\n"),
		);
	}

	const choices = [
		{ name: "Setup (build binaries)", value: "setup" },
		...(hasBinaries
			? [
					{ name: "Check for Updates & Update", value: "update" },
					{ name: "WebSocket Server (continuous updates)", value: "ws" },
				]
			: []),
	];

	const { action } = await inquirer.prompt([
		{
			type: "list",
			name: "action",
			message: "What would you like to do?",
			choices,
		},
	]);

	switch (action) {
		case "setup":
			await runSetup();
			break;
		case "update":
			await runUpdate();
			break;
		case "ws":
			await runWebSocketServer();
			break;
	}
}
