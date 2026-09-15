import { useMemo } from "react";
import { Skeleton } from "#/components/ui/skeleton";
import { buildBoard } from "#/lib/base/board";
import { toRosterOptions } from "#/lib/base/roster";
import { useOptimizer } from "#/lib/base/use-optimizer";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "../OptimizerTab.messages";
import type { IOptimizerProps } from "../optimizers";
import { BasePanel } from "./BasePanel";
import { BaseOptimizerProvider } from "./base-context";
import { crewsForShift } from "./shift-crews";

export function BaseOptimizer({ uid, roster, operatorsStatic }: IOptimizerProps) {
    /** The empty state is declared next to the tab that hosts this optimizer. */
    const t: TypedT<typeof messages> = useT("user");
    const api = useOptimizer(uid);

    const rosterById = useMemo(() => new Map(toRosterOptions(roster, operatorsStatic, api.ignorePromotion).map((op) => [op.id, op])), [roster, operatorsStatic, api.ignorePromotion]);
    const presetBySlot = useMemo(() => new Map(api.presets.map((p) => [p.slot_id, p.shifts])), [api.presets]);

    const { rooms, marks } = useMemo(() => crewsForShift(api, presetBySlot), [api, presetBySlot]);
    const board = useMemo(() => buildBoard(api.slots, rooms, api.catalog, rosterById, marks), [api.slots, rooms, api.catalog, rosterById, marks]);

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
                <p className="text-muted-foreground text-sm">{t("profile.optimizer.noBaseData")}</p>
            </div>
        );
    }

    return (
        <BaseOptimizerProvider value={api}>
            <BasePanel board={board} />
        </BaseOptimizerProvider>
    );
}
