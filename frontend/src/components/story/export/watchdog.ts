/**
 * A STALL DETECTOR. `kick()` on every sign of life; if none comes for `ms`,
 * `onStall` runs once. It exists because a PDF layout that stops making
 * progress used to leave the sheet on "Laying out pages" with no end, and a
 * silent export must fail with words instead.
 */
export interface Watchdog {
    kick: () => void;
    stop: () => void;
}

export const STALL_MS = 60_000;

export function createWatchdog(ms: number, onStall: () => void): Watchdog {
    let timer: ReturnType<typeof setTimeout> | null = null;
    let stopped = false;
    const arm = () => {
        if (timer !== null) clearTimeout(timer);
        timer = setTimeout(() => {
            timer = null;
            if (stopped) return;
            stopped = true;
            onStall();
        }, ms);
    };
    arm();
    return {
        kick: () => {
            if (!stopped) arm();
        },
        stop: () => {
            stopped = true;
            if (timer !== null) clearTimeout(timer);
            timer = null;
        },
    };
}

/** No progress for `seconds` while working on `story` (empty before the first story). */
export class ExportStallError extends Error {
    constructor(
        readonly story: string,
        readonly seconds: number,
    ) {
        super(`No progress for ${seconds} s${story ? ` while laying out ${story}` : ""}`);
        this.name = "ExportStallError";
    }
}
