/**
 * The Squad Builder page's state: the working copy, which slot it belongs to,
 * the last saved version of that slot, and the page's own settings. A pure
 * reducer, so seat moves and the dirty check are tested without React.
 */

import type { SavedSquad } from "#/types/generated/SavedSquad";
import type { SquadMember } from "#/types/generated/SquadMember";
import { DEFAULT_START_DP } from "./constants";
import { draftsEqual, emptyDraft, type ISquadDraft, NAME_MAX_CHARS, normalizeMembers, SQUAD_SIZE } from "./model";

export type StartDpFrom = { kind: "stage"; code: string } | { kind: "page" };

/** The working copy as it was before a shared squad was opened over it. */
interface IStash {
    draft: ISquadDraft;
    slot: number | null;
    savedBaseline: ISquadDraft | null;
}

export interface ISquadPageState {
    draft: ISquadDraft;
    /** 1-4, or `null` for the scratch squad. */
    slot: number | null;
    /** What the active slot holds on the server; `null` for scratch or an empty slot. */
    savedBaseline: ISquadDraft | null;
    startDp: number;
    startDpFrom: StartDpFrom;
    compact: boolean;
    /** A shared squad is on screen, read only, over the stashed working copy. */
    readOnlyShare: boolean;
    stash: IStash | null;
}

export { DEFAULT_START_DP } from "./constants";

export const INITIAL_STATE: ISquadPageState = {
    draft: emptyDraft(),
    slot: null,
    savedBaseline: null,
    startDp: DEFAULT_START_DP,
    startDpFrom: { kind: "page" },
    compact: false,
    readOnlyShare: false,
    stash: null,
};

export type SquadAction =
    | { type: "HYDRATE"; draft: ISquadDraft; slot: number | null; savedBaseline: ISquadDraft | null; startDp: number; startDpFrom: StartDpFrom; compact: boolean }
    | { type: "SET_NAME"; name: string }
    | { type: "SET_MEMBER"; seat: number; member: SquadMember | null }
    | { type: "SET_SUPPORT"; member: SquadMember | null }
    | { type: "SWAP"; a: number; b: number }
    | { type: "CLEAR" }
    | { type: "REVERT" }
    | { type: "LOAD"; draft: ISquadDraft; slot: number | null; savedBaseline: ISquadDraft | null }
    | { type: "SAVED"; slot: number; draft: ISquadDraft }
    | { type: "SLOT_DELETED" }
    | { type: "SET_STAGE"; stageId: string | null }
    | { type: "SET_START_DP"; startDp: number; from: StartDpFrom }
    | { type: "SET_COMPACT"; compact: boolean }
    | { type: "OPEN_SHARE"; draft: ISquadDraft }
    | { type: "CLOSE_SHARE" }
    | { type: "KEEP_SHARE" };

/** Swap two seats; either may be empty. Out-of-range or equal seats leave the list as it was. */
export function swapSeats(members: ReadonlyArray<SquadMember | null>, a: number, b: number): Array<SquadMember | null> {
    const next = normalizeMembers(members);
    if (a === b || a < 0 || b < 0 || a >= SQUAD_SIZE || b >= SQUAD_SIZE) return next;
    [next[a], next[b]] = [next[b], next[a]];
    return next;
}

/** The first empty seat after `from` (wrapping), or `null` when the squad is full. */
export function nextEmptySeat(members: ReadonlyArray<SquadMember | null>, from: number): number | null {
    for (let i = 1; i <= SQUAD_SIZE; i++) {
        const seat = (from + i) % SQUAD_SIZE;
        if (!members[seat]) return seat;
    }
    return null;
}

/** Unsaved changes against the slot's saved version; a scratch squad is dirty once it holds anything. */
export function isDirty(state: Pick<ISquadPageState, "draft" | "savedBaseline">): boolean {
    return !draftsEqual(state.draft, state.savedBaseline ?? emptyDraft());
}

function isMember(v: unknown): v is SquadMember {
    if (!v || typeof v !== "object") return false;
    const m = v as SquadMember;
    return typeof m.char_id === "string" && typeof m.skill_index === "number" && (m.module_id === null || typeof m.module_id === "string") && (m.build === null || typeof m.build === "object");
}

/** A stored draft, or `null` when what is stored is not one. Unreadable seats come back empty. */
export function parseStoredDraft(raw: unknown): ISquadDraft | null {
    if (!raw || typeof raw !== "object") return null;
    const d = raw as Partial<ISquadDraft>;
    if (!Array.isArray(d.members)) return null;
    return {
        name: typeof d.name === "string" ? d.name.slice(0, NAME_MAX_CHARS) : "",
        members: normalizeMembers(d.members.map((m) => (isMember(m) ? m : null))),
        support: isMember(d.support) ? d.support : null,
        stageId: typeof d.stageId === "string" ? d.stageId : null,
    };
}

export function draftFromSaved(saved: SavedSquad): ISquadDraft {
    return { name: saved.name, members: normalizeMembers(saved.members), support: saved.support, stageId: saved.stage_id };
}

export function squadReducer(state: ISquadPageState, action: SquadAction): ISquadPageState {
    const edit = (draft: ISquadDraft): ISquadPageState => (state.readOnlyShare ? state : { ...state, draft });
    switch (action.type) {
        case "HYDRATE":
            return { ...state, draft: action.draft, slot: action.slot, savedBaseline: action.savedBaseline, startDp: action.startDp, startDpFrom: action.startDpFrom, compact: action.compact };
        case "SET_NAME":
            return edit({ ...state.draft, name: action.name.slice(0, NAME_MAX_CHARS) });
        case "SET_MEMBER": {
            if (action.seat < 0 || action.seat >= SQUAD_SIZE) return state;
            const members = normalizeMembers(state.draft.members);
            members[action.seat] = action.member;
            return edit({ ...state.draft, members });
        }
        case "SET_SUPPORT":
            return edit({ ...state.draft, support: action.member });
        case "SWAP":
            return edit({ ...state.draft, members: swapSeats(state.draft.members, action.a, action.b) });
        case "CLEAR":
            return edit({ ...emptyDraft(state.draft.name), stageId: state.draft.stageId });
        case "REVERT":
            return edit(state.savedBaseline ?? emptyDraft());
        case "LOAD":
            return { ...state, draft: action.draft, slot: action.slot, savedBaseline: action.savedBaseline, readOnlyShare: false, stash: null };
        case "SAVED":
            return { ...state, draft: action.draft, slot: action.slot, savedBaseline: action.draft, readOnlyShare: false, stash: null };
        case "SLOT_DELETED":
            return { ...state, slot: null, savedBaseline: null };
        case "SET_STAGE":
            return edit({ ...state.draft, stageId: action.stageId });
        case "SET_START_DP":
            return { ...state, startDp: Math.max(0, Math.round(action.startDp)), startDpFrom: action.from };
        case "SET_COMPACT":
            return { ...state, compact: action.compact };
        case "OPEN_SHARE":
            return {
                ...state,
                draft: action.draft,
                readOnlyShare: true,
                stash: state.stash ?? { draft: state.draft, slot: state.slot, savedBaseline: state.savedBaseline },
                slot: null,
                savedBaseline: null,
            };
        case "CLOSE_SHARE":
            if (!state.stash) return { ...state, readOnlyShare: false };
            return { ...state, ...state.stash, readOnlyShare: false, stash: null };
        case "KEEP_SHARE":
            return { ...state, readOnlyShare: false, stash: null, slot: null, savedBaseline: null };
    }
}
