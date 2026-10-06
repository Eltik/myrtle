import { describe, expect, it } from "vitest";
import { filterForSource, type IGalleryFilter, type IGalleryTile, NO_GALLERY_FILTER } from "../gallery";
import { galleryFilterActions } from "./sources/Gallery";

// The Archives hold Integrated Strategies and Side Stories, never Main Story.
const ARCHIVES = [
    { kind: "archive_pic", id: "pic_rogue_2_KV1", title: "Key Visual", groupId: "rogue_2", groupName: "Mizuki & Caerula Arbor", category: "is" },
    { kind: "archive_pic", id: "act13side_pic_0", title: "Long Night", groupId: "act13side", groupName: "Near Light", category: "side" },
] as unknown as IGalleryTile[];

/** The browser's `chosen` state and its actions, as the Archives source reads it. */
function archivesBrowser(start: IGalleryFilter) {
    let chosen = start;
    const actions = () =>
        galleryFilterActions((update) => {
            chosen = update(chosen);
        });
    // What the chips show: the filter as the Archives read it.
    const shown = () => filterForSource(ARCHIVES, chosen).categories;
    return { actions, chosen: () => chosen, shown };
}

describe("toggleCategory", () => {
    it("keeps a category set aside for another source when one is toggled on", () => {
        const browser = archivesBrowser({ ...NO_GALLERY_FILTER, categories: ["main", "is"] });
        browser.actions().toggleCategory("side");
        expect(browser.chosen().categories).toEqual(["main", "is", "side"]);
        expect(browser.shown()).toEqual(["is", "side"]);
    });

    it("keeps it when one is toggled off, and turns off only the clicked one", () => {
        const browser = archivesBrowser({ ...NO_GALLERY_FILTER, categories: ["main", "is"] });
        browser.actions().toggleCategory("is");
        expect(browser.chosen().categories).toEqual(["main"]);
        expect(browser.shown()).toEqual([]);
    });

    it("toggles on and off as before with nothing set aside", () => {
        const browser = archivesBrowser(NO_GALLERY_FILTER);
        browser.actions().toggleCategory("is");
        expect(browser.chosen().categories).toEqual(["is"]);
        browser.actions().toggleCategory("is");
        expect(browser.chosen().categories).toEqual([]);
    });
});
