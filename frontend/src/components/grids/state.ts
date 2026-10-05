import type { IGrid } from "#/lib/api/grids";
import { type ITierEntity, type TierEntityKind, toTierEntity, UNPLACED } from "#/lib/api/tier-entities";
import { truncateCodePoints } from "#/lib/markdown/sanitize-input";
import type { GridCell } from "#/types/generated/GridCell";
import type { GridInput } from "#/types/generated/GridInput";
import { ABOUT_ME_KINDS, ABOUT_ME_LABELS, ABOUT_ME_SIZE, BLANK_GRID_KINDS, cellPosition, clampGridSize, GRID_LABEL_MAX, type GridStarter, orderKinds } from "./shared";

// The grid editor's state and its reducer. Pure: no React, no server
// functions, so the tests import it directly. A grid is saved WHOLE (one PUT
// of `toGridInput`), so the state is the document itself plus the resolved
// entity of each cell for rendering.

export interface IGridEditCell {
    label: string;
    kind: TierEntityKind | null;
    id: string | null;
    /** What the pick resolves to, for the tile. `null` with a kind and id set means no loaded server knows it: kept, never dropped. */
    entity: ITierEntity | null;
    /** The server `entity` came from when it is not the reader's (an operator only CN has released), for its art. Shown, never saved. */
    server: string | null;
}

export interface IGridEditState {
    title: string;
    description: string;
    isListed: boolean;
    rows: number;
    cols: number;
    /** Exactly `rows * cols`, row-major: index = r * cols + c. */
    cells: IGridEditCell[];
    /** The types a cell may hold: at least one, in `ALL_ENTITY_KINDS` order. Every cell's pick is of one of them. */
    entityKinds: TierEntityKind[];
}

export type GridEditAction =
    | { type: "setTitle"; title: string }
    | { type: "setDescription"; description: string }
    | { type: "setListed"; isListed: boolean }
    | { type: "resize"; rows: number; cols: number }
    | { type: "setLabel"; index: number; label: string }
    /** `server` is where the pick came from when it is not the reader's server, else `null`. */
    | { type: "setEntity"; index: number; entity: ITierEntity; server: string | null }
    | { type: "clearEntity"; index: number }
    | { type: "swap"; from: number; to: number }
    /** Replace the allowed types. Picks of a type no longer allowed are cleared, labels kept. An empty set is refused. */
    | { type: "setKinds"; kinds: readonly TierEntityKind[] }
    | { type: "load"; state: IGridEditState };

export const EMPTY_CELL: IGridEditCell = { label: "", kind: null, id: null, entity: null, server: null };

/** A cell holding something a shrink would throw away: a label or a pick. */
function cellHasContent(cell: IGridEditCell): boolean {
    return cell.label.trim().length > 0 || cell.kind !== null;
}

/** The cells at (r, c) with r >= rows or c >= cols, which a resize to `rows x cols` drops. */
function cellsOutside(state: Pick<IGridEditState, "rows" | "cols" | "cells">, rows: number, cols: number): IGridEditCell[] {
    const out: IGridEditCell[] = [];
    for (let r = 0; r < state.rows; r++) {
        for (let c = 0; c < state.cols; c++) {
            if (r >= rows || c >= cols) out.push(state.cells[r * state.cols + c] ?? EMPTY_CELL);
        }
    }
    return out;
}

/** How many cells with content a resize to `rows x cols` would drop. The editor asks before it does. */
export function contentLostByResize(state: IGridEditState, rows: number, cols: number): number {
    return cellsOutside(state, clampGridSize(rows), clampGridSize(cols)).filter(cellHasContent).length;
}

/** How many picks a change of the allowed types to `kinds` would clear. The editor asks before it does. */
export function picksLostByKinds(state: Pick<IGridEditState, "cells">, kinds: readonly TierEntityKind[]): number {
    return state.cells.filter((cell) => cell.kind !== null && !kinds.includes(cell.kind)).length;
}

/** Every cell keeps its (row, col); new positions are empty, positions outside the new size are dropped. */
function resizeCells(state: IGridEditState, rows: number, cols: number): IGridEditCell[] {
    const cells: IGridEditCell[] = [];
    for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
            const kept = r < state.rows && c < state.cols ? state.cells[r * state.cols + c] : undefined;
            cells.push(kept ?? EMPTY_CELL);
        }
    }
    return cells;
}

function updateCell(state: IGridEditState, index: number, update: (cell: IGridEditCell) => IGridEditCell): IGridEditState {
    const cell = state.cells[index];
    if (!cell) return state;
    const cells = state.cells.slice();
    cells[index] = update(cell);
    return { ...state, cells };
}

export function gridReducer(state: IGridEditState, action: GridEditAction): IGridEditState {
    switch (action.type) {
        case "setTitle":
            return { ...state, title: action.title };
        case "setDescription":
            return { ...state, description: action.description };
        case "setListed":
            return { ...state, isListed: action.isListed };
        case "resize": {
            const rows = clampGridSize(action.rows);
            const cols = clampGridSize(action.cols);
            if (rows === state.rows && cols === state.cols) return state;
            return { ...state, rows, cols, cells: resizeCells(state, rows, cols) };
        }
        case "setLabel":
            return updateCell(state, action.index, (cell) => ({ ...cell, label: truncateCodePoints(action.label, GRID_LABEL_MAX) }));
        case "setEntity":
            return updateCell(state, action.index, (cell) => ({ ...cell, kind: action.entity.kind, id: action.entity.id, entity: action.entity, server: action.server }));
        case "clearEntity":
            return updateCell(state, action.index, (cell) => ({ ...cell, kind: null, id: null, entity: null, server: null }));
        case "swap": {
            const { from, to } = action;
            const a = state.cells[from];
            const b = state.cells[to];
            if (!a || !b || from === to) return state;
            const cells = state.cells.slice();
            cells[from] = b;
            cells[to] = a;
            return { ...state, cells };
        }
        case "setKinds": {
            const kinds = orderKinds(action.kinds);
            if (kinds.length === 0) return state;
            const cells = state.cells.map((cell) => (cell.kind !== null && !kinds.includes(cell.kind) ? { ...cell, kind: null, id: null, entity: null, server: null } : cell));
            return { ...state, entityKinds: kinds, cells };
        }
        case "load":
            return action.state;
    }
}

/** The saved document: what a PUT or POST sends. */
export function toGridInput(state: IGridEditState): GridInput {
    const description = state.description.trim();
    return {
        title: state.title.trim(),
        description: description.length > 0 ? description : null,
        rows: state.rows,
        cols: state.cols,
        cells: state.cells.map((cell) => ({ label: cell.label.trim(), entity_kind: cell.kind, entity_id: cell.id })),
        is_listed: state.isListed,
        entity_kinds: orderKinds(state.entityKinds),
    };
}

/** A stored cell's pick as a tile view model; `null` when it holds none. An id the served data does not know still resolves, to a placeholder. */
function cellEntity(cell: GridCell): ITierEntity | null {
    if (!cell.entity_kind || !cell.entity_id) return null;
    return toTierEntity(cell.entity_kind, cell.entity_id, cell.entity, UNPLACED);
}

/** A loaded grid as editor state. A cell count off from `rows * cols` is padded or cut so the invariant holds. */
export function gridToState(grid: IGrid): IGridEditState {
    const rows = clampGridSize(grid.rows);
    const cols = clampGridSize(grid.cols);
    const cells: IGridEditCell[] = [];
    for (let i = 0; i < rows * cols; i++) {
        const cell = grid.cells[i];
        if (!cell) {
            cells.push(EMPTY_CELL);
            continue;
        }
        const picked = cell.entity_kind !== null && cell.entity_id !== null;
        cells.push({
            label: cell.label,
            kind: picked ? cell.entity_kind : null,
            id: picked ? cell.entity_id : null,
            entity: cellEntity(cell),
            server: cell.entity ? cell.entity_server : null,
        });
    }
    return { title: grid.title, description: grid.description ?? "", isListed: grid.is_listed, rows, cols, cells, entityKinds: orderKinds(grid.entity_kinds) };
}

/** Whether `current` would save anything `original` does not already hold. */
export function isGridDirty(original: IGridEditState, current: IGridEditState): boolean {
    if (original === current) return false;
    return JSON.stringify(toGridInput(original)) !== JSON.stringify(toGridInput(current));
}

/**
 * What the editor holds once a save returns. The saved grid is always the new
 * baseline; it replaces the editor's state only when nothing changed since
 * `sent` went out, so edits made while the save was in flight are kept (and
 * stay dirty against the new baseline).
 */
export function afterSave(sent: IGridEditState, current: IGridEditState, saved: IGridEditState): { original: IGridEditState; state: IGridEditState } {
    return { original: saved, state: isGridDirty(sent, current) ? current : saved };
}

/** How many cells hold a pick of each type, for the types dialog. */
export function picksByKind(cells: readonly IGridEditCell[]): Partial<Record<TierEntityKind, number>> {
    const counts: Partial<Record<TierEntityKind, number>> = {};
    for (const cell of cells) if (cell.kind) counts[cell.kind] = (counts[cell.kind] ?? 0) + 1;
    return counts;
}

/** The cell the entity picker is open for. */
export interface IPickerTarget {
    index: number;
    row: number;
    col: number;
    /** Trimmed; empty when the cell has no label. */
    label: string;
    current: ITierEntity | null;
    /** The cell holds a pick, resolved or not: the picker's Clear is offered. */
    hasPick: boolean;
}

/** The picker's view of the cell at `index`; `null` when the picker is closed or the index is off the board. */
export function pickerTarget(state: Pick<IGridEditState, "cols" | "cells">, index: number | null): IPickerTarget | null {
    if (index === null) return null;
    const cell = state.cells[index];
    if (!cell) return null;
    return { index, ...cellPosition(index, state.cols), label: cell.label.trim(), current: cell.entity, hasPick: cell.kind !== null };
}

/** The tab the entity picker opens on: the current pick's type, else the last tab used, else the first allowed type; each only when the grid allows it. */
export function initialPickerKind(allowed: readonly TierEntityKind[], current: TierEntityKind | null, last: TierEntityKind | null): TierEntityKind {
    for (const kind of [current, last]) {
        if (kind && allowed.includes(kind)) return kind;
    }
    return allowed[0] ?? "operator";
}

/** The allowed types the create dialog preselects for a starter. */
export function starterKinds(starter: GridStarter): TierEntityKind[] {
    return orderKinds(starter === "about-me" ? ABOUT_ME_KINDS : BLANK_GRID_KINDS);
}

/** A new grid's document from the create dialog. The About Me starter is always its own 6 x 6, whatever size was asked for. */
export function starterGridInput(input: { title: string; rows: number; cols: number; starter: GridStarter; kinds: readonly TierEntityKind[] }): GridInput {
    const aboutMe = input.starter === "about-me";
    const rows = aboutMe ? ABOUT_ME_SIZE : clampGridSize(input.rows);
    const cols = aboutMe ? ABOUT_ME_SIZE : clampGridSize(input.cols);
    const labels = aboutMe ? ABOUT_ME_LABELS : [];
    return {
        title: input.title.trim(),
        description: null,
        rows,
        cols,
        cells: Array.from({ length: rows * cols }, (_, i) => ({ label: labels[i] ?? "", entity_kind: null, entity_id: null })),
        is_listed: true,
        entity_kinds: orderKinds(input.kinds),
    };
}
