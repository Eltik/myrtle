import { DEFAULT_ENTITY_KINDS, type ITierEntity, parseEntityKey, type TierEntityKind, toTierEntity, UNPLACED } from "#/lib/api/tier-entities";
import type { ITierListDetail } from "#/lib/api/tier-lists";
import type { TypedT } from "#/lib/i18n/messages";
import { normalizeHexColor, operatorPlacementNote } from "../shared";
import type { messages as stateMessages } from "./state.messages";

/** The `t` `diffStates` needs, narrowed to the keys it can render. */
export type EditStateT = TypedT<typeof stateMessages>;

const DRAFT_PREFIX = "draft_";

let _draftCounter = 0;
function nextDraftId(): string {
    _draftCounter += 1;
    return `${DRAFT_PREFIX}${Date.now().toString(36)}_${_draftCounter}`;
}

export function isDraftId(id: string): boolean {
    return id.startsWith(DRAFT_PREFIX);
}

export interface IEditTier {
    id: string;
    name: string;
    color: string;
    description: string;
    /** Placed entities in order, by `${kind}:${id}` key (see `entityKey`). */
    entityKeys: string[];
}

export interface IEditState {
    title: string;
    description: string;
    /** The kinds the pool offers, in tab order. Saved with the list's details. Absent in a state built before kinds were editable: operators. */
    entityKinds?: TierEntityKind[];
    tiers: IEditTier[];
    entityByKey: Record<string, ITierEntity>;
    descriptionByKey: Record<string, string>;
}

export type EditAction =
    | { type: "SET_META"; title: string; description: string }
    | { type: "SET_ENTITY_KINDS"; kinds: TierEntityKind[] }
    | { type: "ADD_TIER"; name: string; color: string; description: string }
    | { type: "UPDATE_TIER"; tierId: string; name?: string; color?: string; description?: string }
    | { type: "DELETE_TIER"; tierId: string }
    | { type: "CLEAR_TIER"; tierId: string }
    | { type: "MOVE_TIER"; tierId: string; direction: "up" | "down" }
    /** `entity` is what the key resolves to at drop time, kept so the tile still renders after its kind leaves the pool. */
    | { type: "PLACE_ENTITY"; key: string; tierId: string | null; index?: number; entity?: ITierEntity }
    | { type: "SET_ENTITY_DESCRIPTION"; key: string; description: string }
    | { type: "RESET"; state: IEditState };

/** The ladder: the colour a tier gets from its row position (S red, A orange, ...) when the author has not chosen one. */
const LADDER_TIER_COLORS = ["#dc4d56", "#e0834a", "#d8b54a", "#5dbf86", "#5aa9d9", "#9b73d4", "#8a8a8a"];

export function nextFallbackTierColor(existing: number): string {
    return LADDER_TIER_COLORS[existing % LADDER_TIER_COLORS.length] ?? "#8a8a8a";
}

/**
 * A ladder colour describes the ROW, so a move leaves it on the row and
 * slides the tier out from under it. A colour the author chose belongs to
 * the tier and travels with it.
 */
export function isLadderTierColor(color: string): boolean {
    return LADDER_TIER_COLORS.includes(color.toLowerCase());
}

function sanitizeTierColor(raw: string | null | undefined, index: number): string {
    return normalizeHexColor(raw) ?? nextFallbackTierColor(index);
}

export function detailToEditState(detail: ITierListDetail): IEditState {
    const sortedTiers = [...detail.tiers].sort((a, b) => a.displayOrder - b.displayOrder);
    const entityByKey: Record<string, ITierEntity> = {};
    const descriptionByKey: Record<string, string> = {};
    const tiers: IEditTier[] = sortedTiers.map((t, i) => {
        const sorted = [...t.entities].sort((a, b) => a.subOrder - b.subOrder);
        for (const entity of sorted) {
            entityByKey[entity.key] = entity;
            const blurb = operatorPlacementNote(entity);
            if (blurb) descriptionByKey[entity.key] = blurb;
        }
        return {
            id: t.id,
            name: t.name,
            color: sanitizeTierColor(t.color, i),
            description: t.description ?? "",
            entityKeys: sorted.map((entity) => entity.key),
        };
    });
    return {
        title: detail.title,
        description: detail.description ?? "",
        entityKinds: [...detail.entityKinds],
        tiers,
        entityByKey,
        descriptionByKey,
    };
}

function withoutEntityEverywhere(tiers: IEditTier[], key: string): IEditTier[] {
    return tiers.map((t) => (t.entityKeys.includes(key) ? { ...t, entityKeys: t.entityKeys.filter((k) => k !== key) } : t));
}

export function editReducer(state: IEditState, action: EditAction): IEditState {
    switch (action.type) {
        case "RESET":
            return action.state;
        case "SET_META":
            return { ...state, title: action.title, description: action.description };
        case "SET_ENTITY_KINDS":
            // A list offers at least one kind; the backend refuses an empty set too.
            return action.kinds.length > 0 ? { ...state, entityKinds: [...new Set(action.kinds)] } : state;
        case "ADD_TIER":
            return {
                ...state,
                tiers: [
                    ...state.tiers,
                    {
                        id: nextDraftId(),
                        name: action.name,
                        color: action.color,
                        description: action.description,
                        entityKeys: [],
                    },
                ],
            };
        case "UPDATE_TIER":
            return {
                ...state,
                tiers: state.tiers.map((t) =>
                    t.id === action.tierId
                        ? {
                              ...t,
                              ...(action.name !== undefined ? { name: action.name } : {}),
                              ...(action.color !== undefined ? { color: action.color } : {}),
                              ...(action.description !== undefined ? { description: action.description } : {}),
                          }
                        : t,
                ),
            };
        case "DELETE_TIER":
            return { ...state, tiers: state.tiers.filter((t) => t.id !== action.tierId) };
        case "CLEAR_TIER":
            return {
                ...state,
                tiers: state.tiers.map((t) => (t.id === action.tierId ? { ...t, entityKeys: [] } : t)),
            };
        case "MOVE_TIER": {
            const idx = state.tiers.findIndex((t) => t.id === action.tierId);
            if (idx < 0) return state;
            const swap = action.direction === "up" ? idx - 1 : idx + 1;
            if (swap < 0 || swap >= state.tiers.length) return state;
            const moving = state.tiers[idx];
            const displaced = state.tiers[swap];
            if (!moving || !displaced) return state;
            // Both wearing ladder colours: the colours stay on their rows; either custom: each keeps its own.
            const colorsStayOnRows = isLadderTierColor(moving.color) && isLadderTierColor(displaced.color);
            const next = [...state.tiers];
            next[idx] = colorsStayOnRows ? { ...displaced, color: moving.color } : displaced;
            next[swap] = colorsStayOnRows ? { ...moving, color: displaced.color } : moving;
            return { ...state, tiers: next };
        }
        case "PLACE_ENTITY": {
            const cleared = withoutEntityEverywhere(state.tiers, action.key);
            if (action.tierId === null) return { ...state, tiers: cleared };
            const tiers = cleared.map((t) => {
                if (t.id !== action.tierId) return t;
                const idx = action.index ?? t.entityKeys.length;
                const clamped = Math.max(0, Math.min(idx, t.entityKeys.length));
                const next = [...t.entityKeys];
                next.splice(clamped, 0, action.key);
                return { ...t, entityKeys: next };
            });
            if (!action.entity || state.entityByKey[action.key]) return { ...state, tiers };
            return { ...state, tiers, entityByKey: { ...state.entityByKey, [action.key]: action.entity } };
        }
        case "SET_ENTITY_DESCRIPTION":
            return {
                ...state,
                descriptionByKey: { ...state.descriptionByKey, [action.key]: action.description },
            };
        default:
            return state;
    }
}

/**
 * The tile a placed key renders as. A key nothing resolves (a kind no longer
 * offered and never loaded) gets the unresolved placeholder rather than no
 * tile, so the row's chips always line up with its `entityKeys`.
 */
export function placedEntity(entityByKey: Record<string, ITierEntity>, key: string): ITierEntity {
    const known = entityByKey[key];
    if (known) return known;
    const { kind, id } = parseEntityKey(key);
    return toTierEntity(kind, id, null, UNPLACED);
}

export function placedEntityKeys(state: IEditState): Set<string> {
    const set = new Set<string>();
    for (const t of state.tiers) for (const id of t.entityKeys) set.add(id);
    return set;
}

/** The kinds a state's pool offers. */
export function offeredKinds(state: IEditState): TierEntityKind[] {
    return state.entityKinds && state.entityKinds.length > 0 ? state.entityKinds : [...DEFAULT_ENTITY_KINDS];
}

/** Whether the offered kinds differ, order included (it is the tab order). */
export function kindsChanged(original: IEditState, current: IEditState): boolean {
    const a = offeredKinds(original);
    const b = offeredKinds(current);
    return a.length !== b.length || a.some((k, i) => b[i] !== k);
}

export interface IPendingChange {
    kind: "title-desc" | "entity-kinds" | "tier-create" | "tier-update" | "tier-delete" | "tier-move" | "placement-add" | "placement-remove" | "placement-move" | "placement-desc";
    label: string;
}

/** The trimmed note on one placement, `""` when it has none. */
export function entityDescription(state: IEditState, key: string): string {
    return (state.descriptionByKey[key] ?? "").trim();
}

/** Where each placed key sits: its tier and its index in that tier. */
function placementsByKey(tiers: IEditTier[]): Map<string, { tier: IEditTier; subOrder: number }> {
    const placements = new Map<string, { tier: IEditTier; subOrder: number }>();
    for (const tier of tiers) {
        for (const [i, key] of tier.entityKeys.entries()) placements.set(key, { tier, subOrder: i });
    }
    return placements;
}

export function diffStates(original: IEditState, current: IEditState, t: EditStateT): IPendingChange[] {
    const changes: IPendingChange[] = [];
    if (original.title !== current.title || original.description !== current.description) {
        changes.push({ kind: "title-desc", label: t("edit.change.listDetails") });
    }
    if (kindsChanged(original, current)) {
        changes.push({ kind: "entity-kinds", label: t("edit.change.kinds") });
    }

    const origTierById = new Map(original.tiers.map((t) => [t.id, t] as const));
    const currTierById = new Map(current.tiers.map((t) => [t.id, t] as const));

    for (const tier of current.tiers) {
        if (isDraftId(tier.id)) {
            changes.push({ kind: "tier-create", label: t("edit.change.tierCreated", { name: tier.name }) });
            continue;
        }
        const original = origTierById.get(tier.id);
        if (!original) continue;
        if (original.name !== tier.name || original.color !== tier.color || original.description !== tier.description) {
            changes.push({ kind: "tier-update", label: t("edit.change.tierUpdated", { name: tier.name }) });
        }
    }
    for (const tier of original.tiers) {
        if (!currTierById.has(tier.id)) changes.push({ kind: "tier-delete", label: t("edit.change.tierDeleted", { name: tier.name }) });
    }

    const orderChanged = original.tiers.length === current.tiers.length && original.tiers.some((t, i) => current.tiers[i]?.id !== t.id);
    if (orderChanged) changes.push({ kind: "tier-move", label: t("edit.change.tiersReordered") });

    const origPlacement = placementsByKey(original.tiers);
    const currPlacement = placementsByKey(current.tiers);

    let added = 0;
    let removed = 0;
    let moved = 0;
    let reordered = 0;
    for (const [id, now] of currPlacement) {
        const then = origPlacement.get(id);
        if (!then) {
            added++;
            continue;
        }
        if (then.tier.id !== now.tier.id) moved++;
        else if (then.subOrder !== now.subOrder) reordered++;
    }
    for (const id of origPlacement.keys()) {
        if (!currPlacement.has(id)) removed++;
    }
    let described = 0;
    for (const id of currPlacement.keys()) {
        if (!origPlacement.has(id)) continue;
        if (entityDescription(original, id) !== entityDescription(current, id)) described++;
    }

    if (added) changes.push({ kind: "placement-add", label: t("edit.change.placed", { count: added }) });
    if (moved) changes.push({ kind: "placement-move", label: t("edit.change.moved", { count: moved }) });
    if (reordered) changes.push({ kind: "placement-move", label: t("edit.change.reordered", { count: reordered }) });
    if (removed) changes.push({ kind: "placement-remove", label: t("edit.change.unplaced", { count: removed }) });
    if (described) changes.push({ kind: "placement-desc", label: t("edit.change.described", { count: described }) });

    return changes;
}

/** One placement call the save must make, keyed on the wire's (kind, id). */
export type PlacementChange =
    | { op: "remove"; kind: TierEntityKind; id: string }
    | { op: "move"; kind: TierEntityKind; id: string; tier: IEditTier; subOrder: number }
    | { op: "add"; kind: TierEntityKind; id: string; tier: IEditTier; subOrder: number; description: string }
    | { op: "describe"; kind: TierEntityKind; id: string; description: string };

/**
 * The placement calls that turn `original` into `current`, in the order the
 * save runs them: removals, then moves and reorders, then additions, then
 * note edits on placements that existed before. A placement in a draft tier
 * always moves, since its tier is created first and gets a new id. A
 * placement whose original tier is deleted is re-added instead: the tier
 * delete runs first and cascades it away, so there is nothing left to move.
 * Identity is the `${kind}:${id}` key, so two kinds sharing an id never collide.
 */
export function planPlacementChanges(original: IEditState, current: IEditState): PlacementChange[] {
    const origPlacement = placementsByKey(original.tiers);
    const currPlacement = placementsByKey(current.tiers);

    const currentTierIds = new Set(current.tiers.map((t) => t.id));
    // Keys whose original tier is gone: the save deletes that tier before placements, taking the row with it.
    const orphaned = new Set<string>();
    for (const [key, then] of origPlacement) if (currPlacement.has(key) && !currentTierIds.has(then.tier.id)) orphaned.add(key);

    const changes: PlacementChange[] = [];
    for (const [key, then] of origPlacement) {
        const { kind, id } = parseEntityKey(key);
        const now = currPlacement.get(key);
        if (!now) {
            changes.push({ op: "remove", kind, id });
            continue;
        }
        if (orphaned.has(key)) continue;
        if (!isDraftId(now.tier.id) && now.tier.id === then.tier.id && now.subOrder === then.subOrder) continue;
        changes.push({ op: "move", kind, id, tier: now.tier, subOrder: now.subOrder });
    }
    for (const [key, now] of currPlacement) {
        if (origPlacement.has(key) && !orphaned.has(key)) continue;
        const { kind, id } = parseEntityKey(key);
        changes.push({ op: "add", kind, id, tier: now.tier, subOrder: now.subOrder, description: entityDescription(current, key) });
    }
    for (const key of currPlacement.keys()) {
        if (!origPlacement.has(key) || orphaned.has(key)) continue;
        const next = entityDescription(current, key);
        if (next === entityDescription(original, key)) continue;
        const { kind, id } = parseEntityKey(key);
        changes.push({ op: "describe", kind, id, description: next });
    }
    return changes;
}
