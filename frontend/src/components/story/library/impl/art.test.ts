import { describe, expect, it } from "vitest";
import { iconIsLogo, iconIsWide, plateCrop, plateSource, sectionIconSource, titleSource } from "./art";
import type { LibEntry, LibGroup } from "./derive";

function entry(id: string, over: Partial<LibEntry> = {}): LibEntry {
    return { id, name: id, sort: 1, groupId: "g", hasScript: true, requiredStages: [], ...over };
}

function group(id: string, over: Partial<LibGroup> = {}): LibGroup {
    return { id, name: id, category: "side", entryType: "ACTIVITY", actType: "ACTIVITY_STORY", startTime: 0, stories: [entry(`${id}_1`)], ...over };
}

describe("titleSource", () => {
    it("draws the authored logotype when the wire sends one, which is 81 of the 87 non-record groups", () => {
        expect(titleSource(group("act18d0", { titleImageUrl: "/textures/spritepack/mixstory_title_sprites_0/t.png" }))).toEqual({ kind: "logotype", url: "/textures/spritepack/mixstory_title_sprites_0/t.png" });
    });

    it("falls back to the fitted TEXT title on the 6 groups no storyline shelf lists", () => {
        expect(titleSource(group("act45side"))).toEqual({ kind: "text" });
    });

    it("treats an empty string as absent rather than drawing a zero-byte image", () => {
        expect(titleSource(group("act18d0", { titleImageUrl: "" }))).toEqual({ kind: "text" });
    });
});

describe("plateCrop", () => {
    it("centres the crop on an event, which is what the game's own card does at anchoredPos (0,0)", () => {
        expect(plateCrop(group("act28side"))).toBe("50% 50%");
    });

    it("lifts the crop on MAINLINE, whose 432x432 key visual has the chapter title typeset into rows 0.65..0.78", () => {
        expect(plateCrop(group("main_9", { category: "main" }))).toBe("50% 20%");
    });

    it("treats a vignette and a record like an event: only the mainline posters carry burnt-in type", () => {
        expect(plateCrop(group("story_leto_set_1", { category: "record" }))).toBe("50% 50%");
        expect(plateCrop(group("act13mini", { category: "vignette" }))).toBe("50% 50%");
    });
});

describe("plateSource", () => {
    it("prefers the key visual over the cover, which are never the same file", () => {
        expect(plateSource(group("main_3", { bannerUrl: "/kv.png", coverUrl: "/cover.png" }))).toEqual({ kind: "banner", url: "/kv.png" });
    });

    it("falls back to the cover where no shelf lists the group and there is no key visual", () => {
        expect(plateSource(group("act11d7", { coverUrl: "/cover.png" }))).toEqual({ kind: "cover", url: "/cover.png" });
    });

    it("answers none for the groups that carry neither, which draw the typographic plate", () => {
        expect(plateSource(group("act11d7"))).toEqual({ kind: "none" });
    });

    it("takes the key visual on the four chapters whose DERIVED cover is a fade, which is what the stub and the sampler both read", () => {
        for (const [id, cover] of [
            ["main_9", "/textures/avg/bg/avg_bkg_h1_bg_bl_0/bg_black.png"],
            ["main_12", "/textures/avg/bg/avg_bkg_h1_bg_bl_0/bg_black.png"],
            ["main_14", "/textures/avg/bg/avg_bkg_h1_bg_bl_0/bg_black.png"],
            ["main_15", "/textures/avg/bg/avg_bkg_h1_bg_wh_0/bg_white.png"],
        ]) {
            expect(plateSource(group(id, { bannerUrl: `/textures/spritepack/mixstory_kv_sprites_0/kv_${id}.png`, coverUrl: cover }))).toEqual({ kind: "banner", url: `/textures/spritepack/mixstory_kv_sprites_0/kv_${id}.png` });
        }
    });

    it("still answers the fade cover for the one group that has no key visual to prefer (act36side)", () => {
        expect(plateSource(group("act36side", { coverUrl: "/textures/avg/bg/avg_bkg_h1_bg_bl_0/bg_black.png" }))).toEqual({ kind: "cover", url: "/textures/avg/bg/avg_bkg_h1_bg_bl_0/bg_black.png" });
    });
});

describe("sectionIconSource", () => {
    it("takes the arc's own icon over the shelf's, because four arcs on one shelf would otherwise repeat one glyph", () => {
        expect(sectionIconSource("/act_0.png", "/mainline_logo.png", "/mainline_abbr.png")).toEqual({ kind: "arc", url: "/act_0.png" });
    });

    it("draws the shelf's LOGO on a section that is a whole shelf, and the abbreviation only when there is no logo", () => {
        expect(sectionIconSource(undefined, "/storyline_Rl.png", "/storyline_abbr_Rl.png")).toEqual({ kind: "logo", url: "/storyline_Rl.png" });
        expect(sectionIconSource(undefined, undefined, "/storyline_abbr_Rl.png")).toEqual({ kind: "shelf", url: "/storyline_abbr_Rl.png" });
    });

    it("falls back to the hashed lucide glyph when the wire names none of the three", () => {
        expect(sectionIconSource(undefined, undefined, undefined)).toEqual({ kind: "glyph" });
    });

    it("sizes ONLY the arc icon by height (a 184x52 banner) and ONLY the logo as the larger square", () => {
        expect(iconIsWide(sectionIconSource("/act_0.png", undefined, undefined))).toBe(true);
        expect(iconIsWide(sectionIconSource(undefined, "/storyline_Rl.png", undefined))).toBe(false);
        expect(iconIsLogo(sectionIconSource(undefined, "/storyline_Rl.png", undefined))).toBe(true);
        expect(iconIsLogo(sectionIconSource(undefined, undefined, "/storyline_abbr_Rl.png"))).toBe(false);
        expect(iconIsWide(sectionIconSource(undefined, undefined, undefined))).toBe(false);
    });
});
