import { describe, expect, it } from "vitest";
import { DESKTOP_FALLBACK_VIEWPORT, defaultPreviewMode, PHONE_VIEWPORT, previewMaxHeight, previewScale, previewViewport, STRIP_HEIGHT, STRIP_SLACK, stripShown } from "./preview-geometry";

describe("previewViewport", () => {
    it("previews the page as it is on a desktop, and a 390 px phone", () => {
        expect(previewViewport("desktop", 1500)).toBe(1500);
        expect(previewViewport("phone", 1500)).toBe(PHONE_VIEWPORT);
        expect(previewViewport("desktop", 640)).toBe(640);
        expect(previewViewport("phone", 640)).toBe(390);
    });

    it("previews this phone's own width on a phone, and a 1280 px desktop", () => {
        expect(previewViewport("phone", 375)).toBe(375);
        expect(previewViewport("phone", 639)).toBe(639);
        expect(previewViewport("desktop", 375)).toBe(DESKTOP_FALLBACK_VIEWPORT);
    });

    it("opens in the mode that matches the screen", () => {
        expect(defaultPreviewMode(1500)).toBe("desktop");
        expect(defaultPreviewMode(640)).toBe("desktop");
        expect(defaultPreviewMode(639)).toBe("phone");
    });
});

describe("previewScale", () => {
    it("shows a header that fits life-size, never larger", () => {
        expect(previewScale(362, 299, 1452, 316)).toBe(1);
        expect(previewScale(100, 100, 1000, 1000)).toBe(1);
    });

    it("scales by the tighter of the two limits", () => {
        expect(previewScale(1440, 240, 1200, 316)).toBeCloseTo(1200 / 1440, 12);
        expect(previewScale(362, 299, 1452, 200)).toBeCloseTo(200 / 299, 12);
    });

    it("scales by 1 before the header is measured", () => {
        expect(previewScale(0, 0, 1000, 300)).toBe(1);
        expect(previewScale(500, 200, 0, 300)).toBe(1);
    });

    it("caps the preview at 42% of the viewport's height, 160 px at least", () => {
        expect(previewMaxHeight(753)).toBe(316);
        expect(previewMaxHeight(300)).toBe(160);
    });
});

describe("stripShown", () => {
    it("shows the strip once the full preview's bottom is under the strip's own bottom, with one px of slack", () => {
        expect(stripShown(400)).toBe(false);
        expect(stripShown(STRIP_HEIGHT + STRIP_SLACK)).toBe(false);
        expect(stripShown(STRIP_HEIGHT + 0.99)).toBe(true);
        expect(stripShown(STRIP_HEIGHT)).toBe(true);
        expect(stripShown(147.75)).toBe(true);
        expect(stripShown(-500)).toBe(true);
    });

    /**
     * The scroller as the editor lays it out on Character art: the preview section, then a
     * block filling the height under the strip. Nothing in it depends on whether the strip
     * shows, so the scroll range is one number; scrollHeight and scrollTop are whole px, the
     * section's height is not.
     */
    const scroller = (section: number, clientHeight: number) => {
        const scrollHeight = Math.round(section + (clientHeight - STRIP_HEIGHT));
        return { max: scrollHeight - clientHeight, bottomAt: (scrollTop: number) => section - scrollTop };
    };

    it("never flips while scrolling down to the end, and shows at the end, whatever the section's fraction", () => {
        // 290.75: Desktop at 1500 x 716; 373: Phone at 1500 x 716, where the old rule flipped 48 times in 30 steps.
        for (const section of [290.75, 373, 300.5, 301.25, 160]) {
            const { max, bottomAt } = scroller(section, 660);
            const states: boolean[] = [];
            for (let top = 0; top <= max + 300; top += 60) states.push(stripShown(bottomAt(Math.min(top, max))));
            // Wheel steps at the end clamp to `max`: the strip stays as it is, and it is shown.
            for (let i = 0; i < 10; i++) states.push(stripShown(bottomAt(max)));
            const flips = states.filter((shown, i) => i > 0 && shown !== states[i - 1]).length;
            expect({ section, flips, end: states.at(-1) }).toEqual({ section, flips: 1, end: true });
        }
    });
});
