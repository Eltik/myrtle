import type { MaintenanceWindow } from "#/types/generated/MaintenanceWindow";

/**
 * What the countdown should say right now. Recomputed every tick on the client, so a window
 * moves from `upcoming` to `ongoing` at its start without waiting for the next poll.
 *
 * - `upcoming`: counts down to the start.
 * - `ongoing`: counts down to the planned end.
 * - `extended`: Yostar moved the end later; counts down to the new end.
 * - `overrun`: the end has passed and the game is still down. Nothing to count down to.
 * - `openEnded`: down, and no notice gives an end.
 */
export type MaintenanceStage = "upcoming" | "ongoing" | "extended" | "overrun" | "openEnded";

export interface IMaintenanceView {
    stage: MaintenanceStage;
    /** Seconds to count down, or `null` when there is nothing to count down to. */
    secondsLeft: number | null;
}

export function maintenanceView(w: MaintenanceWindow, nowSec: number): IMaintenanceView {
    if (nowSec < w.starts_at) return { stage: "upcoming", secondsLeft: w.starts_at - nowSec };
    if (w.ends_at === null) return { stage: "openEnded", secondsLeft: null };
    if (nowSec >= w.ends_at) return { stage: "overrun", secondsLeft: null };
    return { stage: w.phase === "extended" ? "extended" : "ongoing", secondsLeft: w.ends_at - nowSec };
}

/** Every stage but `upcoming` means the game is down now: a sync would be refused. */
export function isGameDown(stage: MaintenanceStage): boolean {
    return stage !== "upcoming";
}

/**
 * A short span: `3d 4h`, `2h 14m`, `14m`, `45s`. Two units at most, the larger first; minutes
 * round up so a countdown never reads `0m` while time is left.
 */
export function formatSpan(seconds: number): string {
    const s = Math.max(0, Math.floor(seconds));
    if (s < 60) return `${s}s`;
    const totalMin = Math.ceil(s / 60);
    const days = Math.floor(totalMin / 1440);
    const hours = Math.floor((totalMin % 1440) / 60);
    const minutes = totalMin % 60;
    if (days > 0) return hours > 0 ? `${days}d ${hours}h` : `${days}d`;
    if (hours > 0) return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`;
    return `${minutes}m`;
}

/**
 * Whether a visitor should see a window for `server`. Signed out (`undefined`/`null`) sees it:
 * the notices are Yostar's global ones. A player on another server does not.
 * Case-blind: a profile carries `servers.code` (`EN`), a window its lowercase id (`en`).
 */
export function maintenanceAppliesTo(windowServer: string, userServer: string | null | undefined): boolean {
    if (!userServer) return true;
    return userServer.toLowerCase() === windowServer.toLowerCase();
}
