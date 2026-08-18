import { useMemo } from "react";
import { Skeleton } from "#/components/ui/skeleton";
import { buildBoard } from "#/lib/base/layout";
import { toRosterOptions } from "#/lib/base/roster";
import { useOptimizer } from "#/lib/base/use-optimizer";
import type { IOptimizerProps } from "../optimizers";
import { BoardView } from "./BoardView";
import { OptimizerProvider } from "./optimizer-context";
import { crewsForShift } from "./shift-crews";

export function BaseOptimizer({ uid, roster, operatorsStatic }: IOptimizerProps) {
    const api = useOptimizer(uid);

    const rosterNames = useMemo(() => new Map(toRosterOptions(roster, operatorsStatic).map((op) => [op.id, op.name])), [roster, operatorsStatic]);
    const presetBySlot = useMemo(() => new Map(api.presets.map((p) => [p.slot_id, p.shifts])), [api.presets]);

    const { rooms, marks } = useMemo(() => crewsForShift(api, presetBySlot, api.catalog), [api, presetBySlot]);
    const board = useMemo(() => buildBoard(api.slots, rooms, api.catalog, rosterNames, marks), [api.slots, rooms, api.catalog, rosterNames, marks]);

    if (api.layoutLoading || api.catalogLoading) {
        return (
            <div className="flex flex-col gap-3">
                <Skeleton className="h-10 w-full" />
                <Skeleton className="h-64 w-full" />
            </div>
        );
    }

    if (api.layout.length === 0) {
        return (
            <div className="rounded-xl border border-border border-dashed p-8 text-center">
                <p className="text-muted-foreground text-sm">This profile has no base data to plan against yet.</p>
            </div>
        );
    }

    return (
        <OptimizerProvider value={api}>
            <BoardView board={board} />
        </OptimizerProvider>
    );
}
