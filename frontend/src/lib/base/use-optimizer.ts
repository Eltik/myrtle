import { useMutation, useQuery } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useDebounce } from "#/hooks/use-debounce";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { baseCatalogQueryOptions, baseLayoutQueryOptions, evaluateLayoutQueryOptions, type FacilityType, type ICatalogFormula, type ICatalogRoom, type ICatalogSlot, type IDraftRoom, type IOptimizeResponse, type IRotationResponse, type ISlotPresets, optimizeLayoutFn, rotationPlanFn } from "#/lib/api/base";
import type { IShiftRoom } from "#/lib/api/user";
import { type Catalog, isPlannable, seatsOf } from "./layout";

/**
 * Planner state.
 *
 * The draft layout lives here and in localStorage; everything derived from it -
 * efficiencies, yields, morale endurance, proposals - comes from the backend.
 * There is deliberately no local scoring: the board shows what the clause
 * engine says, or it shows the previous answer while a new one is in flight.
 */

export interface IOptimizerApi {
    /** The layout being edited. */
    layout: IDraftRoom[];
    /** True once the player has diverged from their real in-game base. */
    dirty: boolean;
    catalog: Catalog;
    /** The floorplan every base shares - the board's geometry. */
    slots: ICatalogSlot[];
    /** The factory recipes, named by the game rather than by us. */
    formulas: ICatalogFormula[];
    /** The player's own saved shift rotation, per slot. Empty if they never set one. */
    presets: ISlotPresets[];
    /**
     * Shifts in a base day, from the engine. Not derived from the player's
     * queues: rooms queue independently and most queue fewer presets than there
     * are shifts, so counting them would hide a shift that genuinely exists.
     */
    shiftCount: number;
    /**
     * Every room the board draws: the player's real base with the draft laid
     * over it. Wider than `layout`, which drops the structure before the engine
     * sees it - the board still has to label a corridor.
     */
    boardRooms: IDraftRoom[];
    catalogLoading: boolean;
    /** True while the player's real base is still being fetched. */
    layoutLoading: boolean;

    selectedSlotId: string | null;
    setSelectedSlotId: (slotId: string | null) => void;
    selectedRoom: IDraftRoom | null;

    /** Scored draft. `undefined` until the first evaluation returns. */
    evaluation: ReturnType<typeof useEvaluation>["data"];
    evaluating: boolean;
    evaluationError: Error | null;

    assign: (slotId: string, operatorId: string) => void;
    unassign: (slotId: string, operatorId: string) => void;
    clearRoom: (slotId: string) => void;
    setFormula: (slotId: string, formula: IDraftRoom["formula_type"]) => void;
    setLevel: (slotId: string, level: number) => void;
    /** Where an operator currently sits in the draft, if anywhere. */
    slotOf: (operatorId: string) => string | null;
    seatsFor: (room: IDraftRoom) => number;

    lockedIds: string[];
    toggleLocked: (operatorId: string) => void;
    excludedIds: string[];
    toggleExcluded: (operatorId: string) => void;

    rotation: IRotationResponse | null;
    rotating: boolean;
    rotationError: Error | null;
    /**
     * Which shift the board is showing. `null` = the draft being edited; a
     * number = that shift of the rotation, read-only.
     */
    viewShift: number | null;
    setViewShift: (shift: number | null) => void;
    /** The rotation's staffing for `slotId` in the viewed shift, if any. */
    shiftRoom: (slotId: string) => IShiftRoom | undefined;

    proposal: IOptimizeResponse | null;
    optimizing: boolean;
    optimizeError: Error | null;
    runOptimize: (scope: string[]) => void;
    acceptRoom: (slotId: string) => void;
    acceptAll: () => void;
    /**
     * Throw away the optimizer's answer and go back to the player's own base.
     * One run produces both a staffing and the rotation that services it, so
     * both go together - dropping only the staffing would leave the board
     * showing planned shifts with no plan behind them.
     */
    discardPlan: () => void;

    /** Return the draft to the player's real in-game base. */
    reset: () => void;
}

interface IPersisted {
    layout: IDraftRoom[];
    locked: string[];
    excluded: string[];
}

/**
 * Scoped per profile: planning against two different rosters must not share a
 * draft, and a stale draft from another player would be nonsense.
 */
// v2: v1 drafts were seeded from `improvements.base.current`, a production-only
// view that omitted dormitories and power plants. Those drafts are wrong at the
// root, so they are abandoned rather than migrated.
/** Stable identities, so an unloaded catalogue does not re-derive the board every render. */
const EMPTY_SLOTS: ICatalogSlot[] = [];
const EMPTY_FORMULAS: ICatalogFormula[] = [];
const EMPTY_PRESETS: ISlotPresets[] = [];

function storageKey(uid: string): string {
    return `base-optimizer:${uid}:v2`;
}

function useEvaluation(uid: string, layout: IDraftRoom[]) {
    // Editing is bursty - a drag lands several state updates in a row. Debounce
    // so a single settled arrangement is scored, not each keystroke of it.
    const settled = useDebounce(layout, 350);
    return useQuery(evaluateLayoutQueryOptions(uid, settled));
}

export function useOptimizer(uid: string): IOptimizerApi {
    // The player's REAL base - every built slot. Not `improvements.base.current`,
    // which carries only production rooms and a staffed Control Center; seeding
    // from that silently drops dormitories and power plants, wrecking both the
    // power balance and any clause that scales with dorm levels.
    const layoutQuery = useQuery(baseLayoutQueryOptions(uid));

    const catalogQuery = useQuery(baseCatalogQueryOptions());
    const catalog = useMemo(() => {
        const map = new Map<FacilityType, ICatalogRoom>();
        for (const room of catalogQuery.data?.rooms ?? []) map.set(room.room_type, room);
        return map;
    }, [catalogQuery.data]);
    const slots = catalogQuery.data?.slots ?? EMPTY_SLOTS;
    const formulas = catalogQuery.data?.formulas ?? EMPTY_FORMULAS;

    // Structure (corridors, elevators, activity rooms) is dropped here rather
    // than on the board, so it never reaches the engine either. That is safe
    // precisely because it is inert: zero seats, zero electricity, and no buff
    // targets it - so facility counts, dorm levels and power are all unchanged.
    const realLayout = useMemo(() => (layoutQuery.data?.rooms ?? []).filter((room) => isPlannable(room.room_type, catalog)), [layoutQuery.data, catalog]);

    const [persisted, setPersisted] = useLocalStorageState<IPersisted>(storageKey(uid), {
        layout: [],
        locked: [],
        excluded: [],
    });

    /*
     * The real base arrives asynchronously; adopt it as the draft the first
     * time it lands, and leave a plan in progress alone thereafter.
     *
     * Accepting an optimizer proposal writes here, so a stored plan has to
     * survive a reload - which also means a plan can drift from the real base
     * with no way back. `reset` is that way back, and the board surfaces it
     * whenever `dirty` is true. Without a visible reset this guard strands the
     * player on a plan they can neither see the origin of nor undo.
     */
    useEffect(() => {
        // Wait for the catalogue: adopting before it lands would persist an
        // unfiltered layout that never gets cleaned up.
        if (realLayout.length === 0 || catalog.size === 0) return;
        setPersisted((prev) => (prev.layout.length === 0 ? { ...prev, layout: realLayout } : prev));
    }, [realLayout, catalog, setPersisted]);

    const layout = persisted.layout;

    const [selectedSlotId, setSelectedSlotId] = useState<string | null>(null);
    const [proposal, setProposal] = useState<IOptimizeResponse | null>(null);

    const boardRooms = useMemo(() => {
        const real = layoutQuery.data?.rooms ?? [];
        // The real base carries the structure the plan drops, so it leads. But
        // when that fetch has failed the plan is still a complete set of rooms -
        // drawing it beats drawing an empty base next to a scored headline.
        if (real.length === 0) return layout;
        const drafted = new Map(layout.map((room) => [room.slot_id, room]));
        return real.map((room) => drafted.get(room.slot_id) ?? room);
    }, [layoutQuery.data, layout]);

    const presets = layoutQuery.data?.presets ?? EMPTY_PRESETS;

    const evaluation = useEvaluation(uid, layout);

    const patchLayout = useCallback((fn: (rooms: IDraftRoom[]) => IDraftRoom[]) => setPersisted((prev) => ({ ...prev, layout: fn(prev.layout) })), [setPersisted]);

    const seatsFor = useCallback((room: IDraftRoom) => seatsOf(room, catalog), [catalog]);

    const slotOf = useCallback((operatorId: string) => layout.find((r) => r.operators.includes(operatorId))?.slot_id ?? null, [layout]);

    const assign = useCallback(
        (slotId: string, operatorId: string) => {
            patchLayout((rooms) => {
                const target = rooms.find((r) => r.slot_id === slotId);
                if (!target) return rooms;
                if (target.operators.includes(operatorId)) return rooms;
                if (target.operators.length >= seatsOf(target, catalog)) return rooms;
                // One operator, one seat - the backend rejects a layout that
                // breaks this, so vacate the old room as part of the same edit.
                return rooms.map((room) => {
                    if (room.slot_id === slotId) return { ...room, operators: [...room.operators, operatorId] };
                    if (room.operators.includes(operatorId)) {
                        return { ...room, operators: room.operators.filter((id) => id !== operatorId) };
                    }
                    return room;
                });
            });
        },
        [patchLayout, catalog],
    );

    const unassign = useCallback(
        (slotId: string, operatorId: string) => {
            patchLayout((rooms) => rooms.map((room) => (room.slot_id === slotId ? { ...room, operators: room.operators.filter((id) => id !== operatorId) } : room)));
        },
        [patchLayout],
    );

    const clearRoom = useCallback(
        (slotId: string) => {
            patchLayout((rooms) => rooms.map((room) => (room.slot_id === slotId ? { ...room, operators: [] } : room)));
        },
        [patchLayout],
    );

    const setFormula = useCallback(
        (slotId: string, formula: IDraftRoom["formula_type"]) => {
            patchLayout((rooms) => rooms.map((room) => (room.slot_id === slotId ? { ...room, formula_type: formula } : room)));
        },
        [patchLayout],
    );

    const setLevel = useCallback(
        (slotId: string, level: number) => {
            patchLayout((rooms) =>
                rooms.map((room) => {
                    if (room.slot_id !== slotId) return room;
                    // Downgrading can leave more operators than seats, which the
                    // backend rejects outright - drop the overflow here so the
                    // edit stays valid instead of erroring on the next score.
                    const seats = seatsOf({ ...room, level }, catalog);
                    return { ...room, level, operators: room.operators.slice(0, seats) };
                }),
            );
        },
        [patchLayout, catalog],
    );

    const toggleLocked = useCallback(
        (operatorId: string) => {
            setPersisted((prev) => ({
                ...prev,
                locked: prev.locked.includes(operatorId) ? prev.locked.filter((id) => id !== operatorId) : [...prev.locked, operatorId],
            }));
        },
        [setPersisted],
    );

    const toggleExcluded = useCallback(
        (operatorId: string) => {
            setPersisted((prev) => ({
                ...prev,
                excluded: prev.excluded.includes(operatorId) ? prev.excluded.filter((id) => id !== operatorId) : [...prev.excluded, operatorId],
            }));
        },
        [setPersisted],
    );

    const [rotation, setRotation] = useState<IRotationResponse | null>(null);

    // The rotation's own count wins once it has run; before that the catalogue
    // carries the constant. Zero only while the catalogue is still loading.
    const shiftCount = rotation?.shift_count ?? catalogQuery.data?.shift_count ?? 0;
    const [viewShift, setViewShift] = useState<number | null>(null);

    const rotationMutation = useMutation({
        mutationFn: () => rotationPlanFn({ data: { uid, layout, locked: persisted.locked, excluded: persisted.excluded } }),
        onSuccess: setRotation,
    });

    const optimizeMutation = useMutation({
        mutationFn: (scope: string[]) =>
            optimizeLayoutFn({
                data: { uid, layout, scope, locked: persisted.locked, excluded: persisted.excluded },
            }),
        onSuccess: setProposal,
    });

    // One action, both answers. A proposal tells you the best peak staffing; the
    // rotation tells you whether that staffing survives a week. Making the
    // second an extra click meant most people never saw the number that decides
    // whether the first is actually usable. They run against the same draft, so
    // they are consistent by construction.
    const runOptimize = useCallback(
        (scope: string[]) => {
            optimizeMutation.mutate(scope);
            rotationMutation.mutate();
        },
        [optimizeMutation, rotationMutation],
    );

    /**
     * Apply a proposal's rooms to the draft. Operators the proposal seats
     * elsewhere are vacated first, so applying one room at a time can never
     * leave the same operator in two places.
     */
    const applyRooms = useCallback(
        (slotIds: string[]) => {
            if (!proposal) return;
            const bySlot = new Map(proposal.proposal.rooms.map((r) => [r.slot_id, r.operators.map((o) => o.operator_id)]));
            const incoming = new Set(slotIds.flatMap((id) => bySlot.get(id) ?? []));
            patchLayout((rooms) =>
                rooms.map((room) => {
                    if (slotIds.includes(room.slot_id)) return { ...room, operators: bySlot.get(room.slot_id) ?? [] };
                    if (room.operators.some((id) => incoming.has(id))) {
                        return { ...room, operators: room.operators.filter((id) => !incoming.has(id)) };
                    }
                    return room;
                }),
            );
        },
        [proposal, patchLayout],
    );

    const acceptRoom = useCallback(
        (slotId: string) => {
            applyRooms([slotId]);
            setProposal((prev) => {
                if (!prev) return prev;
                const room_diffs = prev.room_diffs.filter((d) => d.slot_id !== slotId);
                return room_diffs.length > 0 ? { ...prev, room_diffs } : null;
            });
        },
        [applyRooms],
    );

    const acceptAll = useCallback(() => {
        if (!proposal) return;
        applyRooms(proposal.room_diffs.map((d) => d.slot_id));
        setProposal(null);
    }, [proposal, applyRooms]);

    const discardPlan = useCallback(() => {
        setProposal(null);
        setRotation(null);
    }, []);

    const reset = useCallback(() => {
        setPersisted({ layout: realLayout, locked: [], excluded: [] });
        setProposal(null);
        setRotation(null);
    }, [setPersisted, realLayout]);

    // A rotation describes one specific staffing. The moment the draft changes
    // it is describing something that no longer exists, so drop it - and fall
    // back to the draft view, rather than stranding the board on a shift of a
    // rotation that no longer exists.
    // biome-ignore lint/correctness/useExhaustiveDependencies: keyed on the layout's identity, not the array reference
    useEffect(() => {
        setRotation(null);
        setViewShift(null);
    }, [layoutIdentity(layout)]);

    const shiftRoom = useCallback(
        (slotId: string) => {
            if (viewShift == null) return undefined;
            return rotation?.rotation.shifts.find((s) => s.index === viewShift)?.rooms.find((r) => r.slot_id === slotId);
        },
        [rotation, viewShift],
    );

    const dirty = useMemo(() => !sameLayout(layout, realLayout), [layout, realLayout]);

    const selectedRoom = useMemo(() => layout.find((r) => r.slot_id === selectedSlotId) ?? null, [layout, selectedSlotId]);

    return {
        layout,
        dirty,
        catalog,
        slots,
        formulas,
        presets,
        shiftCount,
        boardRooms,
        catalogLoading: catalogQuery.isLoading,
        // Both must land before the board is meaningful: the catalogue decides
        // which of the layout's rooms are plannable at all.
        layoutLoading: layoutQuery.isLoading || catalogQuery.isLoading,
        selectedSlotId,
        setSelectedSlotId,
        selectedRoom,
        evaluation: evaluation.data,
        evaluating: evaluation.isFetching,
        evaluationError: evaluation.error,
        assign,
        unassign,
        clearRoom,
        setFormula,
        setLevel,
        slotOf,
        seatsFor,
        lockedIds: persisted.locked,
        toggleLocked,
        excludedIds: persisted.excluded,
        toggleExcluded,
        rotation,
        rotating: rotationMutation.isPending,
        rotationError: rotationMutation.error,
        viewShift,
        setViewShift,
        shiftRoom,
        proposal,
        optimizing: optimizeMutation.isPending || rotationMutation.isPending,
        optimizeError: optimizeMutation.error,
        runOptimize,
        acceptRoom,
        acceptAll,
        discardPlan,
        reset,
    };
}

/**
 * A layout's identity: room shape plus crew, insensitive to seat and room
 * order. Two layouts with the same identity score identically.
 */
function layoutIdentity(rooms: IDraftRoom[]): string {
    return rooms
        .map((r) => `${r.slot_id}:${r.room_type}:${r.level}:${r.formula_type ?? ""}:${[...r.operators].sort().join(",")}`)
        .sort()
        .join("|");
}

/** Crew-identity comparison: seat order is not a difference worth flagging. */
function sameLayout(a: IDraftRoom[], b: IDraftRoom[]): boolean {
    return a.length === b.length && layoutIdentity(a) === layoutIdentity(b);
}
