import { describe, expect, it } from "vitest";
import { nextSkip, SKIP_CLOSED, skipAvailable, synopsisParagraphs } from "./skip";

describe("nextSkip", () => {
    it("every press opens the sheet: there is no once-per-session confirm", () => {
        const first = nextSkip(SKIP_CLOSED, "press", "reading");
        expect(first).toEqual({ state: { open: true }, toEnd: false });
        const cancelled = nextSkip(first.state, "cancel", "reading");
        expect(cancelled).toEqual({ state: SKIP_CLOSED, toEnd: false });
        // The second press of the session asks again.
        expect(nextSkip(cancelled.state, "press", "reading").state.open).toBe(true);
    });

    it("confirm closes the sheet and jumps to the end, with no stepping in between", () => {
        const open = nextSkip(SKIP_CLOSED, "press", "reading").state;
        expect(nextSkip(open, "confirm", "reading")).toEqual({ state: SKIP_CLOSED, toEnd: true });
    });

    it("the button toggles: a press while the sheet is open closes it", () => {
        expect(nextSkip({ open: true }, "press", "reading")).toEqual({ state: SKIP_CLOSED, toEnd: false });
    });

    it("does nothing on the title or the end card", () => {
        for (const phase of ["title", "resume", "end"]) {
            expect(nextSkip(SKIP_CLOSED, "press", phase)).toEqual({ state: SKIP_CLOSED, toEnd: false });
            // A sheet somehow still open when the story ended never jumps twice.
            expect(nextSkip({ open: true }, "confirm", phase).toEnd).toBe(false);
        }
    });

    it("a confirm with no sheet open is inert", () => {
        expect(nextSkip(SKIP_CLOSED, "confirm", "reading").toEnd).toBe(false);
    });
});

describe("skipAvailable", () => {
    it("is only true while a story is being read", () => {
        expect(skipAvailable("reading")).toBe(true);
        expect(skipAvailable("title")).toBe(false);
        expect(skipAvailable("end")).toBe(false);
    });
});

describe("synopsisParagraphs", () => {
    it("splits on lines, trims and drops blanks", () => {
        expect(synopsisParagraphs("First.\r\n\r\n  Second.  \nThird.")).toEqual(["First.", "Second.", "Third."]);
    });

    it("is empty for a missing or blank synopsis", () => {
        expect(synopsisParagraphs(undefined)).toEqual([]);
        expect(synopsisParagraphs(null)).toEqual([]);
        expect(synopsisParagraphs(" \n ")).toEqual([]);
    });
});
