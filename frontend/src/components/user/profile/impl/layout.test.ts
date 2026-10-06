import { describe, expect, it } from "vitest";
import type { ProfileLayout } from "#/types/generated/ProfileLayout";
import { defaultLayout, isDefaultLayout, moveEntry, moveTab, normalizeLayout, resolveActiveTab, shownTabs, tabMemory, tabsForSave, toggleTab } from "./layout";
import { TAB_IDS, type TabId } from "./types";

/**
 * The tab bar before layouts existed, copied from the `tabs` memo in
 * `UserProfile.tsx` at 681b35cb: the order every profile showed, all seven tabs,
 * for every reader. The NULL-layout cases below must reproduce it exactly for a
 * visitor; the owner alone also sees the Showcase tab ahead of it.
 */
const BEFORE_LAYOUTS: TabId[] = ["stats", "score", "roster", "plans", "inventory", "enemies", "optimizer"];

function layout(...tabs: [TabId, boolean][]): ProfileLayout {
    return { tabs: tabs.map(([id, visible]) => ({ id, visible })) };
}

/** What `/get-user` sends the owner: the stored layout, normalized. */
function ownerPayload(stored: ProfileLayout): ProfileLayout {
    return { tabs: normalizeLayout(stored.tabs) };
}

/** What it sends a visitor: the same with the private tabs left out. */
function visitorPayload(stored: ProfileLayout): ProfileLayout {
    return { tabs: normalizeLayout(stored.tabs).filter((t) => t.visible) };
}

describe("no layout (the kill switch)", () => {
    it("shows a visitor every tab in the old order, and the owner the same behind Showcase", () => {
        expect(shownTabs(null, false)).toEqual(BEFORE_LAYOUTS);
        // The owner alone sees the (empty) Showcase tab, where they set it up.
        expect(shownTabs(null, true)).toEqual(["showcase", ...BEFORE_LAYOUTS]);
        expect(TAB_IDS).toEqual(["showcase", ...BEFORE_LAYOUTS]);
    });

    it("keeps the remembered tab for everyone, as the page always did", () => {
        expect(tabMemory(null, false)).toBe("stored");
        expect(tabMemory(null, true)).toBe("stored");
        for (const stored of BEFORE_LAYOUTS) {
            // A visitor's in-visit pick is never consulted under the stored rule.
            expect(resolveActiveTab(shownTabs(null, false), "stored", stored, "optimizer")).toBe(stored);
            expect(resolveActiveTab(shownTabs(null, true), "stored", stored, null)).toBe(stored);
        }
    });

    it("opens on Stats with nothing remembered, the old default", () => {
        // `useLocalStorageState("user:profile:tab", "stats")` hands "stats" when empty.
        expect(resolveActiveTab(shownTabs(null, false), "stored", "stats", null)).toBe("stats");
    });
});

describe("a customized layout", () => {
    const stored = layout(["roster", true], ["stats", false], ["score", true], ["plans", false]);
    const asOwner = ownerPayload(stored);
    const asVisitor = visitorPayload(stored);

    it("orders the bar by the layout and hides private tabs from a visitor", () => {
        expect(shownTabs(asVisitor, false)).toEqual(["roster", "score", "inventory", "enemies", "optimizer"]);
    });

    it("shows the owner every tab, private ones included, in layout order", () => {
        expect(shownTabs(asOwner, true)).toEqual(["showcase", "roster", "stats", "score", "plans", "inventory", "enemies", "optimizer"]);
    });

    it("opens a visitor on the owner's first visible tab, not the remembered one", () => {
        expect(tabMemory(asVisitor, false)).toBe("visitor");
        expect(resolveActiveTab(shownTabs(asVisitor, false), "visitor", "score", null)).toBe("roster");
    });

    it("follows a visitor's pick within the visit", () => {
        expect(resolveActiveTab(shownTabs(asVisitor, false), "visitor", "stats", "enemies")).toBe("enemies");
    });

    it("falls back to the first visible tab when the pick is not shown", () => {
        expect(resolveActiveTab(shownTabs(asVisitor, false), "visitor", "stats", "plans")).toBe("roster");
    });

    it("keeps the owner on their remembered tab, even a private one", () => {
        expect(tabMemory(asOwner, true)).toBe("stored");
        expect(resolveActiveTab(shownTabs(asOwner, true), "stored", "plans", null)).toBe("plans");
    });

    it("has no active tab when every tab is private", () => {
        const none = visitorPayload(layout(...BEFORE_LAYOUTS.map((id): [TabId, boolean] => [id, false])));
        // Showcase is visible but empty, so it is no tab to a visitor either.
        expect(none.tabs).toEqual([{ id: "showcase", visible: true }]);
        expect(shownTabs(none, false)).toEqual([]);
        expect(resolveActiveTab(shownTabs(none, false), "visitor", "stats", null)).toBeNull();
    });

    it("never brings back a tab the visitor payload leaves out", () => {
        // Private tabs are absent from a visitor's layout, not flagged; re-normalizing
        // it client-side would append them as visible.
        expect(asVisitor.tabs.map((t) => t.id)).toEqual(["showcase", "roster", "score", "inventory", "enemies", "optimizer"]);
        expect(shownTabs(asVisitor, false)).not.toContain("stats");
        expect(shownTabs(asVisitor, false)).not.toContain("plans");
    });

    it("hides a flagged tab from a visitor even in an owner-shaped payload", () => {
        expect(shownTabs(asOwner, false)).toEqual(["roster", "score", "inventory", "enemies", "optimizer"]);
    });
});

describe("normalizeLayout", () => {
    it("drops unknown ids and repeats, then appends the missing tabs visible", () => {
        const tabs = normalizeLayout([
            { id: "plans", visible: false },
            { id: "achievements", visible: false },
            { id: "plans", visible: true },
        ]);
        // A missing Showcase goes first, as the backend places it; the rest are appended.
        expect(tabs).toEqual([{ id: "showcase", visible: true }, { id: "plans", visible: false }, ...BEFORE_LAYOUTS.filter((id) => id !== "plans").map((id) => ({ id, visible: true }))]);
    });

    it("turns an empty or missing list into the default", () => {
        expect(normalizeLayout([])).toEqual(defaultLayout());
        expect(normalizeLayout(null)).toEqual(defaultLayout());
    });
});

describe("editing", () => {
    it("moves a tab and clamps at the ends", () => {
        const tabs = defaultLayout();
        expect(moveTab(tabs, 1, 1).map((t) => t.id)).toEqual(["showcase", "score", "stats", "roster", "plans", "inventory", "enemies", "optimizer"]);
        expect(moveTab(tabs, 7, -7).map((t) => t.id)).toEqual(["optimizer", "showcase", "stats", "score", "roster", "plans", "inventory", "enemies"]);
        expect(moveTab(tabs, 0, -1)).toEqual(tabs);
        expect(moveTab(tabs, 7, 3)).toEqual(tabs);
    });

    it("says where a moved entry lands, and null when nothing moves", () => {
        const list = ["a", "b", "c", "d"];
        expect(moveEntry(list, 1, 1)).toEqual({ next: ["a", "c", "b", "d"], to: 2 });
        expect(moveEntry(list, 2, -9)).toEqual({ next: ["c", "a", "b", "d"], to: 0 });
        expect(moveEntry(list, 3, 1)).toBeNull();
        expect(moveEntry(list, 0, -1)).toBeNull();
        expect(moveEntry(list, 4, -1)).toBeNull();
        expect(moveEntry(list, -1, 1)).toBeNull();
    });

    it("toggles one tab's visibility", () => {
        const tabs = toggleTab(defaultLayout(), "inventory");
        expect(tabs.find((t) => t.id === "inventory")?.visible).toBe(false);
        expect(tabs.filter((t) => !t.visible)).toHaveLength(1);
    });

    it("recognizes the canonical layout, and only it", () => {
        expect(isDefaultLayout(defaultLayout())).toBe(true);
        expect(isDefaultLayout(moveTab(defaultLayout(), 1, 1))).toBe(false);
        expect(isDefaultLayout(toggleTab(defaultLayout(), "stats"))).toBe(false);
    });
});

describe("saving", () => {
    const GRID = { type: "grid" as const, slug: "about-me" };
    const BACKGROUND = { kind: "skin" as const, id: "char_002_amiya@epoque#4" };

    it("sends the tabs key alone, so the stored showcase and background are never overwritten", () => {
        const tabs = toggleTab(defaultLayout(), "roster");
        const payload = tabsForSave(tabs, false, { tabs, showcase: { blocks: [GRID] }, background: BACKGROUND });
        expect(payload).toEqual({ tabs });
        expect(Object.keys(payload ?? {})).toEqual(["tabs"]);
    });

    it("sends canonical tabs as tabs, not null, when the owner did not press Reset", () => {
        expect(tabsForSave(defaultLayout(), false, null)).toEqual({ tabs: defaultLayout() });
        expect(tabsForSave(defaultLayout(), false, { tabs: defaultLayout(), showcase: { blocks: [] } })).toEqual({ tabs: defaultLayout() });
    });

    it("sends null after Reset only when the reset loses nothing", () => {
        expect(tabsForSave(defaultLayout(), true, null)).toBeNull();
        expect(tabsForSave(defaultLayout(), true, { tabs: defaultLayout(), showcase: { blocks: [] } })).toBeNull();
        // A block or a background is something to keep: Reset then sends the tabs alone.
        expect(tabsForSave(defaultLayout(), true, { tabs: defaultLayout(), showcase: { blocks: [GRID] } })).toEqual({ tabs: defaultLayout() });
        expect(tabsForSave(defaultLayout(), true, { tabs: defaultLayout(), background: BACKGROUND })).toEqual({ tabs: defaultLayout() });
        // Reset, then a change: not the canonical layout, so the tabs.
        const moved = moveTab(defaultLayout(), 1, 1);
        expect(tabsForSave(moved, true, null)).toEqual({ tabs: moved });
    });
});
