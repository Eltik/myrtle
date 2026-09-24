import { describe, expect, it } from "vitest";
import { CUSTOM_FONT_FAMILY, FONT_FILE_ACCEPT, fontFamilyFor, isFontFileName, MAX_FONT_BYTES, PRESET_FONT_FAMILY, STORY_FONTS, SYSTEM_FONT_STACK } from "./fonts";

const PRESET = "var(--font-sans)";

describe("fontFamilyFor", () => {
    it("returns the preset's own family for the default, unchanged", () => {
        expect(fontFamilyFor("preset", PRESET, false)).toBe(PRESET);
        expect(fontFamilyFor("preset", "var(--font-display)", true)).toBe("var(--font-display)");
    });

    it("maps every site token to its variable", () => {
        expect(fontFamilyFor("sans", PRESET, false)).toBe("var(--font-sans)");
        expect(fontFamilyFor("heading", PRESET, false)).toBe("var(--font-heading)");
        expect(fontFamilyFor("display", PRESET, false)).toBe("var(--font-display)");
        expect(fontFamilyFor("mono", PRESET, false)).toBe("var(--font-mono)");
        expect(fontFamilyFor("cjk", PRESET, false)).toBe("var(--font-cjk)");
    });

    it("uses the platform stack for System and never a site webfont", () => {
        const family = fontFamilyFor("system", PRESET, false);
        expect(family).toBe(SYSTEM_FONT_STACK);
        expect(family).not.toContain("var(--font");
    });

    it("puts the custom family first with the preset behind it, once it is loaded", () => {
        expect(fontFamilyFor("custom", PRESET, true)).toBe(`"${CUSTOM_FONT_FAMILY}", ${PRESET}`);
    });

    it("falls back to the preset while the custom face is not loaded", () => {
        expect(fontFamilyFor("custom", PRESET, false)).toBe(PRESET);
    });

    it("puts a Terra script first and the reading preset behind it, so a glyph it lacks falls to the reader's own font", () => {
        expect(fontFamilyFor("terraAegir", "var(--font-sans)", false)).toBe('"Terra Aegir", var(--font-sans)');
        expect(fontFamilyFor("terraSami", "var(--font-display)", false)).toBe('"Terra Sami", var(--font-display)');
        expect(fontFamilyFor("terraSarkaz", "var(--font-sans)", false)).toBe('"Terra Sarkaz", var(--font-sans)');
    });

    it("puts OpenDyslexic first and the reader's default face behind it, so a CJK line is not dropped on a Latin-only face", () => {
        expect(fontFamilyFor("dyslexic", PRESET, false)).toBe(`"OpenDyslexic", ${PRESET_FONT_FAMILY}`);
    });

    it("covers every id in the registry", () => {
        for (const id of STORY_FONTS) expect(typeof fontFamilyFor(id, PRESET, true)).toBe("string");
        expect(STORY_FONTS).toHaveLength(12);
    });
});

describe("font files", () => {
    it("accepts the four extensions in any case and nothing else", () => {
        expect(isFontFileName("SFNSMono.ttf")).toBe(true);
        expect(isFontFileName("Inter.OTF")).toBe(true);
        expect(isFontFileName("x.woff")).toBe(true);
        expect(isFontFileName("x.woff2")).toBe(true);
        expect(isFontFileName("x.ttf.png")).toBe(false);
        expect(isFontFileName("noextension")).toBe(false);
    });

    it("declares the same four extensions to the file input", () => {
        expect(FONT_FILE_ACCEPT).toBe(".ttf,.otf,.woff,.woff2");
    });

    it("caps an upload at 32 MB", () => {
        expect(MAX_FONT_BYTES).toBe(33554432);
    });
});
