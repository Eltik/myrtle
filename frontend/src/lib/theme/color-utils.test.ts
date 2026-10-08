import { afterEach, describe, expect, it } from "vitest";
import { ACCENT_STORAGE_KEY, accentToRenderedHex, applyAccentColors, clearAccentColors, DEFAULT_PRIMARY_HUE, generateAccentColors, getStoredAccent, parseAccent, serializeAccent, setStoredAccent } from "./color-utils";

const LIGHT_FG = "oklch(0.985 0.002 285)";
const DARK_FG = "oklch(0.13 0.005 285)";

describe("parseAccent and serializeAccent", () => {
    it("round-trip presets and custom hexes", () => {
        expect(serializeAccent({ type: "preset", hue: 250 })).toBe("h:250");
        expect(serializeAccent({ type: "custom", hex: "#ff8800" })).toBe("c:#ff8800");
        expect(parseAccent("h:250")).toEqual({ type: "preset", hue: 250 });
        expect(parseAccent("c:#ff8800")).toEqual({ type: "custom", hex: "#ff8800" });
    });

    it("wraps hues into 0-360", () => {
        expect(parseAccent("h:370")).toEqual({ type: "preset", hue: 10 });
        expect(parseAccent("h:-30")).toEqual({ type: "preset", hue: 330 });
        expect(parseAccent("h:12.5")).toEqual({ type: "preset", hue: 12.5 });
    });

    it("lowercases custom hexes and rejects anything but #rrggbb", () => {
        expect(parseAccent("c:#ABCDEF")).toEqual({ type: "custom", hex: "#abcdef" });
        for (const raw of ["c:#abc", "c:abcdef", "c:#abcdeg", "c:#abcdef0"]) {
            expect(parseAccent(raw)).toBeNull();
        }
    });

    it("reads a bare number as a legacy hue", () => {
        expect(parseAccent("145")).toEqual({ type: "preset", hue: 145 });
        expect(parseAccent("400")).toEqual({ type: "preset", hue: 40 });
        // `parseFloat` stops at the first non-number, so a trailing suffix still parses.
        expect(parseAccent("145deg")).toEqual({ type: "preset", hue: 145 });
    });

    it("rejects garbage", () => {
        for (const raw of ["", "h:", "h:abc", "blue", "x:10"]) {
            expect(parseAccent(raw)).toBeNull();
        }
    });
});

describe("generateAccentColors", () => {
    it("builds a preset from the OKLch formula per theme", () => {
        const dark = generateAccentColors({ type: "preset", hue: 250 }, true);
        expect(dark.primary).toBe("oklch(0.75 0.15 250)");
        expect(dark.primaryForeground).toBe(DARK_FG);
        expect(dark.glowPrimary).toBe("oklch(0.75 0.15 250 / 0.5)");
        expect(dark.glowPrimaryIntense).toBe("oklch(0.75 0.15 250 / 0.8)");
        expect(dark.glowTextIcon).toBe("oklch(0.75 0.15 250 / 0.6)");

        const light = generateAccentColors({ type: "preset", hue: 250 }, false);
        expect(light.primary).toBe("oklch(0.58 0.22 250)");
        expect(light.primaryForeground).toBe(LIGHT_FG);
        expect(light.glowPrimary).toBe("oklch(0.58 0.22 250 / 0.35)");
        expect(light.glowTextIcon).toBe("oklch(0.58 0.22 250 / 0.55)");
        expect(light.ring).toBe(light.primary);
        expect(light.sidebarPrimary).toBe(light.primary);
    });

    it("uses a custom hex as is, with alpha glows and a legible foreground", () => {
        const c = generateAccentColors({ type: "custom", hex: "#336699" }, true);
        expect(c.primary).toBe("#336699");
        expect(c.glowPrimary).toBe("rgb(51 102 153 / 0.5)");
        expect(c.glowPrimaryIntense).toBe("rgb(51 102 153 / 0.8)");
        expect(c.primaryForeground).toBe(LIGHT_FG);
        expect(generateAccentColors({ type: "custom", hex: "#ffee88" }, false).primaryForeground).toBe(DARK_FG);
    });

    it("expands three-digit hexes and passes unreadable ones through", () => {
        expect(generateAccentColors({ type: "custom", hex: "#fff" }, true).glowPrimary).toBe("rgb(255 255 255 / 0.5)");
        const bad = generateAccentColors({ type: "custom", hex: "nope" }, true);
        expect(bad.glowPrimary).toBe("nope");
        expect(bad.primaryForeground).toBe(LIGHT_FG);
    });

    it("parses a hex with a stray non-hex digit by its leading digits", () => {
        // CURRENT BEHAVIOR, suspected bug: `hexToRgb` relies on `parseInt`, which
        // stops at the first non-hex character instead of rejecting the input, so
        // "#12345g" reads as 0x12345 rather than as invalid. Only reachable for a
        // custom accent that bypassed `parseAccent`'s #rrggbb check.
        expect(generateAccentColors({ type: "custom", hex: "#12345g" }, true).glowPrimary).toBe("rgb(1 35 69 / 0.5)");
    });
});

describe("accentToRenderedHex", () => {
    it("returns a custom hex unchanged", () => {
        expect(accentToRenderedHex({ type: "custom", hex: "#123456" }, true)).toBe("#123456");
    });

    it("renders presets to sRGB per theme and defaults to the brand hue", () => {
        const hex = /^#[0-9a-f]{6}$/;
        const dark = accentToRenderedHex({ type: "preset", hue: 250 }, true);
        const light = accentToRenderedHex({ type: "preset", hue: 250 }, false);
        expect(dark).toMatch(hex);
        expect(light).toMatch(hex);
        expect(dark).not.toBe(light);
        expect(accentToRenderedHex(null, true)).toBe(accentToRenderedHex({ type: "preset", hue: DEFAULT_PRIMARY_HUE }, true));
    });
});

describe("CSS variables", () => {
    it("are set and cleared on the root", () => {
        const root = document.createElement("div");
        applyAccentColors(root, generateAccentColors({ type: "custom", hex: "#336699" }, true));
        expect(root.style.getPropertyValue("--primary")).toBe("#336699");
        expect(root.style.getPropertyValue("--chart-1")).toBe("#336699");
        expect(root.style.getPropertyValue("--glow-primary")).not.toBe("");
        clearAccentColors(root);
        expect(root.style.getPropertyValue("--primary")).toBe("");
        expect(root.style.getPropertyValue("--glow-text-icon")).toBe("");
    });
});

describe("stored accent", () => {
    afterEach(() => window.localStorage.removeItem(ACCENT_STORAGE_KEY));

    it("saves, reads back and clears", () => {
        expect(getStoredAccent()).toBeNull();
        setStoredAccent({ type: "preset", hue: 180 });
        expect(window.localStorage.getItem(ACCENT_STORAGE_KEY)).toBe("h:180");
        expect(getStoredAccent()).toEqual({ type: "preset", hue: 180 });
        setStoredAccent(null);
        expect(getStoredAccent()).toBeNull();
    });
});
