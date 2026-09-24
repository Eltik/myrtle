/**
 * The reader's CHROME: the two frosted pills over the stage and the scrubber
 * above them.
 *
 * The pills are the client's own grouping, a left cluster and a right cluster
 * over the scene, and like the client's they STAY: the idle fade is opt-in
 * (`autoHideToolbar`) and off by default.
 *
 * THREE ways the chrome goes, and they are not the same thing:
 *
 * 1. Theater mode, the client's eye: the text box, both pills and the scrubber
 *    go and the scene stays. Any tap or key hands them back.
 * 2. "Hide toolbar", the chevron at the end of a pill and `T`: the pills and
 *    the scrubber go, the TEXT BOX STAYS, and it is remembered across stories
 *    because it was asked for on purpose. A handle at the top edge brings them
 *    back.
 * 3. The auto-hide, off by default: the pills fade after the reader's own idle
 *    time and a pointer MOVE brings them back.
 *
 * What does NOT bring them back is reading. Space, Enter, the arrows, a tap on
 * the stage and auto-play are how a story is read, and a bar that reappears on
 * every Space is the auto-hide doing the opposite of what it is for; only
 * pointer movement, a touch in the top band, `T` and a dialog reveal.
 *
 * Everything here is pure. The reader owns the timer and the booleans; this
 * file owns the RULES, so the state machine is testable without a DOM.
 */

import { clamp } from "#/lib/story/num";

/** The default idle wait, in milliseconds. The reader's own setting overrides it, 1 s to 15 s. */
export const CHROME_IDLE_MS = 2500;

/** The idle setting's range and step, in seconds. */
export const CHROME_IDLE_MIN_SEC = 1;
export const CHROME_IDLE_MAX_SEC = 15;
export const CHROME_IDLE_STEP_SEC = 0.5;

/**
 * The band at the top of the stage a TOUCH reveals the chrome from. A tap
 * lower down is a page turn, so it must not also bring the bar back; 64 px is
 * the pills' own 8 px offset plus a 44 px target plus room to miss.
 */
export const CHROME_REVEAL_BAND_PX = 64;

/** Fade out, in milliseconds. Slower than the way in, so the bar does not blink. */
export const CHROME_FADE_OUT_MS = 200;

/** Fade in, in milliseconds. */
export const CHROME_FADE_IN_MS = 150;

export interface ChromeInputs {
    /** Theater mode, the eye button: nothing but the scene. It outranks everything. */
    theater: boolean;
    /** "Hide toolbar": the pills and the scrubber, and nothing else. Persisted. */
    collapsed: boolean;
    /** The auto-hide is OFF: the pills stay. The kill switch, and the default. */
    alwaysShow: boolean;
    /** A backlog, chapter or settings dialog is open. */
    dialogOpen: boolean;
    /** The pointer is over a pill or the scrubber. Keeps a FADING bar up, never a collapsed one. */
    pointerOverChrome: boolean;
    /** The pointer is on the reveal handle: a peek at a collapsed toolbar, and the only thing that outranks it. */
    pointerOverHandle: boolean;
    /** The reader's phase. Only "reading" ever hides the chrome. */
    phase: string;
    /** The idle wait has passed with no pointer movement. */
    idle: boolean;
}

/**
 * Whether the chrome is on screen. The order is the rule: theater mode first,
 * because the eye hides the pills whatever else is set; then the reveal
 * handle, the one thing that peeks at a collapsed toolbar; then the deliberate
 * "Hide toolbar", which outranks a dialog and the pointer on a pill alike,
 * because a chevron that does nothing until the pointer leaves it is a chevron
 * that looks broken; then the kill switch, the three states that never
 * auto-hide (a dialog, the pointer on the bar, and the title, resume and end
 * cards), and only then the idle timer.
 */
export function chromeShown(o: ChromeInputs): boolean {
    if (o.theater) return false;
    if (o.pointerOverHandle) return true;
    if (o.collapsed) return false;
    if (o.alwaysShow) return true;
    if (o.dialogOpen || o.pointerOverChrome || o.phase !== "reading") return true;
    return !o.idle;
}

/**
 * How long the PEEK survives the pointer leaving the handle and the pills
 * alike, in milliseconds.
 *
 * Without it the peek was unusable: the handle reveals the toolbar, and the
 * pointer then has to cross the gap between the handle and whichever pill it
 * is aiming for. The handle is 64 px wide and centred, the pills sit at the
 * two ends of an `inset-x-2` row, so at 1440 the nearest pill edge is roughly
 * 640 px away and every one of those pixels is STAGE. `pointerleave` on the
 * handle fired the instant the pointer started moving and the bar went with
 * it. The peek is now held over the UNION of the handle and the pills, and the
 * grace covers the stage between them: 300 ms is a comfortable crossing at any
 * speed a pointer actually travels and short enough that a pointer that left
 * for good does not leave the bar hanging.
 */
export const PEEK_GRACE_MS = 300;

/** What moves the peek. `enter` is the handle or either pill; `expire` is the grace running out; `open` is the toolbar coming back for real. */
export type PeekEvent = "enter" | "expire" | "open";

/**
 * The peek after one event. Leaving is deliberately NOT an event here: a leave
 * starts the grace timer, and only its `expire` drops the peek, which is the
 * whole point of the union rule. Re-entering anything in the union before the
 * timer fires cancels it and the peek never blinks.
 */
export function nextPeek(e: PeekEvent): boolean {
    return e === "enter";
}

/** Whether the top-edge reveal handle is on screen: only a collapsed toolbar has one. */
export function revealHandleShown(o: Pick<ChromeInputs, "theater" | "collapsed">): boolean {
    return o.collapsed && !o.theater;
}

/**
 * What the reader just did. `advance` covers Space, Enter, the arrows, a tap
 * on the stage and auto-play alike, because they are all the same act: reading.
 */
export type ReaderInput = "pointermove" | "pointerdown" | "advance" | "key" | "pin" | "dialog";

/**
 * Whether this input brings a faded toolbar back. Reading never does, which is
 * the whole rule; a pointer DOWN does only inside the top band, so a tap that
 * turns the page does not also raise the bar on a phone.
 */
export function revealsChrome(input: ReaderInput, pointerY = Number.POSITIVE_INFINITY, band = CHROME_REVEAL_BAND_PX): boolean {
    switch (input) {
        case "pointermove":
        case "pin":
        case "dialog":
            return true;
        case "pointerdown":
            return pointerY <= band;
        case "advance":
        case "key":
            return false;
    }
}

/** The idle wait in milliseconds, from the reader's own setting, clamped to its range. */
export function chromeIdleMs(idleSec: number): number {
    if (!Number.isFinite(idleSec)) return CHROME_IDLE_MS;
    return clamp(idleSec, CHROME_IDLE_MIN_SEC, CHROME_IDLE_MAX_SEC) * 1000;
}

/**
 * Where the scrub tooltip's left edge goes, in pixels from the stage's left
 * edge, so the tip follows the pointer but never leaves the stage. The tip is
 * CENTRED on the pointer where there is room and slides against the nearer
 * edge where there is not; a tip wider than the stage pins to the left pad
 * rather than resolving to a negative left.
 */
export function clampTooltipLeft(pointerX: number, tipWidth: number, stageWidth: number, pad = 8): number {
    const want = pointerX - tipWidth / 2;
    const max = stageWidth - tipWidth - pad;
    if (!(max > pad)) return pad;
    return clamp(want, pad, max);
}

/**
 * What reaches theater mode. `toggle` is the eye button and `Esc`; `activity`
 * is any tap, click or key, which is how the client gives the UI back.
 */
export type TheaterEvent = "toggle" | "activity";

/** Theater mode after an event. Activity only ever RESTORES, it never hides. */
export function nextTheater(on: boolean, e: TheaterEvent): boolean {
    return e === "toggle" ? !on : false;
}

/**
 * Whether the reader SWALLOWS this event to give the UI back instead of acting
 * on it. A tap that restores the chrome must not also advance the line, which
 * is the client's behaviour and the reason this is a rule and not an `if` at
 * the call site.
 */
export function theaterConsumes(on: boolean, e: TheaterEvent): boolean {
    return on && e === "activity";
}
