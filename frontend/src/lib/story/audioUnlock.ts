/**
 * THE GESTURE GATE: the one thing standing between a built AudioContext and a
 * context that actually runs.
 *
 * Real Chrome refuses to start an AudioContext that was created outside a user
 * gesture, and it refuses QUIETLY: `resume()` returns a promise that neither
 * resolves nor rejects, `state` stays "suspended" and `currentTime` is frozen
 * at 0, so every source the reader schedules lands on a dead clock while the
 * instrument still reports `playing: true` with a join. Measured 2026-09-22 on
 * `?halt=43` in the user's own Chrome: resume() still pending after 800 ms,
 * `currentTime` 0.000 across a 1.0 s wall-clock window, RMS 0.000 at `musicBus`
 * over 2.0 s, and `musicBus` left at gain 1 instead of the 0.4 the settings
 * asked for, because the `setTargetAtTime` was scheduled on that frozen clock.
 * Headless Chromium runs with the autoplay policy disabled, which is why eleven
 * green browser runs never saw it.
 *
 * `arm()` on the audio module alone cannot fix it, because only the handlers
 * that call `arm()` get a second chance: a reader whose next interaction is the
 * Auto button, the volume slider or the auto-play timer never calls it again
 * and stays silent for the session. This gate listens on the FIRST interaction
 * ANYWHERE on the page and takes itself off once the context runs.
 *
 * It lives apart from `audio.ts` because it shares nothing with the bus and
 * voice scheduling there: it holds one boolean, it reads the context through a
 * getter, and it says so by calling `onRunning`.
 */

const UNLOCK_EVENTS = ["pointerdown", "mousedown", "touchend", "keydown"] as const;

export interface IUnlockGate {
    /**
     * Bind the listeners and ask for a resume. The listeners go on BEFORE the
     * resume, because a gesture-blocked resume leaves the state "suspended"
     * behind a promise that never settles: there is no result to branch on.
     */
    arm(): void;
    /** The context reached "running": take the listeners off and tell the owner. */
    settle(): void;
    /** The context went back to "suspended": listen again. */
    bind(): void;
    /** Teardown, listeners off, nothing else. */
    dispose(): void;
}

/**
 * @param context reads the live context, or null once it is gone.
 * @param onRunning called exactly when a resume lands and the context runs, so
 *   the owner can re-apply the bus ramps that were scheduled on the frozen clock.
 */
export function createUnlockGate(context: () => AudioContext | null, onRunning: () => void): IUnlockGate {
    let bound = false;

    function settle(): void {
        const ctx = context();
        if (!ctx || ctx.state !== "running") return;
        unbind();
        onRunning();
    }

    function onGesture(): void {
        const ctx = context();
        if (!ctx) {
            unbind();
            return;
        }
        void ctx.resume().then(settle, () => undefined);
    }

    function bind(): void {
        if (bound || typeof window === "undefined" || !context()) return;
        bound = true;
        for (const type of UNLOCK_EVENTS) window.addEventListener(type, onGesture, { capture: true, passive: true });
    }

    function unbind(): void {
        if (!bound || typeof window === "undefined") return;
        bound = false;
        for (const type of UNLOCK_EVENTS) window.removeEventListener(type, onGesture, { capture: true });
    }

    return {
        arm() {
            bind();
            const ctx = context();
            if (ctx) void ctx.resume().then(settle, () => undefined);
        },
        settle,
        bind,
        dispose: unbind,
    };
}
