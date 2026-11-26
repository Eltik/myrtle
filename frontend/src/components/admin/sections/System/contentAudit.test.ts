import { describe, expect, it } from "vitest";
import type { IContentAuditEntry } from "#/lib/api/admin";
import { buildTimeline } from "./audit";
import { actedOnOthers, contentChanges } from "./contentAudit";

const actor = { user_id: "u1", uid: "123", nickname: "shirayuki", secretary: null, secretary_skin_id: null };

function entry(over: Partial<IContentAuditEntry> = {}): IContentAuditEntry {
    return {
        id: 1,
        action: "edit",
        target_kind: "tier_list",
        target_id: "t1",
        target_name: "Meta",
        target_slug: "meta-abc123",
        actor,
        owner: { user_id: "u1", uid: "123", nickname: "shirayuki" },
        subject: null,
        details: {},
        event_count: 1,
        first_at: "2026-10-28T10:00:00Z",
        changed_at: "2026-10-28T10:00:00Z",
        ...over,
    };
}

describe("contentChanges", () => {
    it("words a coalesced edit run in reading order", () => {
        const changes = contentChanges(
            entry({
                details: {
                    renamed_from: "Old meta",
                    tiers_renamed: [{ id: "a", from: "S", to: "S+" }],
                    tiers_reordered: [
                        { id: "a", name: "S+" },
                        { id: "b", name: "A" },
                    ],
                    entries_moved: ["operator:x", "operator:y", "operator:z"],
                    entries_removed: ["operator:w"],
                },
            }),
        );
        expect(changes).toEqual([
            { kind: "renamed", from: "Old meta", to: "Meta" },
            { kind: "tierRenamed", from: "S", to: "S+" },
            { kind: "tiersReordered", count: 2 },
            { kind: "entriesMoved", count: 3 },
            { kind: "entriesRemoved", count: 1 },
        ]);
    });

    it("folds tier renames past the third into one clause", () => {
        const renames = ["A", "B", "C", "D", "E"].map((n, i) => ({ id: String(i), from: n, to: `${n}+` }));
        const changes = contentChanges(entry({ details: { tiers_renamed: renames } }));
        expect(changes.filter((c) => c.kind === "tierRenamed")).toHaveLength(3);
        expect(changes.at(-1)).toEqual({ kind: "more", count: 2 });
    });

    it("reads one-off actions from their own fields only", () => {
        expect(contentChanges(entry({ action: "publish", details: { version: 4, changelog: "nerfs" } }))).toEqual([{ kind: "published", version: 4, changelog: "nerfs" }]);
        expect(contentChanges(entry({ action: "visibility", details: { listed: false } }))).toEqual([{ kind: "listed", listed: false }]);
        expect(contentChanges(entry({ action: "flair", details: {} }))).toEqual([{ kind: "flair", label: null }]);
        expect(contentChanges(entry({ action: "delete", details: { tiers: 6, entries: 80 } }))).toEqual([{ kind: "deleted", tiers: 6, entries: 80 }]);
    });

    it("flags an action on someone else's content", () => {
        expect(actedOnOthers(entry())).toBe(false);
        expect(actedOnOthers(entry({ owner: { user_id: "u2", uid: "9", nickname: "owner" } }))).toBe(true);
        expect(actedOnOthers(entry({ owner: null }))).toBe(false);
    });
});

describe("buildTimeline with content", () => {
    it("merges the content log and holds back rows a missing content page could interleave with", () => {
        const { rows, held } = buildTimeline({
            notes: [],
            translations: [],
            notesHasMore: false,
            translationsHasMore: false,
            content: [entry({ id: 1, changed_at: "2026-10-28T10:00:00Z" }), entry({ id: 2, changed_at: "2026-10-20T10:00:00Z" })],
            contentHasMore: true,
            source: "all",
        });
        expect(rows.map((r) => r.key)).toEqual(["c1", "c2"]);
        expect(rows[0]).toMatchObject({ source: "content", field: "edit" });
        expect(held).toBe(0);
    });

    it("drops content rows from the notes view", () => {
        const { rows } = buildTimeline({ notes: [], translations: [], notesHasMore: false, translationsHasMore: false, content: [entry()], source: "notes" });
        expect(rows).toEqual([]);
    });
});
