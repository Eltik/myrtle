/**
 * SKIP, the client's own fast-forward. The button sets the playback ratio to
 * ZERO, which is `AVGController.animateRatio = 0` and the reason skipping in
 * the game is instant rather than merely fast: every scene duration is
 * multiplied by it, so each step resolves in the frame it starts.
 *
 * It STOPS by itself at the three places the client stops: a decision, the end
 * of the story, and a click on the stage. The first press of a session asks
 * once, as the client asks, and every press after that in the same session
 * goes straight through.
 *
 * Pure, because the reader's timers and the engine are not: the phases here
 * are testable without a DOM.
 */

/** `off` is the resting state, `confirming` is the one-line dialog, `on` is skipping. */
export type SkipPhase = "off" | "confirming" | "on";

export interface SkipState {
    phase: SkipPhase;
    /** The session has already answered the confirm once. */
    confirmed: boolean;
}

export const SKIP_OFF: SkipState = { phase: "off", confirmed: false };

/**
 * What moves it. `press` is the button and `S`; `confirm` and `cancel` are the
 * dialog's two answers; `decision`, `boundary` (the end card, or a new story)
 * and `click` are the three stops.
 */
export type SkipEvent = "press" | "confirm" | "cancel" | "decision" | "boundary" | "click";

/**
 * The next state. A press while skipping STOPS, which is what makes the button
 * a toggle and matches the client, where tapping anything ends the skip.
 */
export function nextSkip(s: SkipState, e: SkipEvent): SkipState {
    switch (e) {
        case "press":
            if (s.phase === "on" || s.phase === "confirming") return { ...s, phase: "off" };
            return { ...s, phase: s.confirmed ? "on" : "confirming" };
        case "confirm":
            return { phase: "on", confirmed: true };
        case "cancel":
            return { ...s, phase: "off" };
        case "decision":
        case "boundary":
        case "click":
            // Only a RUNNING skip is stopped by these. A click while the
            // confirm is open belongs to the dialog, not to the stage.
            return s.phase === "on" ? { ...s, phase: "off" } : s;
    }
}

/** The ratio the engine runs at: zero while skipping, the setting otherwise. */
export function skipRatio(s: SkipState, settingRatio: number): number {
    return s.phase === "on" ? 0 : settingRatio;
}

/**
 * How long the reader waits between two skipped halts. Not zero: the advance
 * is a React state update per halt, and a zero-delay loop starves the click
 * that is supposed to stop it. 16 ms is one frame at 60 Hz, so a 735-halt
 * script skips in about 12 s and every frame in it can still be clicked.
 */
export const SKIP_STEP_MS = 16;

/**
 * Whether the reader should schedule another advance right now. A CUTSCENE
 * counts as a line here: a running skip that stopped at a video would sit
 * through a clip the viewer just asked to skip past.
 */
export function skipAdvances(s: SkipState, phase: string, haltKind: string | undefined): boolean {
    return s.phase === "on" && phase === "reading" && (haltKind === "line" || haltKind === "video");
}
