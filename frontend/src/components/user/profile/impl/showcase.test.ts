import { describe, expect, it } from "vitest";
import { toTierEntity, UNPLACED } from "#/lib/api/tier-entities";
import type { EntitySummary } from "#/types/generated/EntitySummary";
import type { ProfileLayout } from "#/types/generated/ProfileLayout";
import type { ShowcaseBlock } from "#/types/generated/ShowcaseBlock";
import type { ShowcaseView } from "#/types/generated/ShowcaseView";
import { normalizeLayout, resolveActiveTab, shownTabs, tabMemory } from "./layout";
import { blocksForSave, cleanTitle, draftFromView, droppedOnSave, newFavourites, SHOWCASE_MAX_BLOCKS, SHOWCASE_MAX_IDS, type ShowcaseDraftBlock, showcaseForSave, shownBlocks, slugFromInput, toggleFavourite } from "./showcase";
import type { TabId } from "./types";

const GRID: ShowcaseBlock = { type: "grid", slug: "about-me" };
const PLAN_ID = "0e87c223-eeee-4e2c-9a42-62acb51a3f10";

function summary(id: string, name = id): EntitySummary {
    return { kind: "main_story", id, name, icon: null, href: null, facets: {} };
}

/** What `/get-user` sends a visitor for a stored layout with these tabs and blocks. */
function visitorPayload(tabs: [TabId, boolean][], blocks: ShowcaseBlock[]): ProfileLayout {
    const all = normalizeLayout(tabs.map(([id, visible]) => ({ id, visible })));
    const showcaseVisible = all.find((t) => t.id === "showcase")?.visible ?? true;
    return { tabs: all.filter((t) => t.visible), ...(showcaseVisible ? { showcase: { blocks } } : {}) };
}

/** The tab a reader opens on, with nothing picked and Stats remembered (the old default). */
function landing(layout: ProfileLayout | null, isOwner: boolean): TabId | null {
    return resolveActiveTab(shownTabs(layout, isOwner), tabMemory(layout, isOwner), "stats", null);
}

describe("the Showcase tab and the landing tab", () => {
    it("with no layout, gives a visitor no Showcase and the old landing (the kill switch)", () => {
        expect(shownTabs(null, false)).not.toContain("showcase");
        expect(landing(null, false)).toBe("stats");
        // The owner keeps their remembered tab; Showcase is offered, not forced.
        expect(shownTabs(null, true)[0]).toBe("showcase");
        expect(landing(null, true)).toBe("stats");
    });

    it("lands a visitor on a showcase with a block", () => {
        const layout = visitorPayload([], [GRID]);
        expect(shownTabs(layout, false)[0]).toBe("showcase");
        expect(landing(layout, false)).toBe("showcase");
    });

    it("hides an empty showcase from a visitor, who lands on the next tab", () => {
        const layout = visitorPayload([["roster", true]], []);
        expect(shownTabs(layout, false)).not.toContain("showcase");
        expect(landing(layout, false)).toBe("roster");
        // The owner still sees it, to fill it.
        const owner: ProfileLayout = { tabs: normalizeLayout(layout.tabs), showcase: { blocks: [] } };
        expect(shownTabs(owner, true)[0]).toBe("showcase");
    });

    it("hides a private showcase from a visitor whatever it holds", () => {
        const layout = visitorPayload([["showcase", false]], [GRID]);
        expect(layout.showcase).toBeUndefined();
        expect(shownTabs(layout, false)).not.toContain("showcase");
        expect(landing(layout, false)).toBe("stats");
    });

    it("follows the owner's order when they moved Showcase down", () => {
        const layout = visitorPayload(
            [
                ["score", true],
                ["showcase", true],
            ],
            [GRID],
        );
        expect(shownTabs(layout, false).slice(0, 2)).toEqual(["score", "showcase"]);
        expect(landing(layout, false)).toBe("score");
    });
});

describe("blocks", () => {
    const view: ShowcaseView = {
        blocks: [
            {
                block: { type: "favourites", entity_kind: "main_story", ids: ["main_0", "main_9"], title: "Chapters" },
                removed: false,
                entities: [
                    { id: "main_0", entity: summary("main_0", "Prologue"), entity_server: null },
                    { id: "main_9", entity: null, entity_server: null },
                ],
            },
            { block: GRID, removed: false, entities: [] },
            { block: { type: "tier_list", slug: "gone" }, removed: true, entities: [] },
            { block: { type: "plan", id: PLAN_ID }, removed: false, entities: [] },
        ],
    };

    it("shows the owner every block and a visitor none that is removed", () => {
        expect(shownBlocks(view, true)).toHaveLength(4);
        expect(shownBlocks(view, false).map((b) => b.block.type)).toEqual(["favourites", "grid", "plan"]);
    });

    it("drops removed blocks and unknown favourites on save, and counts them for the notice", () => {
        const draft = draftFromView(view);
        expect(droppedOnSave(draft)).toBe(2);
        expect(blocksForSave(draft)).toEqual([{ type: "favourites", entity_kind: "main_story", ids: ["main_0"], title: "Chapters" }, GRID, { type: "plan", id: PLAN_ID }]);
    });

    it("round-trips a clean showcase unchanged", () => {
        const clean: ShowcaseView = { blocks: view.blocks.filter((b) => !b.removed).map((b) => (b.block.type === "favourites" ? { ...b, block: { ...b.block, ids: ["main_0"] }, entities: b.entities.slice(0, 1) } : b)) };
        expect(blocksForSave(draftFromView(clean))).toEqual(clean.blocks.map((b) => b.block));
    });

    it("drops an empty favourites block, cleans titles, keeps the first of a repeated referent", () => {
        const prologue = toTierEntity("main_story", "main_0", summary("main_0"), UNPLACED);
        const titled = { ...newFavourites("main_story"), title: "  Mine\u0007  ", entities: [{ id: "main_0", entity: prologue, server: null }] } as ShowcaseDraftBlock;
        const draft: ShowcaseDraftBlock[] = [newFavourites("operator"), titled, { key: "a", removed: false, type: "grid", slug: "x" }, { key: "b", removed: false, type: "grid", slug: "x" }, { key: "c", removed: false, type: "plan", id: PLAN_ID }, { key: "d", removed: false, type: "plan", id: PLAN_ID.toUpperCase() }];
        // The empty operator block is counted for the notice.
        expect(droppedOnSave(draft)).toBe(1);
        expect(blocksForSave(draft)).toEqual([
            { type: "favourites", entity_kind: "main_story", ids: ["main_0"], title: "Mine" },
            { type: "grid", slug: "x" },
            { type: "plan", id: PLAN_ID },
        ]);
    });

    it("cuts the showcase to its block limit", () => {
        const draft: ShowcaseDraftBlock[] = Array.from({ length: SHOWCASE_MAX_BLOCKS + 3 }, (_, i) => ({ key: String(i), removed: false, type: "grid", slug: `g${i}` }));
        expect(blocksForSave(draft)).toHaveLength(SHOWCASE_MAX_BLOCKS);
    });

    it("cleans titles the way the backend stores them", () => {
        expect(cleanTitle("  a\nb  ")).toBe("ab");
        expect(Array.from(cleanTitle("é".repeat(60)))).toHaveLength(40);
        expect(cleanTitle("<b>x</b>")).toBe("<b>x</b>");
    });

    it("adds a favourite, takes it out on a second pick, and stops at the cap", () => {
        const entity = toTierEntity("main_story", "main_0", summary("main_0"), UNPLACED);
        const once = toggleFavourite([], entity, null);
        expect(once.map((e) => e.id)).toEqual(["main_0"]);
        expect(toggleFavourite(once, entity, null)).toEqual([]);
        const full = Array.from({ length: SHOWCASE_MAX_IDS }, (_, i) => ({ id: `x${i}`, entity: null, server: null }));
        expect(toggleFavourite(full, entity, null)).toHaveLength(SHOWCASE_MAX_IDS);
    });
});

describe("saving", () => {
    it("sends the showcase key alone, so the stored tabs and background are never overwritten", () => {
        const payload = showcaseForSave([GRID]);
        expect(payload).toEqual({ showcase: { blocks: [GRID] } });
        expect(Object.keys(payload)).toEqual(["showcase"]);
    });

    it("sends an empty showcase as an empty block list, never the null that resets the whole layout", () => {
        const payload = showcaseForSave([]);
        expect(payload).not.toBeNull();
        expect(payload).toEqual({ showcase: { blocks: [] } });
    });
});

describe("slugFromInput", () => {
    it("reads a bare slug, a site link and a path", () => {
        expect(slugFromInput("about-me-7v5y09", "grid")).toBe("about-me-7v5y09");
        expect(slugFromInput(" https://myrtle.moe/grids/about-me-7v5y09?x=1#top ", "grid")).toBe("about-me-7v5y09");
        expect(slugFromInput("/tier-lists/meta/", "tier_list")).toBe("meta");
        expect(slugFromInput("http://localhost:3000/tier-lists/%D0%B0-8ydysa", "tier_list")).toBe("а-8ydysa");
    });

    it("refuses a link to the other kind, a blank, or a path with no slug", () => {
        expect(slugFromInput("https://myrtle.moe/tier-lists/meta", "grid")).toBeNull();
        expect(slugFromInput("   ", "grid")).toBeNull();
        expect(slugFromInput("https://myrtle.moe/grids/", "grid")).toBeNull();
        expect(slugFromInput("two words", "grid")).toBeNull();
    });
});
