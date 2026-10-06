import { describe, expect, it } from "vitest";
import { skinTexture, skinThumbnail } from "#/components/operators/detail/impl/assets";
import { artSlack, artStyle, backgroundSources, clampFocus, clampScale, cropAxes, croppableAxes, DEFAULT_FOCUS_Y, galleryPictureUrl, galleryThumbUrl, isGalleryKind, objectPosition, panFocus, pickBackground, SCALE_MAX, SCALE_MIN, sameBackground, withFocusX, withFocusY, withScale, zoomOf } from "./background";

/** The part of a URL after the backend origin, so the test does not depend on `VITE_BACKEND_URL`. */
function paths(urls: string[]): string[] {
    return urls.map((url) => url.slice(url.indexOf("/api/")));
}

describe("backgroundSources", () => {
    it("tries a skin's reduced art, then its full art, on the default server and then CN", () => {
        expect(paths(backgroundSources({ kind: "skin", id: "char_002_amiya@epoque#4" }))).toEqual([
            "/api/assets/textures/skinpack/char_002_amiya/char_002_amiya_epoque%234b.png",
            "/api/assets/textures/skinpack/char_002_amiya/char_002_amiya_epoque%234.png",
            "/api/cn/assets/textures/skinpack/char_002_amiya/char_002_amiya_epoque%234b.png",
            "/api/cn/assets/textures/skinpack/char_002_amiya/char_002_amiya_epoque%234.png",
        ]);
    });

    it("builds the same skin URLs the skins pages build", () => {
        const [reduced, full] = backgroundSources({ kind: "skin", id: "char_002_amiya@winter#1" });
        expect(reduced).toBe(skinThumbnail("char_002_amiya", "char_002_amiya@winter#1"));
        expect(full).toBe(skinTexture("char_002_amiya", "char_002_amiya@winter#1"));
    });

    it("files an outfit under the id's own prefix, not the wearer", () => {
        // skin_table files these two under char_002_amiya; the art lives under the id's prefix.
        expect(paths(backgroundSources({ kind: "skin", id: "char_1001_amiya2@sale#16" }))[0]).toBe("/api/assets/textures/skinpack/char_1001_amiya2/char_1001_amiya2_sale%2316b.png");
    });

    it("tries an operator's elite 2 art, reduced first, then elite 1", () => {
        expect(paths(backgroundSources({ kind: "operator", id: "char_003_kalts" })).slice(0, 4)).toEqual([
            "/api/assets/textures/chararts/char_003_kalts/char_003_kalts_2b.png",
            "/api/assets/textures/chararts/char_003_kalts/char_003_kalts_2.png",
            "/api/assets/textures/chararts/char_003_kalts/char_003_kalts_1b.png",
            "/api/assets/textures/chararts/char_003_kalts/char_003_kalts_1.png",
        ]);
    });

    it("has nothing to try for a skin id with no outfit part", () => {
        expect(backgroundSources({ kind: "skin", id: "char_002_amiya" })).toEqual([]);
        expect(backgroundSources({ kind: "skin", id: "@x#1" })).toEqual([]);
    });
});

describe("focus", () => {
    it("reads a missing focus as the centre and clamps the rest", () => {
        expect(objectPosition({})).toBe("50% 50%");
        expect(objectPosition({ focus_y: 20 })).toBe("50% 20%");
        expect(objectPosition({ focus_x: -5, focus_y: 140 })).toBe("0% 100%");
        // 0 is a focus, not a missing one.
        expect(objectPosition({ focus_x: 0, focus_y: 0 })).toBe("0% 0%");
    });

    it("clamps and rounds like the backend", () => {
        expect([clampFocus(-1), clampFocus(0.49), clampFocus(99.5), clampFocus(250)]).toEqual([0, 0, 100, 100]);
        expect(withFocusY({ kind: "operator", id: "a" }, 33.6)).toEqual({ kind: "operator", id: "a", focus_y: 34 });
    });
});

describe("picking", () => {
    it("starts a new art at the default crop and keeps the crop of the one already shown", () => {
        expect(pickBackground(null, "operator", "char_003_kalts")).toEqual({ kind: "operator", id: "char_003_kalts", focus_y: DEFAULT_FOCUS_Y });
        const shown = { kind: "operator" as const, id: "char_003_kalts", focus_y: 70 };
        expect(pickBackground(shown, "operator", "char_003_kalts")).toBe(shown);
        expect(pickBackground(shown, "skin", "char_003_kalts@x#1").focus_y).toBe(DEFAULT_FOCUS_Y);
    });

    it("compares by picture and crop, a missing focus equal to the centre", () => {
        expect(sameBackground(null, undefined)).toBe(true);
        expect(sameBackground(null, { kind: "operator", id: "a" })).toBe(false);
        expect(sameBackground({ kind: "operator", id: "a" }, { kind: "operator", id: "a", focus_x: 50, focus_y: 50 })).toBe(true);
        expect(sameBackground({ kind: "operator", id: "a", focus_y: 20 }, { kind: "operator", id: "a", focus_y: 21 })).toBe(false);
        expect(sameBackground({ kind: "operator", id: "a" }, { kind: "skin", id: "a" })).toBe(false);
    });
});

describe("story CGs and scenes", () => {
    it("draw the header JPEG of their own kind, on the default server and then through CN", () => {
        expect(paths(backgroundSources({ kind: "story_cg", id: "avg_1_1" }))).toEqual(["/api/story/art-gallery/cg/avg_1_1/header", "/api/cn/story/art-gallery/cg/avg_1_1/header"]);
        expect(paths(backgroundSources({ kind: "story_scene", id: "bg_indoor_1" }))).toEqual(["/api/story/art-gallery/scene/bg_indoor_1/header", "/api/cn/story/art-gallery/scene/bg_indoor_1/header"]);
    });

    it("tile with the 320 px JPEG, and behave as a gallery picture", () => {
        expect(paths([galleryThumbUrl("story_scene", "a b#1"), galleryThumbUrl("story_cg", "avg_1_1"), galleryThumbUrl("archive_pic", "act13side_pic_0")])).toEqual(["/api/story/art-gallery/scene/a%20b%231/thumb", "/api/story/art-gallery/cg/avg_1_1/thumb", "/api/story/gallery/act13side_pic_0/thumb"]);
        expect(pickBackground(null, "story_cg", "avg_1_1")).toEqual({ kind: "story_cg", id: "avg_1_1" });
        expect(cropAxes("story_scene")).toEqual(["x", "y"]);
        expect(["skin", "operator", "archive_pic", "story_cg", "story_scene"].map((k) => isGalleryKind(k as "skin"))).toEqual([false, false, true, true, true]);
    });
});

describe("gallery pictures", () => {
    it("draw the 1600 px header JPEG, on the default server and then through CN", () => {
        expect(paths(backgroundSources({ kind: "archive_pic", id: "pic_rogue_1_KV1" }))).toEqual(["/api/story/gallery/pic_rogue_1_KV1/header", "/api/cn/story/gallery/pic_rogue_1_KV1/header"]);
    });

    it("tile the picker with the 320 px JPEG, the id kept in its case and encoded", () => {
        expect(paths([galleryPictureUrl("act13side_pic_0", "thumb")])).toEqual(["/api/story/gallery/act13side_pic_0/thumb"]);
        expect(paths([galleryPictureUrl("a b#1", "thumb", "cn")])).toEqual(["/api/cn/story/gallery/a%20b%231/thumb"]);
    });

    it("start at the centre, not the portrait kinds' face height", () => {
        expect(pickBackground(null, "archive_pic", "act13side_pic_0")).toEqual({ kind: "archive_pic", id: "act13side_pic_0" });
        expect(objectPosition(pickBackground(null, "archive_pic", "act13side_pic_0"))).toBe("50% 50%");
        expect(pickBackground(null, "operator", "char_003_kalts").focus_y).toBe(DEFAULT_FOCUS_Y);
    });

    it("keep their crop when picked again, and crop on both axes", () => {
        const bg = withFocusY(withFocusX(pickBackground(null, "archive_pic", "act13side_pic_0"), 71.6), -3);
        expect(bg).toEqual({ kind: "archive_pic", id: "act13side_pic_0", focus_x: 72, focus_y: 0 });
        expect(pickBackground(bg, "archive_pic", "act13side_pic_0")).toBe(bg);
        expect(objectPosition(bg)).toBe("72% 0%");
        expect(cropAxes("archive_pic")).toEqual(["x", "y"]);
        expect(cropAxes("skin")).toEqual(["y"]);
        expect(cropAxes("operator")).toEqual(["y"]);
    });

    it("are not the same background as an operator of the same id", () => {
        expect(sameBackground({ kind: "archive_pic", id: "x" }, { kind: "operator", id: "x" })).toBe(false);
        expect(clampFocus(100.4)).toBe(100);
    });
});

describe("zoom", () => {
    it("reads a missing scale as 100 and clamps and rounds like the backend", () => {
        expect(zoomOf({})).toBe(SCALE_MIN);
        expect(zoomOf({ scale: 0 })).toBe(SCALE_MIN);
        expect(zoomOf({ scale: 149.5 })).toBe(150);
        expect([clampScale(40), clampScale(1e9)]).toEqual([SCALE_MIN, SCALE_MAX]);
    });

    it("drops the key at 100, so an unzoomed background keeps its pre-zoom shape", () => {
        const bg = { kind: "operator" as const, id: "a", focus_y: 25 };
        expect(withScale(bg, 180)).toEqual({ ...bg, scale: 180 });
        expect(withScale({ ...bg, scale: 180 }, 100)).toEqual(bg);
        expect(withScale(bg, 20)).toEqual(bg);
        expect(withScale(bg, 999).scale).toBe(SCALE_MAX);
    });

    it("compares by zoom, a missing scale equal to 100", () => {
        expect(sameBackground({ kind: "operator", id: "a" }, { kind: "operator", id: "a", scale: 100 })).toBe(true);
        expect(sameBackground({ kind: "operator", id: "a" }, { kind: "operator", id: "a", scale: 101 })).toBe(false);
    });

    it("draws an unzoomed art with object-position alone, exactly as before zoom", () => {
        expect(artStyle({ focus_y: 51 })).toEqual({ objectPosition: "50% 51%" });
        expect(artStyle({ focus_y: 51, scale: 100 })).toEqual({ objectPosition: "50% 51%" });
    });

    it("scales about the focus and fits the mask back onto the unscaled box", () => {
        const style = artStyle({ focus_x: 70, focus_y: 20, scale: 200 });
        expect(style).toMatchObject({ objectPosition: "70% 20%", transform: "scale(2)", transformOrigin: "70% 20%", maskSize: "50% 50%", maskPosition: "70% 20%", maskRepeat: "no-repeat" });
    });

    it("covers the box at every zoom and focus", () => {
        // A box [0, W] scaled by s about O = pW maps to [O(1 - s), O + s(W - O)].
        const W = 892;
        for (let scale = SCALE_MIN; scale <= SCALE_MAX; scale += 10) {
            for (let p = 0; p <= 100; p += 5) {
                const s = scale / 100;
                const o = (p / 100) * W;
                expect(o * (1 - s)).toBeLessThanOrEqual(0);
                expect(o + s * (W - o)).toBeGreaterThanOrEqual(W - 1e-9);
                // The mask, 1/s of the box at the focus percentage, lands on [0, W] after the transform.
                const maskLeft = (p / 100) * (W - W / s);
                expect(o + s * (maskLeft - o)).toBeCloseTo(0, 6);
                expect(o + s * (maskLeft + W / s - o)).toBeCloseTo(W, 6);
            }
        }
    });

    it("offers both crop axes for any art once zoomed", () => {
        expect(cropAxes("operator", 100)).toEqual(["y"]);
        expect(cropAxes("operator", 101)).toEqual(["x", "y"]);
        expect(cropAxes("skin", 250)).toEqual(["x", "y"]);
    });
});

describe("panning", () => {
    // The live header 2026-10-06: an 892 x 217 art box (the right 62% at 1500 px).
    const gallery = { boxWidth: 892, boxHeight: 217, naturalWidth: 1600, naturalHeight: 900 };
    const square = { boxWidth: 892, boxHeight: 217, naturalWidth: 1024, naturalHeight: 1024 };

    it("moves the art with the pointer: one pixel is 1 / (sR - B) of focus", () => {
        // Cover 0.5575 draws the picture 892 x 501.75; at 2.12 that is 1891 x 1063.7, so 999 and 846.7 px of slack.
        const next = panFocus({ kind: "archive_pic", id: "p", focus_y: 51, scale: 212 }, -200, -50, gallery);
        expect(next).toEqual({ kind: "archive_pic", id: "p", focus_x: 70, focus_y: 57, scale: 212 });
    });

    it("leaves an axis the art does not overflow alone", () => {
        // Unzoomed, a square art is exactly as wide as the box: no horizontal slack.
        const start = { kind: "operator" as const, id: "a", focus_y: 25 };
        const next = panFocus(start, -300, 100, square);
        expect(next.focus_x).toBeUndefined();
        expect(next.focus_y).toBe(clampFocus(25 - (100 / (892 - 217)) * 100));
        // Zoomed to 157 it is 1400 px wide, 508 px of slack: the live drag of 2026-10-06 (450, 150) from (50, 25) landed on (0, 12).
        expect(panFocus({ ...start, scale: 157 }, 100, 150, square)).toMatchObject({ focus_x: 30, focus_y: 12 });
        expect(panFocus({ ...start, scale: 157 }, 450, 150, square)).toMatchObject({ focus_x: 0, focus_y: 12 });
    });

    it("clamps at the edges and ignores an art that has not loaded", () => {
        expect(panFocus({ kind: "operator", id: "a", focus_y: 25, scale: 300 }, -1e6, 1e6, square)).toMatchObject({ focus_x: 100, focus_y: 0 });
        const start = { kind: "operator" as const, id: "a" };
        expect(panFocus(start, 10, 10, { ...square, naturalWidth: 0 })).toBe(start);
    });
});

describe("crop slack", () => {
    // The live header 2026-10-06 at a 1500 x 753 window: the art box is 892 x 217; a story CG is 1600 x 900.
    const desktop = { boxWidth: 892, boxHeight: 217, naturalWidth: 1600, naturalHeight: 900 };
    // A phone's header: the art fills it, narrower than 16:9.
    const phone = { boxWidth: 358, boxHeight: 300, naturalWidth: 1600, naturalHeight: 900 };

    it("measures the overflow on each axis: none sideways for a 16:9 CG in a 4.11:1 box at 100", () => {
        const at100 = artSlack(desktop, undefined);
        expect(at100?.x).toBeCloseTo(0, 6);
        expect(at100?.y).toBeCloseTo(284.75, 2);
        expect(artSlack(desktop, 101)?.x).toBeCloseTo(8.92, 2);
        expect(artSlack({ ...desktop, boxWidth: 0 }, 150)).toBeNull();
    });

    it("enables an axis exactly when it has slack, for every kind", () => {
        expect(croppableAxes("story_cg", undefined, desktop)).toEqual({ x: false, y: true });
        expect(croppableAxes("story_cg", 101, desktop)).toEqual({ x: true, y: true });
        // On the phone it is the other way round: the CG overflows sideways and fits the height.
        expect(croppableAxes("story_cg", undefined, phone)).toEqual({ x: true, y: false });
        expect(croppableAxes("operator", undefined, { ...desktop, naturalWidth: 1024, naturalHeight: 1024 })).toEqual({ x: false, y: true });
    });

    it("falls back to the kind's axes before the art has loaded", () => {
        expect(croppableAxes("story_cg", undefined, null)).toEqual({ x: true, y: true });
        expect(croppableAxes("operator", undefined, null)).toEqual({ x: false, y: true });
        expect(croppableAxes("operator", 120, null)).toEqual({ x: true, y: true });
    });

    it("pans only along an axis with slack, as the sliders read it", () => {
        const start = { kind: "story_cg" as const, id: "61_i14", focus_x: 78, focus_y: 28 };
        expect(panFocus(start, -400, 0, desktop)).toEqual(start);
        expect(panFocus({ ...start, scale: 101 }, -4.46, 0, desktop).focus_x).toBe(clampFocus(78 + 50));
    });
});
