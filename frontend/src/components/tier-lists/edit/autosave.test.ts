import { describe, expect, it } from "vitest";
import { toTierEntity, UNPLACED } from "#/lib/api/tier-entities";
import { AUTOSAVE_DEBOUNCE_MS, AUTOSAVE_MIN_GAP_MS, autosaveDelay, DRAFT_MAX_AGE_MS, draftOfferKind, editStateFingerprint, type IStoredDraft, parseDraft, remapDraftTierIds, restorableDraftState, serializeDraft } from "./autosave";
import { type IEditState, type IEditTier, isDraftId } from "./state";

function tier(id: string, entityKeys: string[] = []): IEditTier {
    return { id, name: id.toUpperCase(), color: "#dc4d56", description: "", entityKeys };
}

function stateOf(tiers: IEditTier[], extra: Partial<IEditState> = {}): IEditState {
    return { title: "t", description: "", tiers, entityByKey: {}, descriptionByKey: {}, ...extra };
}

const NOW = 1_800_000_000_000;

describe("autosaveDelay", () => {
    it("waits out the debounce after the last edit", () => {
        expect(autosaveDelay({ now: NOW, lastEditAt: NOW, lastSaveEndedAt: null })).toBe(AUTOSAVE_DEBOUNCE_MS);
    });

    it("keeps the minimum gap after the previous save when it is the later bound", () => {
        expect(autosaveDelay({ now: NOW, lastEditAt: NOW - AUTOSAVE_DEBOUNCE_MS, lastSaveEndedAt: NOW })).toBe(AUTOSAVE_MIN_GAP_MS);
    });

    it("never goes negative", () => {
        expect(autosaveDelay({ now: NOW, lastEditAt: NOW - 60_000, lastSaveEndedAt: NOW - 60_000 })).toBe(0);
    });
});

describe("editStateFingerprint", () => {
    it("ignores resolved entity data and notes on keys that are not placed", () => {
        const a = stateOf([tier("s", ["operator:char_1"])]);
        const b = stateOf([tier("s", ["operator:char_1"])], {
            entityByKey: { "operator:char_1": toTierEntity("operator", "char_1", null, UNPLACED) },
            descriptionByKey: { "operator:char_2": "not placed" },
        });
        expect(editStateFingerprint(a)).toBe(editStateFingerprint(b));
    });

    it("sees a moved placement, a renamed tier and a changed note", () => {
        const base = stateOf([tier("s", ["operator:char_1"]), tier("a")]);
        const moved = stateOf([tier("s"), tier("a", ["operator:char_1"])]);
        const renamed = stateOf([{ ...tier("s", ["operator:char_1"]), name: "Top" }, tier("a")]);
        const noted = stateOf([tier("s", ["operator:char_1"]), tier("a")], { descriptionByKey: { "operator:char_1": "carry" } });
        const print = editStateFingerprint(base);
        expect(editStateFingerprint(moved)).not.toBe(print);
        expect(editStateFingerprint(renamed)).not.toBe(print);
        expect(editStateFingerprint(noted)).not.toBe(print);
    });

    it("treats a whitespace-only note as no note, like the save does", () => {
        const a = stateOf([tier("s", ["operator:char_1"])]);
        const b = stateOf([tier("s", ["operator:char_1"])], { descriptionByKey: { "operator:char_1": "  " } });
        expect(editStateFingerprint(a)).toBe(editStateFingerprint(b));
    });
});

describe("remapDraftTierIds", () => {
    it("renames created draft tiers and leaves the rest", () => {
        const state = stateOf([tier("s"), tier("draft_x_1", ["operator:char_1"]), tier("draft_y_2")]);
        const next = remapDraftTierIds(state, new Map([["draft_x_1", "real-1"]]));
        expect(next.tiers.map((t) => t.id)).toEqual(["s", "real-1", "draft_y_2"]);
        expect(next.tiers[1]?.entityKeys).toEqual(["operator:char_1"]);
    });

    it("returns the same object when nothing was created", () => {
        const state = stateOf([tier("s")]);
        expect(remapDraftTierIds(state, new Map())).toBe(state);
        expect(remapDraftTierIds(state, new Map([["draft_gone", "real"]]))).toBe(state);
    });
});

describe("stored drafts", () => {
    const server = stateOf([tier("s", ["operator:char_1"]), tier("a")]);
    const edited = stateOf([tier("s"), tier("a", ["operator:char_1"])]);

    it("round-trips through storage", () => {
        const draft = parseDraft(serializeDraft(edited, server, NOW), NOW);
        expect(draft).not.toBeNull();
        expect(draft?.savedAt).toBe(NOW);
        expect(draft?.baseFingerprint).toBe(editStateFingerprint(server));
        expect(editStateFingerprint(draft?.state as IEditState)).toBe(editStateFingerprint(edited));
    });

    it("keeps resolved entities for placed keys only", () => {
        const entity = toTierEntity("operator", "char_1", null, UNPLACED);
        const other = toTierEntity("operator", "char_9", null, UNPLACED);
        const draft = parseDraft(serializeDraft({ ...edited, entityByKey: { "operator:char_1": entity, "operator:char_9": other } }, server, NOW), NOW);
        expect(Object.keys(draft?.state.entityByKey ?? {})).toEqual(["operator:char_1"]);
    });

    it("rejects malformed, foreign-version and expired values", () => {
        expect(parseDraft(null, NOW)).toBeNull();
        expect(parseDraft("{not json", NOW)).toBeNull();
        expect(parseDraft(JSON.stringify({ v: 99, savedAt: NOW, baseFingerprint: "", state: edited }), NOW)).toBeNull();
        expect(parseDraft(JSON.stringify({ v: 1, savedAt: NOW, baseFingerprint: "", state: { ...edited, tiers: [{ id: 3 }] } }), NOW)).toBeNull();
        expect(parseDraft(serializeDraft(edited, server, NOW), NOW + DRAFT_MAX_AGE_MS + 1)).toBeNull();
    });

    it("drops entity kinds it does not know", () => {
        const draft = parseDraft(serializeDraft({ ...edited, entityKinds: ["operator", "nonsense" as never] }, server, NOW), NOW);
        expect(draft?.state.entityKinds).toEqual(["operator"]);
    });

    it("offers a draft made on top of what the server still holds as clean", () => {
        const draft = parseDraft(serializeDraft(edited, server, NOW), NOW) as IStoredDraft;
        expect(draftOfferKind(draft, server)).toBe("clean");
    });

    it("flags a draft whose base the server has moved past as diverged", () => {
        const draft = parseDraft(serializeDraft(edited, server, NOW), NOW) as IStoredDraft;
        const movedOn = stateOf([tier("s", ["operator:char_1"]), tier("a"), tier("b")]);
        expect(draftOfferKind(draft, movedOn)).toBe("diverged");
    });

    it("offers nothing when the server already holds the draft's edits", () => {
        const draft = parseDraft(serializeDraft(edited, server, NOW), NOW) as IStoredDraft;
        expect(draftOfferKind(draft, edited)).toBe("none");
    });

    it("restores a tier deleted on the server as a new tier and resolves entities from the server", () => {
        const entity = toTierEntity("operator", "char_1", null, UNPLACED);
        const serverNow = stateOf([tier("s")], { entityByKey: { "operator:char_1": entity } });
        const draft: IStoredDraft = { savedAt: NOW, baseFingerprint: "", state: stateOf([tier("s"), tier("gone", ["operator:char_1"])]) };
        const restored = restorableDraftState(draft, serverNow);
        expect(restored.tiers[0]?.id).toBe("s");
        expect(isDraftId(restored.tiers[1]?.id ?? "")).toBe(true);
        expect(restored.tiers[1]?.entityKeys).toEqual(["operator:char_1"]);
        expect(restored.entityByKey["operator:char_1"]).toBe(entity);
    });
});
