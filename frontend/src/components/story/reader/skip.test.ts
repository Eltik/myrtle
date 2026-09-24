import { describe, expect, it } from "vitest";
import { nextSkip, SKIP_OFF, type SkipState, skipAdvances, skipRatio } from "./skip";

const armed: SkipState = { phase: "off", confirmed: true };

describe("nextSkip", () => {
    it("asks once, then never again in the same session", () => {
        const first = nextSkip(SKIP_OFF, "press");
        expect(first.phase).toBe("confirming");
        const running = nextSkip(first, "confirm");
        expect(running).toEqual({ phase: "on", confirmed: true });
        const stopped = nextSkip(running, "click");
        expect(stopped.phase).toBe("off");
        // The second press of the session goes straight through.
        expect(nextSkip(stopped, "press").phase).toBe("on");
    });

    it("cancelling leaves the skip off and does NOT count as the session's answer", () => {
        const cancelled = nextSkip(nextSkip(SKIP_OFF, "press"), "cancel");
        expect(cancelled).toEqual({ phase: "off", confirmed: false });
        expect(nextSkip(cancelled, "press").phase).toBe("confirming");
    });

    it("stops at a decision, at the end of the story and on a click", () => {
        for (const stop of ["decision", "boundary", "click"] as const) {
            expect(nextSkip({ phase: "on", confirmed: true }, stop).phase).toBe("off");
        }
    });

    it("a stop is inert while nothing is skipping, and a click never closes the confirm", () => {
        expect(nextSkip(armed, "decision")).toBe(armed);
        const asking = nextSkip(SKIP_OFF, "press");
        expect(nextSkip(asking, "click")).toBe(asking);
    });

    it("the button toggles: a press while skipping stops it, and a press while asking dismisses", () => {
        expect(nextSkip({ phase: "on", confirmed: true }, "press").phase).toBe("off");
        expect(nextSkip({ phase: "confirming", confirmed: false }, "press").phase).toBe("off");
    });
});

describe("skipRatio", () => {
    it("is zero while skipping and the setting's own value otherwise", () => {
        expect(skipRatio({ phase: "on", confirmed: true }, 1)).toBe(0);
        expect(skipRatio({ phase: "on", confirmed: true }, 2.5)).toBe(0);
        expect(skipRatio(armed, 2.5)).toBe(2.5);
        // The confirm is open and nothing has been skipped yet: the scene runs
        // at its own speed behind the dialog.
        expect(skipRatio({ phase: "confirming", confirmed: false }, 1)).toBe(1);
        // A user who set the speed to 0 themselves keeps 0, which is the same
        // number by a different route and must not read as "skipping".
        expect(skipRatio(armed, 0)).toBe(0);
    });
});

describe("skipAdvances", () => {
    it("only steps on a line of a story that is being read", () => {
        const on: SkipState = { phase: "on", confirmed: true };
        expect(skipAdvances(on, "reading", "line")).toBe(true);
        expect(skipAdvances(on, "reading", "decision")).toBe(false);
        expect(skipAdvances(on, "end", "line")).toBe(false);
        expect(skipAdvances(on, "reading", undefined)).toBe(false);
        expect(skipAdvances(armed, "reading", "line")).toBe(false);
    });

    it("steps past a cutscene too, so a running skip does not sit through a clip", () => {
        const on: SkipState = { phase: "on", confirmed: true };
        expect(skipAdvances(on, "reading", "video")).toBe(true);
        expect(skipAdvances(armed, "reading", "video")).toBe(false);
    });
});
