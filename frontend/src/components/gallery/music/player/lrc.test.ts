import { describe, expect, it } from "vitest";
import { activeLineIndex, parseLrc } from "./lrc";

/** Shaped like the label's files: CRLF endings, a credits block at 0, metadata, blank clears. */
const SAMPLE = ["[ti:Example]", "[ar:塞壬唱片-MSR]", "[00:00.00]作曲 Composer: Someone", "[00:00.50]编曲 Arranger: Someone Else", "[00:12.34]First line", "", "[00:20.5]   ", "[00:18.00][01:02.25]Chorus line", "[00:15]Second line", "[01:00.000]Late line"].join("\r\n");

describe("parseLrc", () => {
    it("reads every time tag, drops metadata and blank text, and sorts by time", () => {
        const lines = parseLrc(SAMPLE);
        expect(lines.map((l) => [l.time, l.text])).toEqual([
            [0, "作曲 Composer: Someone"],
            [0.5, "编曲 Arranger: Someone Else"],
            [12.34, "First line"],
            [15, "Second line"],
            [18, "Chorus line"],
            [60, "Late line"],
            [62.25, "Chorus line"],
        ]);
    });

    it("leaves no carriage return in the text", () => {
        for (const line of parseLrc(SAMPLE)) expect(line.text.includes("\r")).toBe(false);
    });

    it("reads a one-digit fraction as tenths", () => {
        expect(parseLrc("[00:01.5]a")[0]?.time).toBe(1.5);
    });

    it("returns nothing for a file with no timed text", () => {
        expect(parseLrc("[ti:x]\r\n\r\nplain words")).toEqual([]);
    });
});

describe("activeLineIndex", () => {
    const lines = parseLrc(SAMPLE);
    it("is -1 before the first line and the last line at or before the time after", () => {
        expect(activeLineIndex(lines, -1)).toBe(-1);
        expect(activeLineIndex(lines, 0)).toBe(0);
        expect(activeLineIndex(lines, 14.9)).toBe(2);
        expect(activeLineIndex(lines, 15)).toBe(3);
        expect(activeLineIndex(lines, 999)).toBe(lines.length - 1);
    });
});
