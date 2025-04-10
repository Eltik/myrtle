import { ProgressBar } from "./progress-bar";

// Multi-progress bar manager
export class MultiProgressBar {
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
