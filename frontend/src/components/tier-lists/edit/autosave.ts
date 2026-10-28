import { ALL_ENTITY_KINDS, type TierEntityKind } from "#/lib/api/tier-entities";
import { entityDescription, type IEditState, type IEditTier, isDraftId, nextDraftId, offeredKinds } from "./state";

/** Quiet time after the last edit before an auto-save starts. */
export const AUTOSAVE_DEBOUNCE_MS = 3000;

/**
 * Least time between the end of one save and the start of the next. A save
 * is one request per changed tier or placement, and every one of them counts
 * against the same per-address allowance as the rest of the site.
 */
export const AUTOSAVE_MIN_GAP_MS = 5000;

/** How long to wait before the next auto-save may start, in ms, never negative. */
export function autosaveDelay({ now, lastEditAt, lastSaveEndedAt }: { now: number; lastEditAt: number; lastSaveEndedAt: number | null }): number {
    const debounced = lastEditAt + AUTOSAVE_DEBOUNCE_MS;
    const spaced = lastSaveEndedAt === null ? 0 : lastSaveEndedAt + AUTOSAVE_MIN_GAP_MS;
    return Math.max(0, Math.max(debounced, spaced) - now);
}

/**
 * Everything a save would write, as one comparable string. Resolved entity
 * data is left out: it is game data, not the list, and it varies by server.
 * Two states with the same fingerprint save to the same list.
 */
export function editStateFingerprint(state: IEditState): string {
    const notes: Array<[string, string]> = [];
    for (const tier of state.tiers) {
        for (const key of tier.entityKeys) {
            const note = entityDescription(state, key);
            if (note) notes.push([key, note]);
        }
    }
    notes.sort((a, b) => (a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0));
    return JSON.stringify({
        title: state.title,
        description: state.description,
        kinds: offeredKinds(state),
        tiers: state.tiers.map((t) => [t.id, t.name, t.color, t.description, t.entityKeys]),
        notes,
    });
}

/**
 * `state` with each draft tier the save already created renamed to its real
 * id, so a later save moves into that tier instead of creating it again.
 */
export function remapDraftTierIds(state: IEditState, created: ReadonlyMap<string, string>): IEditState {
    if (created.size === 0) return state;
    let changed = false;
    const tiers = state.tiers.map((tier) => {
        const real = created.get(tier.id);
        if (!real) return tier;
        changed = true;
        return { ...tier, id: real };
    });
    return changed ? { ...state, tiers } : state;
}

const AUTOSAVE_PREF_KEY = "myrtle.tierLists.autosave";

/** The browser's auto-save choice. Off when nothing is stored or storage is unavailable. */
export function readAutosavePreference(): boolean {
    try {
        return window.localStorage.getItem(AUTOSAVE_PREF_KEY) === "1";
    } catch {
        return false;
    }
}

export function writeAutosavePreference(on: boolean): void {
    try {
        window.localStorage.setItem(AUTOSAVE_PREF_KEY, on ? "1" : "0");
    } catch {
        // Private mode or blocked storage: the toggle still works for this visit.
    }
}

/** A draft older than this is dropped rather than offered. */
export const DRAFT_MAX_AGE_MS = 30 * 24 * 60 * 60 * 1000;

/** Unsaved edits kept in this browser in case the tab dies before they are saved. */
export interface IStoredDraft {
    savedAt: number;
    /** Fingerprint of the server state the edits were made on top of. */
    baseFingerprint: string;
    state: IEditState;
}

const DRAFT_VERSION = 1;

export function draftStorageKey(listId: string): string {
    return `myrtle.tierLists.draft.${listId}`;
}

/** The draft as stored. Resolved entities are kept only for placed keys, which is all the board renders from it. */
export function serializeDraft(state: IEditState, base: IEditState, now: number): string {
    const entityByKey: IEditState["entityByKey"] = {};
    for (const tier of state.tiers) {
        for (const key of tier.entityKeys) {
            const entity = state.entityByKey[key];
            if (entity) entityByKey[key] = entity;
        }
    }
    return JSON.stringify({ v: DRAFT_VERSION, savedAt: now, baseFingerprint: editStateFingerprint(base), state: { ...state, entityByKey } });
}

const isString = (v: unknown): v is string => typeof v === "string";
const isRecord = (v: unknown): v is Record<string, unknown> => typeof v === "object" && v !== null && !Array.isArray(v);
const KNOWN_KINDS = new Set<string>(ALL_ENTITY_KINDS);

function parseTier(raw: unknown): IEditTier | null {
    if (!isRecord(raw)) return null;
    const { id, name, color, description, entityKeys } = raw;
    if (!isString(id) || !isString(name) || !isString(color) || !isString(description)) return null;
    if (!Array.isArray(entityKeys) || !entityKeys.every(isString)) return null;
    return { id, name, color, description, entityKeys };
}

/** A stored draft, or null for anything malformed, from another version, or too old. */
export function parseDraft(raw: string | null, now: number): IStoredDraft | null {
    if (!raw) return null;
    let data: unknown;
    try {
        data = JSON.parse(raw);
    } catch {
        return null;
    }
    if (!isRecord(data) || data.v !== DRAFT_VERSION) return null;
    const { savedAt, baseFingerprint, state } = data;
    if (typeof savedAt !== "number" || !Number.isFinite(savedAt) || now - savedAt > DRAFT_MAX_AGE_MS) return null;
    if (!isString(baseFingerprint) || !isRecord(state)) return null;
    const { title, description, entityKinds, tiers, entityByKey, descriptionByKey } = state;
    if (!isString(title) || !isString(description) || !Array.isArray(tiers) || !isRecord(entityByKey) || !isRecord(descriptionByKey)) return null;
    const parsedTiers = tiers.map(parseTier);
    if (parsedTiers.some((t) => t === null)) return null;
    const notes: Record<string, string> = {};
    for (const [key, note] of Object.entries(descriptionByKey)) if (isString(note)) notes[key] = note;
    const kinds = Array.isArray(entityKinds) ? (entityKinds.filter((k) => isString(k) && KNOWN_KINDS.has(k)) as TierEntityKind[]) : undefined;
    return {
        savedAt,
        baseFingerprint,
        state: {
            title,
            description,
            ...(kinds && kinds.length > 0 ? { entityKinds: kinds } : {}),
            tiers: parsedTiers as IEditTier[],
            entityByKey: entityByKey as IEditState["entityByKey"],
            descriptionByKey: notes,
        },
    };
}

/**
 * Whether a stored draft is worth offering against what the server holds now.
 * `none`: it saves to what the server already has. `clean`: the server has not
 * changed since the draft's edits began. `diverged`: it has, so restoring and
 * saving would write over whatever changed there.
 */
export function draftOfferKind(draft: IStoredDraft, server: IEditState): "none" | "clean" | "diverged" {
    const serverPrint = editStateFingerprint(server);
    const draftPrint = editStateFingerprint(draft.state);
    if (draftPrint === serverPrint || draftPrint === draft.baseFingerprint) return "none";
    return draft.baseFingerprint === serverPrint ? "clean" : "diverged";
}

/**
 * The editor state a restored draft becomes. A saved tier the server no longer
 * has (deleted since) comes back as a new tier, so saving recreates it rather
 * than moving placements into a tier that is gone. Entities the draft did not
 * keep resolve from the server's state.
 */
export function restorableDraftState(draft: IStoredDraft, server: IEditState): IEditState {
    const serverTierIds = new Set(server.tiers.map((t) => t.id));
    const tiers = draft.state.tiers.map((tier) => (isDraftId(tier.id) || serverTierIds.has(tier.id) ? tier : { ...tier, id: nextDraftId() }));
    return { ...draft.state, tiers, entityByKey: { ...server.entityByKey, ...draft.state.entityByKey } };
}

/** Quiet time before the draft is written, so a drag does not write storage on every frame. */
export const DRAFT_WRITE_DELAY_MS = 800;

export function readDraft(listId: string, now: number): IStoredDraft | null {
    try {
        return parseDraft(window.localStorage.getItem(draftStorageKey(listId)), now);
    } catch {
        return null;
    }
}

export function writeDraft(listId: string, state: IEditState, base: IEditState, now: number): void {
    try {
        window.localStorage.setItem(draftStorageKey(listId), serializeDraft(state, base, now));
    } catch {
        // Quota or blocked storage: the draft is a safety net, not a requirement.
    }
}

export function clearDraft(listId: string): void {
    try {
        window.localStorage.removeItem(draftStorageKey(listId));
    } catch {
        // Nothing to clear when storage is unavailable.
    }
}
