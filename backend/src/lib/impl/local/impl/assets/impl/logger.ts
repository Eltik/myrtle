import { existsSync, mkdirSync, appendFileSync } from "node:fs";
import { MultiProgressBar } from "./multi-progress-bar";
import { LogLevel } from "../../../../../../types/impl/lib/impl/local/impl/assets";

// Global progress bar manager that can be accessed by the Logger
let globalProgressManager: MultiProgressBar | null = null;

export class Logger {
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
