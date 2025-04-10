// Simple progress bar class
export class ProgressBar {
    private width: number;
    public total: number;
    public current: number = 0;
    private description: string;
    private lastRender: string = "";
    private fixed: boolean;

    constructor(total: number, description: string = "", width: number = 40, fixed: boolean = false) {
        this.total = total;
        this.description = description;
        this.width = width;
        this.fixed = fixed;
    }

    update(current: number, description?: string): void {
        this.current = current;
        if (description) {
            this.description = description;
        }
        this.render();
    }

    increment(value: number = 1, description?: string): void {
        this.update(this.current + value, description);
    }

    render(): void {
        const percent = Math.min(Math.floor((this.current / this.total) * 100), 100);
        const filledWidth = Math.floor((this.current / this.total) * this.width);
        const bar = "█".repeat(filledWidth) + "-".repeat(this.width - filledWidth);

        const renderText = `${this.description.padEnd(30)} |${bar}| ${percent}% (${this.current}/${this.total})`;

        // Only redraw if the output has changed
        if (renderText !== this.lastRender) {
            if (this.fixed) {
                // Just print a new line with the updated progress
                console.log(renderText);
            } else {
                // Simply log the updated progress
                console.log(renderText);
            }
            this.lastRender = renderText;
        }
    }
}
