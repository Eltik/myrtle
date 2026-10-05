import { describe, expect, it } from "vitest";
import type { IGrid } from "#/lib/api/grids";
import { toTierEntity, UNPLACED } from "#/lib/api/tier-entities";
import { ABOUT_ME_LABELS, cellPosition } from "./shared";
import { afterSave, contentLostByResize, EMPTY_CELL, gridReducer, gridToState, type IGridEditCell, type IGridEditState, initialPickerKind, isGridDirty, pickerTarget, picksByKind, picksLostByKinds, starterGridInput, starterKinds, toGridInput } from "./state";

/** A rows x cols state whose labels name their own position, `r,c`. */
function labelled(rows: number, cols: number): IGridEditState {
    const cells: IGridEditCell[] = [];
    for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) cells.push({ ...EMPTY_CELL, label: `${r},${c}` });
    }
    return { title: "Grid", description: "", isListed: true, rows, cols, cells, entityKinds: ["operator", "skin"] };
}

const labelsOf = (state: IGridEditState) => state.cells.map((c) => c.label);
const amiya = toTierEntity("operator", "char_002_amiya", { kind: "operator", id: "char_002_amiya", name: "Amiya", icon: "/avatar/char_002_amiya", href: null, facets: { rarity: "5" } }, UNPLACED);

describe("resize keeps every cell at its (row, col)", () => {
    it("growing adds empty cells to the right and below", () => {
        const next = gridReducer(labelled(2, 2), { type: "resize", rows: 3, cols: 3 });
        expect(next.cells).toHaveLength(9);
        expect(labelsOf(next)).toEqual(["0,0", "0,1", "", "1,0", "1,1", "", "", "", ""]);
    });

    it("shrinking drops the cut row and column and keeps the rest in place", () => {
        const next = gridReducer(labelled(3, 3), { type: "resize", rows: 2, cols: 2 });
        expect(labelsOf(next)).toEqual(["0,0", "0,1", "1,0", "1,1"]);
    });

    it("columns and rows change independently", () => {
        const next = gridReducer(labelled(2, 3), { type: "resize", rows: 3, cols: 2 });
        expect(labelsOf(next)).toEqual(["0,0", "0,1", "1,0", "1,1", "", ""]);
    });

    it("clamps to 1..10", () => {
        expect(gridReducer(labelled(2, 2), { type: "resize", rows: 0, cols: 99 })).toMatchObject({ rows: 1, cols: 10 });
        expect(gridReducer(labelled(2, 2), { type: "resize", rows: 0, cols: 99 }).cells).toHaveLength(10);
    });

    it("the same size is a no-op that returns the same state", () => {
        const state = labelled(2, 2);
        expect(gridReducer(state, { type: "resize", rows: 2, cols: 2 })).toBe(state);
    });

    it("counts only the dropped cells that hold a label or a pick", () => {
        const state = labelled(3, 3);
        // Clear the label of (2,2) and give (0,2) a pick with no label.
        state.cells[8] = EMPTY_CELL;
        state.cells[2] = { ...EMPTY_CELL, kind: "operator", id: "char_002_amiya", entity: amiya };
        expect(contentLostByResize(state, 2, 2)).toBe(4); // (0,2) pick, (1,2), (2,0), (2,1); (2,2) is empty
        expect(contentLostByResize(state, 3, 3)).toBe(0);
        expect(contentLostByResize(state, 5, 5)).toBe(0);
    });
});

describe("cell actions", () => {
    it("swap exchanges two whole cells, label and pick together", () => {
        let state = gridReducer(labelled(2, 2), { type: "setEntity", index: 0, entity: amiya });
        state = gridReducer(state, { type: "swap", from: 0, to: 3 });
        expect(labelsOf(state)).toEqual(["1,1", "0,1", "1,0", "0,0"]);
        expect(state.cells[3]?.id).toBe("char_002_amiya");
        expect(state.cells[0]?.id).toBeNull();
    });

    it("swap with itself or out of range is a no-op", () => {
        const state = labelled(2, 2);
        expect(gridReducer(state, { type: "swap", from: 1, to: 1 })).toBe(state);
        expect(gridReducer(state, { type: "swap", from: 1, to: 9 })).toBe(state);
    });

    it("setEntity then clearEntity keeps the label", () => {
        let state = gridReducer(labelled(1, 1), { type: "setEntity", index: 0, entity: amiya });
        expect(state.cells[0]).toMatchObject({ label: "0,0", kind: "operator", id: "char_002_amiya" });
        state = gridReducer(state, { type: "clearEntity", index: 0 });
        expect(state.cells[0]).toEqual({ ...EMPTY_CELL, label: "0,0" });
    });

    it("setLabel caps at 60 characters", () => {
        const next = gridReducer(labelled(1, 1), { type: "setLabel", index: 0, label: "x".repeat(80) });
        expect(next.cells[0]?.label).toHaveLength(60);
    });
});

describe("toGridInput", () => {
    const grid: IGrid = {
        id: "00000000-0000-0000-0000-000000000001",
        slug: "abc",
        title: "My grid",
        description: null,
        rows: 1,
        cols: 3,
        cells: [
            { label: "Favorite", entity_kind: "operator", entity_id: "char_002_amiya", entity: { kind: "operator", id: "char_002_amiya", name: "Amiya", icon: "/avatar/char_002_amiya", href: null, facets: {} } },
            // A pick the served data no longer resolves: kept through the round trip.
            { label: "Gone", entity_kind: "skin", entity_id: "char_x#1", entity: null },
            { label: "", entity_kind: null, entity_id: null, entity: null },
        ],
        is_listed: false,
        // Out of order on purpose: the backend's canonical order is not the tabs' order.
        entity_kinds: ["skin", "operator"],
        kinds_locked: false,
        owner: { id: "u", name: "Doctor" },
        template_of: null,
        fork_count: 0,
        can_edit: true,
        created_at: "2026-10-05T00:00:00Z",
        updated_at: "2026-10-05T00:00:00Z",
    } as IGrid;

    it("round-trips a loaded grid, unresolved picks included", () => {
        const state = gridToState(grid);
        expect(state.cells[1]?.entity?.resolved).toBe(false);
        expect(toGridInput(state)).toEqual({
            title: "My grid",
            description: null,
            rows: 1,
            cols: 3,
            cells: [
                { label: "Favorite", entity_kind: "operator", entity_id: "char_002_amiya" },
                { label: "Gone", entity_kind: "skin", entity_id: "char_x#1" },
                { label: "", entity_kind: null, entity_id: null },
            ],
            is_listed: false,
            entity_kinds: ["operator", "skin"],
        });
    });

    it("a loaded grid is clean until something changes", () => {
        const state = gridToState(grid);
        expect(isGridDirty(state, gridToState(grid))).toBe(false);
        expect(isGridDirty(state, gridReducer(state, { type: "setLabel", index: 2, label: "New" }))).toBe(true);
        // Whitespace the save trims away is not a change.
        expect(isGridDirty(state, gridReducer(state, { type: "setLabel", index: 2, label: "  " }))).toBe(false);
    });

    it("trims the title and labels and sends an empty description as null", () => {
        const state = { ...labelled(1, 1), title: "  Spaced  ", description: "   " };
        state.cells[0] = { ...EMPTY_CELL, label: "  hi " };
        expect(toGridInput(state)).toMatchObject({ title: "Spaced", description: null, cells: [{ label: "hi", entity_kind: null, entity_id: null }] });
    });
});

describe("starterGridInput", () => {
    it("blank is rows x cols empty cells", () => {
        const input = starterGridInput({ title: "x", rows: 3, cols: 4, starter: "blank", kinds: ["operator"] });
        expect(input.cells).toHaveLength(12);
        expect(input.cells.every((c) => c.label === "" && c.entity_kind === null)).toBe(true);
    });

    it("About Me is a 6 x 6 of its 36 labels in row-major order", () => {
        const input = starterGridInput({ title: "x", rows: 2, cols: 2, starter: "about-me", kinds: starterKinds("about-me") });
        expect(ABOUT_ME_LABELS).toHaveLength(36);
        expect([input.rows, input.cols]).toEqual([6, 6]);
        expect(input.cells.map((c) => c.label)).toEqual(ABOUT_ME_LABELS);
        expect(input.cells[6]?.label).toBe('Favorite "cute" design');
    });

    it("preselects operators for blank, operators and skins for About Me", () => {
        expect(starterKinds("blank")).toEqual(["operator"]);
        expect(starterKinds("about-me")).toEqual(["operator", "skin"]);
    });

    it("sends the chosen kinds deduplicated in tab order", () => {
        const input = starterGridInput({ title: "x", rows: 1, cols: 1, starter: "blank", kinds: ["enemy", "operator", "enemy"] });
        expect(input.entity_kinds).toEqual(["operator", "enemy"]);
    });
});

describe("allowed types", () => {
    const skin = toTierEntity("skin", "char_002_amiya@test#1", null, UNPLACED);

    function picked(): IGridEditState {
        let state = labelled(1, 3);
        state = gridReducer(state, { type: "setEntity", index: 0, entity: amiya });
        state = gridReducer(state, { type: "setEntity", index: 2, entity: skin });
        return state;
    }

    it("counts the picks a narrower set would clear", () => {
        const state = picked();
        expect(picksLostByKinds(state, ["operator"])).toBe(1);
        expect(picksLostByKinds(state, ["enemy"])).toBe(2);
        expect(picksLostByKinds(state, ["skin", "operator"])).toBe(0);
    });

    it("removing a kind clears its picks and keeps every label", () => {
        const next = gridReducer(picked(), { type: "setKinds", kinds: ["operator"] });
        expect(next.entityKinds).toEqual(["operator"]);
        expect(next.cells[0]?.id).toBe("char_002_amiya");
        expect(next.cells[2]).toEqual({ ...EMPTY_CELL, label: "0,2" });
        expect(labelsOf(next)).toEqual(["0,0", "0,1", "0,2"]);
    });

    it("adding kinds keeps every pick and orders the set like the tabs", () => {
        const state = picked();
        const next = gridReducer(state, { type: "setKinds", kinds: ["enemy", "skin", "operator", "skin"] });
        expect(next.entityKinds).toEqual(["operator", "skin", "enemy"]);
        expect(next.cells).toEqual(state.cells);
    });

    it("an empty set is refused", () => {
        const state = picked();
        expect(gridReducer(state, { type: "setKinds", kinds: [] })).toBe(state);
    });

    it("a change of kinds is dirty; the same set in another order is not", () => {
        const state = picked();
        expect(isGridDirty(state, gridReducer(state, { type: "setKinds", kinds: ["operator", "skin", "enemy"] }))).toBe(true);
        expect(isGridDirty(state, gridReducer(state, { type: "setKinds", kinds: ["skin", "operator"] }))).toBe(false);
        expect(toGridInput(state).entity_kinds).toEqual(["operator", "skin"]);
    });
});

describe("afterSave", () => {
    it("takes the saved grid when nothing changed during the save", () => {
        const sent = labelled(2, 2);
        const saved = { ...sent, title: "Grid (saved)" };
        expect(afterSave(sent, sent, saved)).toEqual({ original: saved, state: saved });
    });

    it("keeps edits made while the save was in flight, dirty against the new baseline", () => {
        const sent = labelled(2, 2);
        const current = gridReducer(sent, { type: "setLabel", index: 0, label: "edited" });
        const saved = labelled(2, 2);
        const next = afterSave(sent, current, saved);
        expect(next.state).toBe(current);
        expect(next.original).toBe(saved);
        expect(isGridDirty(next.original, next.state)).toBe(true);
    });

    it("cuts a label by code points, never through a surrogate pair", () => {
        const next = gridReducer(labelled(1, 1), { type: "setLabel", index: 0, label: "😀".repeat(70) });
        expect(Array.from(next.cells[0]?.label ?? "")).toHaveLength(60);
        expect(next.cells[0]?.label).toBe("😀".repeat(60));
    });
});

describe("initialPickerKind", () => {
    it("opens on the first allowed type when nothing else applies", () => {
        expect(initialPickerKind(["operator", "skin"], null, null)).toBe("operator");
        expect(initialPickerKind(["skin", "enemy"], null, null)).toBe("skin");
    });

    it("prefers the current pick's type, then the last tab, only when allowed", () => {
        expect(initialPickerKind(["operator", "skin"], "skin", "operator")).toBe("skin");
        expect(initialPickerKind(["operator", "skin"], null, "skin")).toBe("skin");
        expect(initialPickerKind(["operator", "skin"], null, "module")).toBe("operator");
        expect(initialPickerKind(["operator", "skin"], "enemy", "module")).toBe("operator");
    });
});

describe("cellPosition", () => {
    it("names a row-major index by its 1-based row and column", () => {
        expect(cellPosition(0, 3)).toEqual({ row: 1, col: 1 });
        expect(cellPosition(5, 3)).toEqual({ row: 2, col: 3 });
        expect(cellPosition(6, 3)).toEqual({ row: 3, col: 1 });
    });
});

describe("picksByKind", () => {
    it("counts picks per type and skips empty cells", () => {
        const state = labelled(2, 2);
        state.cells[0] = { ...EMPTY_CELL, kind: "operator", id: "char_002_amiya", entity: amiya };
        state.cells[3] = { ...EMPTY_CELL, kind: "operator", id: "char_unknown", entity: null };
        state.cells[1] = { ...EMPTY_CELL, kind: "skin", id: "skin_x", entity: null };
        expect(picksByKind(state.cells)).toEqual({ operator: 2, skin: 1 });
        expect(picksByKind(labelled(1, 1).cells)).toEqual({});
    });
});

describe("pickerTarget", () => {
    it("is null when the picker is closed or the index is off the board", () => {
        expect(pickerTarget(labelled(2, 2), null)).toBeNull();
        expect(pickerTarget(labelled(2, 2), 4)).toBeNull();
    });

    it("describes the cell: position, trimmed label, current pick, and whether Clear applies", () => {
        const state = labelled(2, 3);
        state.cells[4] = { label: "  Fav  ", kind: "operator", id: "char_unknown", entity: null };
        expect(pickerTarget(state, 4)).toEqual({ index: 4, row: 2, col: 2, label: "Fav", current: null, hasPick: true });
        expect(pickerTarget(state, 0)).toEqual({ index: 0, row: 1, col: 1, label: "0,0", current: null, hasPick: false });
    });
});
