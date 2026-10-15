import { describe, expect, it } from "vitest";
import { changeKind } from "#/components/admin/shell/model";
import type { IOperatorNote } from "#/lib/api/operator-notes";
import { activeFilterCount, addTag, draftFromNote, draftHasContent, draftStatus, draftsEqual, filterRows, type INoteFilters, joinRows, missingFields, nextUnfinishedOperator, statusCounts, tagsInUse, toUpdateInput, upsertNote } from "./notesModel";

function note(operatorId: string, patch: Partial<IOperatorNote> = {}): IOperatorNote {
    return { id: `n-${operatorId}`, operator_id: operatorId, pros: null, cons: null, notes: null, trivia: null, summary: null, tags: null, created_at: "2026-01-01T00:00:00Z", updated_at: "2026-01-01T00:00:00Z", ...patch };
}

const ops = [
    { id: "char_103_angel", name: "Exusiai", rarity: 6 },
    { id: "char_002_amiya", name: "Amiya", rarity: 5 },
    { id: "char_102_texas", name: "Texas", rarity: 5 },
    { id: "char_263_skadi", name: "Skadi", rarity: 6 },
    { id: "char_4009_frstn3", name: "Friston-3", rarity: 5 },
    { id: "char_017_huang", name: "Blaze", rarity: 6 },
];

const full = { summary: "Burst caster", pros: "Fast", cons: "Fragile", notes: "Use S2", trivia: "Likes rain" };

const notes = [
    note("char_103_angel", { summary: "Anti-air sniper", tags: ["Anti-air", "DPS"], updated_at: "2026-03-01T00:00:00Z" }),
    note("char_102_texas", { tags: [] }),
    note("char_263_skadi", { pros: "  ", tags: ["Physical"], updated_at: "2026-05-01T00:00:00Z" }),
    note("char_4009_frstn3", { trivia: "Built by Rhodes" }),
    note("char_017_huang", { ...full, tags: ["DPS"] }),
];

const compare = (a: string, b: string) => a.localeCompare(b);
const base: INoteFilters = { status: "all", q: "", rarities: [], tags: [], sort: "name" };

describe("joinRows", () => {
    it("keeps every operator: empty with nothing written, done with all five fields, incomplete in between", () => {
        const rows = joinRows(ops, notes);
        expect(rows.map((r) => [r.id, r.status])).toEqual([
            ["char_103_angel", "incomplete"],
            ["char_002_amiya", "empty"],
            ["char_102_texas", "empty"],
            ["char_263_skadi", "incomplete"],
            ["char_4009_frstn3", "incomplete"],
            ["char_017_huang", "filled"],
        ]);
        expect(statusCounts(rows)).toEqual({ all: 6, empty: 2, incomplete: 3, filled: 1 });
    });
});

describe("filterRows", () => {
    const rows = joinRows(ops, notes);
    it("filters by status, rarity and tags (OR) and sorts", () => {
        expect(filterRows(rows, base, compare).map((r) => r.name)).toEqual(["Amiya", "Blaze", "Exusiai", "Friston-3", "Skadi", "Texas"]);
        expect(filterRows(rows, { ...base, status: "empty" }, compare).map((r) => r.name)).toEqual(["Amiya", "Texas"]);
        expect(filterRows(rows, { ...base, status: "incomplete" }, compare).map((r) => r.name)).toEqual(["Exusiai", "Friston-3", "Skadi"]);
        expect(filterRows(rows, { ...base, status: "filled" }, compare).map((r) => r.name)).toEqual(["Blaze"]);
        expect(filterRows(rows, { ...base, rarities: [6] }, compare).map((r) => r.name)).toEqual(["Blaze", "Exusiai", "Skadi"]);
        expect(filterRows(rows, { ...base, tags: ["DPS", "Physical"] }, compare).map((r) => r.name)).toEqual(["Blaze", "Exusiai", "Skadi"]);
        expect(filterRows(rows, { ...base, sort: "rarity" }, compare).map((r) => r.name)).toEqual(["Blaze", "Exusiai", "Skadi", "Amiya", "Friston-3", "Texas"]);
        expect(filterRows(rows, { ...base, sort: "recent" }, compare).map((r) => r.name)[0]).toBe("Skadi");
    });
    it("searches name, id and tags through the shared normalization", () => {
        expect(filterRows(rows, { ...base, q: "EXUS" }, compare).map((r) => r.name)).toEqual(["Exusiai"]);
        expect(filterRows(rows, { ...base, q: "char_002" }, compare).map((r) => r.name)).toEqual(["Amiya"]);
        expect(filterRows(rows, { ...base, q: "anti" }, compare).map((r) => r.name)).toEqual(["Exusiai"]);
    });
    it("lists tags in use and counts panel filters", () => {
        expect(tagsInUse(rows, compare)).toEqual(["Anti-air", "DPS", "Physical"]);
        expect(activeFilterCount({ rarities: [6, 5], tags: ["DPS"], sort: "recent" })).toBe(4);
        expect(activeFilterCount({ rarities: [], tags: [], sort: "name" })).toBe(0);
    });
});

describe("nextUnfinishedOperator", () => {
    const rows = joinRows(ops, notes);
    it("picks the next operator that isn't done after the current one, wrapping, skipping done ones", () => {
        expect(nextUnfinishedOperator(rows, "char_002_amiya")?.id).toBe("char_102_texas");
        expect(nextUnfinishedOperator(rows, "char_102_texas")?.id).toBe("char_263_skadi");
        expect(nextUnfinishedOperator(rows, "char_4009_frstn3")?.id).toBe("char_103_angel");
        expect(nextUnfinishedOperator(rows, "char_017_huang")?.id).toBe("char_103_angel");
    });
    it("is null when no other unfinished operator is left", () => {
        expect(nextUnfinishedOperator([rows[1], rows[5]], "char_002_amiya")).toBeNull();
    });
});

describe("draftStatus", () => {
    it("counts the five text fields, whitespace as blank, tags as content only for Empty", () => {
        const blank = draftFromNote(null);
        expect(draftStatus(blank)).toBe("empty");
        expect(draftStatus({ ...blank, pros: "  \n " })).toBe("empty");
        expect(draftStatus({ ...blank, tags: ["DPS"] })).toBe("incomplete");
        expect(draftStatus({ ...blank, trivia: "x" })).toBe("incomplete");
        expect(draftStatus({ ...blank, ...full })).toBe("filled");
        expect(draftStatus({ ...blank, ...full, cons: " " })).toBe("incomplete");
    });
    it("lists the blank fields in form order", () => {
        expect(missingFields({ ...draftFromNote(null), trivia: "x" })).toEqual(["summary", "pros", "cons", "notes"]);
        expect(missingFields({ ...draftFromNote(null), ...full, pros: "", notes: " " })).toEqual(["pros", "notes"]);
        expect(missingFields({ ...draftFromNote(null), ...full })).toEqual([]);
    });
});

describe("tags", () => {
    it("trims, cuts to 32, dedupes and stops at 16", () => {
        expect(addTag(["A"], "  B ")).toEqual(["A", "B"]);
        expect(addTag(["A"], "A")).toEqual(["A"]);
        expect(addTag(["A"], "   ")).toEqual(["A"]);
        expect(addTag([], "x".repeat(40))).toEqual(["x".repeat(32)]);
        const full = Array.from({ length: 16 }, (_, i) => `t${i}`);
        expect(addTag(full, "new")).toBe(full);
    });
});

describe("drafts", () => {
    it("round-trips a note and detects edits", () => {
        const d = draftFromNote(notes[0]);
        expect(draftsEqual(d, draftFromNote(notes[0]))).toBe(true);
        expect(draftsEqual(d, { ...d, tags: ["Anti-air"] })).toBe(false);
        expect(draftHasContent(draftFromNote(null))).toBe(false);
        expect(toUpdateInput("x", { ...draftFromNote(null), summary: "  hi " })).toEqual({ operatorId: "x", summary: "hi", pros: null, cons: null, notes: null, trivia: null, tags: [] });
    });
    it("upserts a saved note into the cached list", () => {
        const saved = note("char_002_amiya", { summary: "Free caster" });
        expect(upsertNote(notes, saved)).toHaveLength(6);
        expect(upsertNote(upsertNote(notes, saved), saved)).toHaveLength(6);
    });
});

describe("changeKind", () => {
    it("names additions, clears and changes", () => {
        expect(changeKind(null, "x")).toBe("added");
        expect(changeKind("x", null)).toBe("cleared");
        expect(changeKind("x", "y")).toBe("changed");
    });
});
