import { describe, expect, it } from "vitest";
import { buildInboxQueue, emptyOperators, type IInboxInput, peopleItems, queueLocales } from "#/components/admin/shell/inbox";
import { currentPercent, missingCount } from "#/components/admin/shell/model";
import { buildFocusCards, mergeRecentChanges, truncateNames } from "./queue";

const locales = [
    { code: "en", enabled: true, is_source: true },
    { code: "ja", enabled: true, is_source: false },
    { code: "ko", enabled: false, is_source: false },
];
const progress = [
    { locale: "ja", total: 100, translated: 90, stale: 5 },
    { locale: "ko", total: 100, translated: 100, stale: 0 },
];
const ops = [
    { id: "char_2", name: "Exusiai", rarity: 6 },
    { id: "char_1", name: "Amiya", rarity: 5 },
    { id: "char_3", name: "Fang", rarity: 3 },
];

describe("locale arithmetic", () => {
    it("counts missing strings and the current share with one decimal", () => {
        expect(missingCount(progress[0])).toBe(10);
        expect(currentPercent(progress[0])).toBe(85);
        expect(currentPercent({ total: 3, translated: 2, stale: 0 })).toBe(66.6);
        expect(currentPercent({ total: 0, translated: 0, stale: 0 })).toBe(100);
    });
});

describe("queueLocales", () => {
    const all = ["en", "ja", "ko"].map((code) => ({ code, level: "admin" as const }));

    it("keeps a super-admin's public non-source locales only", () => {
        expect(queueLocales({ isSuper: true, myLoc: all }, locales)).toEqual(["ja"]);
    });

    it("keeps a translator's hidden locales but drops View grants", () => {
        expect(queueLocales({ isSuper: false, myLoc: [{ code: "ko", level: "edit" }] }, locales)).toEqual(["ko"]);
        expect(queueLocales({ isSuper: false, myLoc: [{ code: "ja", level: "view" }] }, locales)).toEqual([]);
    });
});

describe("emptyOperators", () => {
    it("lists operators without notes by name", () => {
        expect(emptyOperators(ops, new Set(["char_3"])).map((o) => o.id)).toEqual(["char_1", "char_2"]);
    });
});

describe("buildInboxQueue", () => {
    const orphanGrants = [
        { locale: "vn", user_id: "g1", permission: "admin" },
        { locale: "fr", user_id: "g1", permission: "admin" },
        { locale: "kr", user_id: "g2", permission: "edit" },
    ];
    const base: IInboxInput = { staff: true, canNotes: true, canAssign: true, locales: ["ja"], progress, emptyOps: emptyOperators(ops, new Set()), orphanGrants, healthDegraded: true };

    it("orders translations, notes, people, then system", () => {
        expect(buildInboxQueue(base).map((i) => i.kind)).toEqual(["translationsStale", "translationsMissing", "notesEmpty", "orphanLocaleGrants", "systemDegraded"]);
    });

    it("folds every deleted-account grant into one row that carries what revoking needs", () => {
        expect(buildInboxQueue(base).filter((i) => i.kind === "orphanLocaleGrants")).toEqual([{ kind: "orphanLocaleGrants", grants: orphanGrants }]);
    });

    it("names the six-stars among the empty operators and points at the first by name", () => {
        const notes = buildInboxQueue(base).find((i) => i.kind === "notesEmpty");
        expect(notes).toEqual({ kind: "notesEmpty", count: 3, sixStarNames: ["Exusiai"], firstOperatorId: "char_1" });
    });

    it("drops what the role cannot act on, and everything for non-staff", () => {
        expect(buildInboxQueue({ ...base, canAssign: false, healthDegraded: false }).map((i) => i.kind)).toEqual(["translationsStale", "translationsMissing", "notesEmpty"]);
        expect(buildInboxQueue({ ...base, locales: [], emptyOps: [], orphanGrants: [], healthDegraded: false })).toEqual([]);
        expect(buildInboxQueue({ ...base, staff: false })).toEqual([]);
    });
});

describe("peopleItems", () => {
    const grant = { locale: "vn", user_id: "g1", permission: "admin" };

    it("counts actionable rows, not grants or people, and only for a super-admin", () => {
        expect(peopleItems(true, [grant, { ...grant, locale: "fr" }])).toHaveLength(1);
        expect(peopleItems(true, [])).toEqual([]);
        expect(peopleItems(false, [grant])).toEqual([]);
    });
});

describe("buildFocusCards", () => {
    it("gives a translator one card per granted locale with progress", () => {
        const cards = buildFocusCards({
            role: "translator",
            localeGrants: [
                { code: "ja", level: "edit" },
                { code: "ko", level: "view" },
            ],
            progress,
            emptyOps: [],
            totalOps: 0,
            tierLists: [],
        });
        expect(cards).toEqual([
            { kind: "locale", code: "ja", level: "edit", editable: true, stale: 5, missing: 10, percent: 85 },
            { kind: "locale", code: "ko", level: "view", editable: false, stale: 0, missing: 0, percent: 100 },
        ]);
    });

    it("gives an editor a notes card then one card per granted list", () => {
        const cards = buildFocusCards({
            role: "tier_list_editor",
            localeGrants: [],
            progress: [],
            emptyOps: emptyOperators(ops, new Set(["char_1"])),
            totalOps: 3,
            tierLists: [
                { slug: "a", title: "A", permission: "edit" },
                { slug: "b", title: "B", permission: "publish" },
            ],
        });
        expect(cards.map((c) => (c.kind === "tierList" ? [c.kind, c.canPublish] : [c.kind]))).toEqual([["notes"], ["tierList", false], ["tierList", true]]);
        expect(cards[0]).toMatchObject({ empty: 2, total: 3, sixStarNames: ["Exusiai"], firstOperatorId: "char_2" });
    });

    it("gives staff no focus cards", () => {
        expect(buildFocusCards({ role: "super_admin", localeGrants: [], progress, emptyOps: ops, totalOps: 3, tierLists: [] })).toEqual([]);
    });
});

describe("mergeRecentChanges", () => {
    const actor = (user_id: string, nickname: string | null) => ({ user_id, nickname, uid: null });
    const notes = [
        { id: 1, operator_id: "char_1", field_name: "pros", changed_at: "2026-10-03T00:00:00Z", actor: actor("a", "Ann") },
        { id: 2, operator_id: "char_2", field_name: "tags", changed_at: "2026-10-01T00:00:00Z", actor: actor("b", null) },
    ];
    const translations = [{ id: 7, message_key: "home.title", locale: "ja", changed_at: "2026-10-02T00:00:00Z", actor: actor("b", "Bo") }];

    it("merges both feeds newest first and caps the length", () => {
        expect(mergeRecentChanges(notes, translations).map((r) => r.id)).toEqual(["n1", "t7", "n2"]);
        expect(mergeRecentChanges(notes, translations, { limit: 2 })).toHaveLength(2);
    });

    it("keeps one actor's rows and leaves a deleted actor nameless", () => {
        const own = mergeRecentChanges(notes, translations, { onlyActorId: "b" });
        expect(own.map((r) => [r.id, r.actorName])).toEqual([
            ["t7", "Bo"],
            ["n2", null],
        ]);
    });
});

describe("truncateNames", () => {
    it("shows the first few and counts the rest", () => {
        expect(truncateNames(["a", "b", "c", "d", "e"])).toEqual({ shown: ["a", "b", "c"], rest: 2 });
        expect(truncateNames(["a"])).toEqual({ shown: ["a"], rest: 0 });
    });
});
