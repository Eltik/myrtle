import { useMutation, useQuery } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useDebounce } from "#/hooks/use-debounce";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { baseCatalogQueryOptions, baseLayoutQueryOptions, evaluateLayoutQueryOptions, type FacilityType, type ICatalogRoom, type ICatalogSlot, type IDraftRoom, type IOptimizeResponse, type IRotationResponse, optimizeLayoutFn, rotationPlanFn } from "#/lib/api/base";
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
    discardProposal: () => void;

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
/** Stable identity, so an unloaded catalogue does not re-derive the board every render. */
const EMPTY_SLOTS: ICatalogSlot[] = [];

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

    // The real base arrives asynchronously; adopt it as the draft the first
    // time it lands, and leave an in-progress draft alone thereafter.
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
        const drafted = new Map(layout.map((room) => [room.slot_id, room]));
        return (layoutQuery.data?.rooms ?? []).map((room) => drafted.get(room.slot_id) ?? room);
    }, [layoutQuery.data, layout]);

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

    const discardProposal = useCallback(() => setProposal(null), []);

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
        discardProposal,
        reset,
    };
}

/**
 * A layout's identity: room shape plus crew, insensitive to seat and room
 * order. Two layouts with the same identity score identically.
 */
function layoutIdentity(rooms: IDraftRoom[]): string {
    return rooms
        .map((r) => `${r.slot_id}:${r.level}:${r.formula_type ?? ""}:${[...r.operators].sort().join(",")}`)
        .sort()
        .join("|");
}

/** Crew-identity comparison: seat order is not a difference worth flagging. */
function sameLayout(a: IDraftRoom[], b: IDraftRoom[]): boolean {
    return a.length === b.length && layoutIdentity(a) === layoutIdentity(b);
}
