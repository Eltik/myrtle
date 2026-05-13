import type { ITierListDetail, ITierOperator } from "#/lib/api/tier-lists";

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
    operatorIds: string[];
}

export interface IEditState {
    title: string;
    description: string;
    tiers: IEditTier[];
    operatorById: Record<string, ITierOperator>;
}

export type EditAction =
    | { type: "SET_META"; title: string; description: string }
    | { type: "ADD_TIER"; name: string; color: string; description: string }
    | { type: "UPDATE_TIER"; tierId: string; name?: string; color?: string; description?: string }
    | { type: "DELETE_TIER"; tierId: string }
    | { type: "MOVE_TIER"; tierId: string; direction: "up" | "down" }
    | { type: "PLACE_OPERATOR"; operatorId: string; tierId: string | null; index?: number }
    | { type: "RESET"; state: IEditState };

const DEFAULT_TIER_COLORS = ["#dc4d56", "#e0834a", "#d8b54a", "#5dbf86", "#5aa9d9", "#9b73d4", "#8a8a8a"];

export function nextFallbackTierColor(existing: number): string {
    return DEFAULT_TIER_COLORS[existing % DEFAULT_TIER_COLORS.length] ?? "#8a8a8a";
}

export function detailToEditState(detail: ITierListDetail): IEditState {
    const sortedTiers = [...detail.tiers].sort((a, b) => a.displayOrder - b.displayOrder);
    const operatorById: Record<string, ITierOperator> = {};
    const tiers: IEditTier[] = sortedTiers.map((t, i) => {
        const sortedOps = [...t.operators].sort((a, b) => a.subOrder - b.subOrder);
        for (const op of sortedOps) operatorById[op.id] = op;
        return {
            id: t.id,
            name: t.name,
            color: t.color?.trim() || nextFallbackTierColor(i),
            description: t.description ?? "",
            operatorIds: sortedOps.map((op) => op.id),
        };
    });
    return {
        title: detail.title,
        description: detail.description ?? "",
        tiers,
        operatorById,
    };
}

function withoutOperatorEverywhere(tiers: IEditTier[], operatorId: string): IEditTier[] {
    return tiers.map((t) => (t.operatorIds.includes(operatorId) ? { ...t, operatorIds: t.operatorIds.filter((id) => id !== operatorId) } : t));
}

export function editReducer(state: IEditState, action: EditAction): IEditState {
    switch (action.type) {
        case "RESET":
            return action.state;
        case "SET_META":
            return { ...state, title: action.title, description: action.description };
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
                        operatorIds: [],
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
        case "MOVE_TIER": {
            const idx = state.tiers.findIndex((t) => t.id === action.tierId);
            if (idx < 0) return state;
            const swap = action.direction === "up" ? idx - 1 : idx + 1;
            if (swap < 0 || swap >= state.tiers.length) return state;
            const next = [...state.tiers];
            const a = next[idx];
            const b = next[swap];
            if (!a || !b) return state;
            next[idx] = b;
            next[swap] = a;
            return { ...state, tiers: next };
        }
        case "PLACE_OPERATOR": {
            const cleared = withoutOperatorEverywhere(state.tiers, action.operatorId);
            if (action.tierId === null) return { ...state, tiers: cleared };
            const tiers = cleared.map((t) => {
                if (t.id !== action.tierId) return t;
                const idx = action.index ?? t.operatorIds.length;
                const clamped = Math.max(0, Math.min(idx, t.operatorIds.length));
                const next = [...t.operatorIds];
                next.splice(clamped, 0, action.operatorId);
                return { ...t, operatorIds: next };
            });
            return { ...state, tiers };
        }
        default:
            return state;
    }
}

export function placedOperatorIds(state: IEditState): Set<string> {
    const set = new Set<string>();
    for (const t of state.tiers) for (const id of t.operatorIds) set.add(id);
    return set;
}

export interface IPendingChange {
    kind: "title-desc" | "tier-create" | "tier-update" | "tier-delete" | "tier-move" | "placement-add" | "placement-remove" | "placement-move";
    label: string;
}

export function diffStates(original: IEditState, current: IEditState): IPendingChange[] {
    const changes: IPendingChange[] = [];
    if (original.title !== current.title || original.description !== current.description) {
        changes.push({ kind: "title-desc", label: "List details" });
    }

    const origTierById = new Map(original.tiers.map((t) => [t.id, t] as const));
    const currTierById = new Map(current.tiers.map((t) => [t.id, t] as const));

    for (const t of current.tiers) {
        if (isDraftId(t.id)) {
            changes.push({ kind: "tier-create", label: `New tier "${t.name}"` });
            continue;
        }
        const original = origTierById.get(t.id);
        if (!original) continue;
        if (original.name !== t.name || original.color !== t.color || original.description !== t.description) {
            changes.push({ kind: "tier-update", label: `Updated "${t.name}"` });
        }
    }
    for (const t of original.tiers) {
        if (!currTierById.has(t.id)) changes.push({ kind: "tier-delete", label: `Deleted "${t.name}"` });
    }

    const orderChanged = original.tiers.length === current.tiers.length && original.tiers.some((t, i) => current.tiers[i]?.id !== t.id);
    if (orderChanged) changes.push({ kind: "tier-move", label: "Reordered tiers" });

    const origPlacement = new Map<string, { tierId: string; subOrder: number }>();
    for (const t of original.tiers) {
        for (const [i, id] of t.operatorIds.entries()) origPlacement.set(id, { tierId: t.id, subOrder: i });
    }

    const currPlacement = new Map<string, { tierId: string; subOrder: number }>();
    for (const t of current.tiers) {
        for (const [i, id] of t.operatorIds.entries()) currPlacement.set(id, { tierId: t.id, subOrder: i });
    }

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
        if (then.tierId !== now.tierId) moved++;
        else if (then.subOrder !== now.subOrder) reordered++;
    }
    for (const id of origPlacement.keys()) {
        if (!currPlacement.has(id)) removed++;
    }
    if (added) changes.push({ kind: "placement-add", label: `${added} operator${added === 1 ? "" : "s"} placed` });
    if (moved) changes.push({ kind: "placement-move", label: `${moved} operator${moved === 1 ? "" : "s"} moved` });
    if (reordered) changes.push({ kind: "placement-move", label: `${reordered} operator${reordered === 1 ? "" : "s"} reordered` });
    if (removed) changes.push({ kind: "placement-remove", label: `${removed} operator${removed === 1 ? "" : "s"} unplaced` });

    return changes;
}
