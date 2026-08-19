import { useMutation, useQuery } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useDebounce } from "#/hooks/use-debounce";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import {
    baseCatalogQueryOptions,
    baseLayoutQueryOptions,
    evaluateLayoutQueryOptions,
    type FacilityType,
    type ICatalogFormula,
    type ICatalogRoom,
    type ICatalogSlot,
    type IDraftRoom,
    type IEvaluateResponse,
    type IOptimizeResponse,
    type IRotationResponse,
    type ISlotPresets,
    optimizeLayoutFn,
    rotationPlanQueryOptions,
} from "#/lib/api/base";
import type { IShiftRoom } from "#/lib/api/user";
import { type Catalog, isPlannable } from "./catalog";

export interface IOptimizerAPI {
    layout: IDraftRoom[];
    dirty: boolean;
    catalog: Catalog;
    slots: ICatalogSlot[];
    formulas: ICatalogFormula[];
    presets: ISlotPresets[];
    shiftCount: number;
    boardRooms: IDraftRoom[];
    catalogLoading: boolean;
    layoutLoading: boolean;

    evaluation: IEvaluateResponse | undefined;
    evaluating: boolean;
    evaluationError: Error | null;

    rotation: IRotationResponse | null;

    /** Plan with every operator's highest base skills, promoted or not. */
    ignorePromotion: boolean;
    setIgnorePromotion: (value: boolean) => void;
    viewShift: number | null;
    setViewShift: (shift: number | null) => void;
    shiftRoom: (slotId: string) => IShiftRoom | undefined;

    proposal: IOptimizeResponse | null;
    optimizing: boolean;
    optimizeError: Error | null;
    runOptimize: (scope: string[]) => void;

    reset: () => void;
}

interface IPersisted {
    layout: IDraftRoom[];
    ignorePromotion?: boolean;
}

const EMPTY_SLOTS: ICatalogSlot[] = [];
const EMPTY_FORMULAS: ICatalogFormula[] = [];
const EMPTY_PRESETS: ISlotPresets[] = [];

function storageKey(uid: string): string {
    return `base-optimizer:${uid}:v2`;
}

function useEvaluation(uid: string, layout: IDraftRoom[], ignorePromotion: boolean) {
    const settled = useDebounce(layout, 350);
    return useQuery(evaluateLayoutQueryOptions(uid, settled, ignorePromotion));
}

function useRotation(uid: string, layout: IDraftRoom[], ignorePromotion: boolean) {
    const settled = useDebounce(layout, 350);
    return useQuery(rotationPlanQueryOptions(uid, settled, ignorePromotion));
}

export function useOptimizer(uid: string): IOptimizerAPI {
    const layoutQuery = useQuery(baseLayoutQueryOptions(uid));

    const catalogQuery = useQuery(baseCatalogQueryOptions());
    const catalog = useMemo(() => {
        const map = new Map<FacilityType, ICatalogRoom>();
        for (const room of catalogQuery.data?.rooms ?? []) map.set(room.room_type, room);
        return map;
    }, [catalogQuery.data]);
    const slots = catalogQuery.data?.slots ?? EMPTY_SLOTS;
    const formulas = catalogQuery.data?.formulas ?? EMPTY_FORMULAS;

    const realLayout = useMemo(() => (layoutQuery.data?.rooms ?? []).filter((room: { room_type: string }) => isPlannable(room.room_type, catalog)), [layoutQuery.data, catalog]);

    const [persisted, setPersisted] = useLocalStorageState<IPersisted>(storageKey(uid), {
        layout: [],
    });

    useEffect(() => {
        if (realLayout.length === 0 || catalog.size === 0) return;
        setPersisted((prev) => (prev.layout.length === 0 ? { ...prev, layout: realLayout } : prev));
    }, [realLayout, catalog, setPersisted]);

    const layout = persisted.layout;
    const ignorePromotion = persisted.ignorePromotion ?? false;
    const setIgnorePromotion = useCallback((value: boolean) => setPersisted((prev) => ({ ...prev, ignorePromotion: value })), [setPersisted]);

    const [proposal, setProposal] = useState<IOptimizeResponse | null>(null);

    const boardRooms = useMemo(() => {
        const real = layoutQuery.data?.rooms ?? [];
        if (real.length === 0) return layout;
        const drafted = new Map(layout.map((room) => [room.slot_id, room]));
        return real.map((room) => drafted.get(room.slot_id) ?? room);
    }, [layoutQuery.data, layout]);

    const presets = layoutQuery.data?.presets ?? EMPTY_PRESETS;

    const evaluation = useEvaluation(uid, layout, ignorePromotion);

    const rotationQuery = useRotation(uid, layout, ignorePromotion);
    const rotation = rotationQuery.data ?? null;

    const shiftCount = rotation?.shift_count ?? catalogQuery.data?.shift_count ?? 0;
    const [viewShift, setViewShift] = useState<number | null>(null);

    const optimizeMutation = useMutation({
        mutationFn: (scope: string[]) =>
            optimizeLayoutFn({
                data: { uid, layout, scope, ignorePromotion },
            }),
        onSuccess: setProposal,
    });

    const runOptimize = useCallback((scope: string[]) => optimizeMutation.mutate(scope), [optimizeMutation]);

    const reset = useCallback(() => {
        setPersisted({ layout: realLayout });
        setProposal(null);
    }, [setPersisted, realLayout]);

    const identity = useMemo(() => layoutIdentity(layout), [layout]);

    // biome-ignore lint/correctness/useExhaustiveDependencies: the identity is not read in the body
    useEffect(() => {
        setViewShift(null);
    }, [identity]);

    const shiftRoom = useCallback(
        (slotId: string) => {
            if (viewShift == null) return undefined;
            return rotation?.rotation.shifts.find((s) => s.index === viewShift)?.rooms.find((r) => r.slot_id === slotId);
        },
        [rotation, viewShift],
    );

    const dirty = useMemo(() => identity !== layoutIdentity(realLayout), [identity, realLayout]);

    return useMemo(
        () => ({
            layout,
            dirty,
            catalog,
            slots,
            formulas,
            presets,
            shiftCount,
            boardRooms,
            catalogLoading: catalogQuery.isLoading,
            layoutLoading: layoutQuery.isLoading || catalogQuery.isLoading,
            evaluation: evaluation.data,
            evaluating: evaluation.isFetching,
            evaluationError: evaluation.error,
            rotation,
            ignorePromotion,
            setIgnorePromotion,
            viewShift,
            setViewShift,
            shiftRoom,
            proposal,
            optimizing: optimizeMutation.isPending,
            optimizeError: optimizeMutation.error ?? rotationQuery.error,
            runOptimize,
            reset,
        }),
        [
            layout,
            dirty,
            catalog,
            slots,
            formulas,
            presets,
            shiftCount,
            boardRooms,
            catalogQuery.isLoading,
            layoutQuery.isLoading,
            evaluation.data,
            evaluation.isFetching,
            evaluation.error,
            rotation,
            rotationQuery.error,
            ignorePromotion,
            setIgnorePromotion,
            viewShift,
            shiftRoom,
            proposal,
            optimizeMutation.isPending,
            optimizeMutation.error,
            runOptimize,
            reset,
        ],
    );
}

function layoutIdentity(rooms: IDraftRoom[]): string {
    return rooms
        .map((r) => `${r.slot_id}:${r.room_type}:${r.level}:${r.formula_type ?? ""}:${[...r.operators].sort().join(",")}`)
        .sort()
        .join("|");
}
