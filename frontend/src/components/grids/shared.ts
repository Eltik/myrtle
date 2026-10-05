import type { GridSort } from "#/lib/api/grids";
import { ALL_ENTITY_KINDS, DEFAULT_ENTITY_KINDS, type TierEntityKind } from "#/lib/api/tier-entities";

// The limits the backend enforces on a grid, mirrored so the editor refuses
// what a save would be refused for. See the grids contract and
// `backend/src/app/services/grid.rs`.

export const GRID_MIN_SIZE = 1;
export const GRID_MAX_SIZE = 10;
export const GRID_DEFAULT_SIZE = 6;
export const GRID_TITLE_MAX = 100;
export const GRID_DESCRIPTION_MAX = 2000;
export const GRID_LABEL_MAX = 60;
export const GRIDS_PER_USER_MAX = 50;
/** Page size of the browse list; the backend's default. */
export const GRIDS_PER_PAGE = 24;

export function clampGridSize(n: number): number {
    if (!Number.isFinite(n)) return GRID_DEFAULT_SIZE;
    return Math.min(GRID_MAX_SIZE, Math.max(GRID_MIN_SIZE, Math.round(n)));
}

/** A cell's 1-based (row, col) on a board `cols` wide, as the interface names it. */
export function cellPosition(index: number, cols: number): { row: number; col: number } {
    return { row: Math.floor(index / cols) + 1, col: (index % cols) + 1 };
}

/** `6x6`: rows by columns, the way grids.fun names a board. */
export function gridSizeLabel(rows: number, cols: number): string {
    return `${rows}x${cols}`;
}

/** The browse page's search params. */
export interface IGridsSearch {
    sort: GridSort;
    q: string;
    page: number;
}

/** The browse page with no search: the route strips these, and every link back to the list uses them. */
export const DEFAULT_GRIDS_SEARCH: IGridsSearch = { sort: "recent", q: "", page: 1 };

export type GridStarter = "blank" | "about-me";

/** The allowed types a blank grid starts with: the same as a new tier list. */
export const BLANK_GRID_KINDS: readonly TierEntityKind[] = DEFAULT_ENTITY_KINDS;
/** The allowed types the About Me starter starts with: its prompts are about operators and their outfits. */
export const ABOUT_ME_KINDS: readonly TierEntityKind[] = ["operator", "skin"];

/**
 * `kinds` deduplicated, unknown values dropped, in `ALL_ENTITY_KINDS` order:
 * the order every tab and chip lists kinds in. The backend stores its own
 * canonical order; both sides compare kinds as sets, so the two never clash.
 */
export function orderKinds(kinds: Iterable<string>): TierEntityKind[] {
    const set = new Set<string>(kinds);
    return ALL_ENTITY_KINDS.filter((kind) => set.has(kind));
}

/** The About Me starter's rows and columns, whatever size the create dialog asked for. */
export const ABOUT_ME_SIZE = 6;

/**
 * The "About Me (Arknights)" starter, row-major on a 6 x 6 board. These are
 * written into the new grid as its own labels, the same as labels its author
 * typed, so they are stored data rather than interface copy and are not in the
 * message catalog.
 */
export const ABOUT_ME_LABELS: readonly string[] = [
    "Favorite Char Top 1",
    "Favorite Char Top 2",
    "Favorite Char Top 3",
    "Least Favorite Character",
    "Favorite Female",
    "Favorite Male",
    'Favorite "cute" design',
    'Favorite "cool" design',
    'Favorite "hot" design',
    "Favorite in Lore",
    "Favorite Voice",
    "Favorite Skin",
    "Most used Operator",
    "Character you started with",
    'First Character you "hunted" for',
    "Shared Birthday/-month",
    "Favorite Race",
    "Feline or Perro",
    "Favorite Robot",
    "Favorite 3★",
    "Favorite 4★",
    "Favorite 5★",
    "Favorite 6★",
    "Favorite Alter",
    "Favorite Vanguard",
    "Favorite Guard",
    "Favorite Defender",
    "Favorite Medic",
    "Favorite Sniper",
    "Favorite Caster",
    "Favorite Supporter",
    "Favorite Specialist",
    "Favorite NPC female",
    "Favorite NPC male",
    "Main Assistant",
    "Upcoming Char you want",
];
