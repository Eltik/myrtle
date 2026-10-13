import { describe, expect, it } from "vitest";
import type { ProfileLayout } from "#/types/generated/ProfileLayout";
import type { IUserProfile } from "#/types/user";
import { addCard, DEFAULT_STRIP, HEADER_FIELDS, headerDraft, headerForSave, isDefaultHeader, isHidden, previewHeader, removeCard, STRIP_CARDS, STRIP_MAX, stripCards, stripGridClass, toggleHidden } from "./header";

/** The strip before it was editable, copied from `StatStrip.tsx` at 4b6a51ea: five cards, these classes. */
const BEFORE = { cards: ["operators", "skins", "items", "orundum", "originite"], grid: "grid-cols-2 sm:grid-cols-3 lg:grid-cols-5" };

function layout(header?: ProfileLayout["header"]): ProfileLayout {
    return header === undefined ? { tabs: [] } : { tabs: [], header };
}

describe("no header (the kill switch)", () => {
    it("renders the old five cards in the old grid for everyone", () => {
        for (const l of [null, layout()]) {
            expect(stripCards(l, false)).toEqual(BEFORE.cards);
            expect(stripCards(l, true)).toEqual(BEFORE.cards);
            for (const field of HEADER_FIELDS) expect(isHidden(l, field)).toBe(false);
        }
        expect(stripGridClass(BEFORE.cards.length)).toBe(BEFORE.grid);
    });

    it("mirrors the backend's lists", () => {
        // `ProfileHeaderField::ALL` and `DEFAULT_STRIP` in profile_layout.rs.
        expect(HEADER_FIELDS).toEqual(["level", "registered", "sign_in", "operators", "skins", "items", "orundum", "originite", "lmd", "headhunting_permits", "ten_pull_permits"]);
        expect(DEFAULT_STRIP).toEqual(BEFORE.cards);
        expect(STRIP_CARDS.length).toBe(HEADER_FIELDS.length - 3);
        expect(STRIP_MAX).toBe(5);
    });
});

describe("a header", () => {
    const owner = layout({ hidden: ["originite"], strip: ["lmd", "originite", "operators"] });

    it("shows the owner every chosen card, hidden ones included, in order", () => {
        expect(stripCards(owner, true)).toEqual(["lmd", "originite", "operators"]);
        expect(isHidden(owner, "originite")).toBe(true);
        expect(isHidden(owner, "lmd")).toBe(false);
    });

    it("never shows a visitor a hidden card, even from an owner-shaped payload", () => {
        expect(stripCards(owner, false)).toEqual(["lmd", "operators"]);
        // What the backend sends a visitor: the strip already resolved without it.
        expect(stripCards(layout({ hidden: ["originite"], strip: ["lmd", "operators"] }), false)).toEqual(["lmd", "operators"]);
        expect(stripCards(layout({ hidden: ["originite"] }), false)).toEqual(["operators", "skins", "items", "orundum"]);
    });

    it("drops ids that are not strip cards", () => {
        expect(stripCards(layout({ hidden: [], strip: ["level", "lmd", "sign_in"] }), true)).toEqual(["lmd"]);
    });

    it("lays out zero to five cards", () => {
        expect(stripCards(layout({ hidden: [], strip: [] }), true)).toEqual([]);
        expect(stripGridClass(1)).toBe("grid-cols-1");
        expect(stripGridClass(2)).toBe("grid-cols-2");
        expect(stripGridClass(3)).toBe("grid-cols-2 sm:grid-cols-3");
        expect(stripGridClass(4)).toBe("grid-cols-2 lg:grid-cols-4");
    });
});

describe("the editor's draft", () => {
    it("starts from the stored header, or the default", () => {
        expect(headerDraft(null)).toEqual({ hidden: [], strip: BEFORE.cards });
        expect(isDefaultHeader(headerDraft(null))).toBe(true);
        expect(headerDraft(layout({ hidden: ["lmd", "level"], strip: ["lmd"] }))).toEqual({ hidden: ["level", "lmd"], strip: ["lmd"] });
    });

    it("saves null at the default, so the stored layout loses the key", () => {
        expect(headerForSave(headerDraft(null))).toBeNull();
        // A strip equal to the default is left out, so it keeps following the default.
        expect(headerForSave({ hidden: ["originite"], strip: [...DEFAULT_STRIP] })).toEqual({ hidden: ["originite"] });
        expect(headerForSave({ hidden: [], strip: [] })).toEqual({ hidden: [], strip: [] });
    });

    it("keeps hidden in canonical order and the strip deduplicated and capped", () => {
        let d = headerDraft(null);
        d = toggleHidden(d, "lmd");
        d = toggleHidden(d, "level");
        expect(d.hidden).toEqual(["level", "lmd"]);
        expect(headerForSave(d)).toEqual({ hidden: ["level", "lmd"] });
        expect(toggleHidden(d, "lmd").hidden).toEqual(["level"]);

        expect(addCard(d, "lmd")).toBe(d); // full at five
        const four = removeCard(d, "items");
        expect(addCard(four, "operators")).toBe(four); // already there
        expect(addCard(four, "level")).toBe(four); // not a card
        expect(addCard(four, "lmd").strip).toEqual(["operators", "skins", "orundum", "originite", "lmd"]);
    });

    it("previews on the owner's profile without touching the rest of the layout", () => {
        const profile = { profile_layout: { tabs: [{ id: "roster", visible: false }], background: { kind: "operator", id: "x" } } } as unknown as IUserProfile;
        const preview = previewHeader(profile, { hidden: ["lmd"], strip: ["lmd"] });
        expect(preview.profile_layout?.tabs).toEqual(profile.profile_layout?.tabs);
        expect(preview.profile_layout?.background).toEqual(profile.profile_layout?.background);
        expect(stripCards(preview.profile_layout ?? null, true)).toEqual(["lmd"]);
        expect(profile.profile_layout?.header).toBeUndefined();
    });
});
