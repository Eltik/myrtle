import { useMemo } from "react";
import { Skeleton } from "#/components/ui/skeleton";
import type { IRosterEntry } from "#/lib/api/user";
import { buildBoard } from "#/lib/base/layout";
import { useOptimizer } from "#/lib/base/use-optimizer";
import type { IOperatorListItem } from "#/types/operators";
import { Board } from "./Board";

interface IProps {
    uid: string;
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
}

export function OptimizerTab({ uid }: IProps) {
    const api = useOptimizer(uid);
    const board = useMemo(() => buildBoard(api.slots, api.boardRooms, api.catalog), [api.slots, api.boardRooms, api.catalog]);

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
        <div className="rounded-xl border border-border bg-card p-3">
            {/* The stage owns its own padding, so the card only contributes a gutter. */}
            <div className="relative overflow-x-auto">
                <Board board={board} />
            </div>
        </div>
    );
}
