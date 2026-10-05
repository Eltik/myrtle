import { Trash } from "lucide-react";
import * as React from "react";

import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
import { FilterChip } from "#/components/ui/filter-chip";
import { Skeleton } from "#/components/ui/skeleton";
import type { IOperatorPlanResponse, IPlanGroup } from "#/lib/api/planner";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./OperatorPlanner.messages";
import { type IPlanEntryContext, PlanEntry } from "./PlanEntry";
import { groupFilterCounts, UNGROUPED_FILTER_KEY } from "./planFilters";
import type { IPlannerSelection } from "./usePlannerSelection";

interface IPlanListProps {
    plans: IOperatorPlanResponse[];
    groups: IPlanGroup[];
    /** Index of the first unpinned group when pinned and unpinned groups are both present; -1 hides the divider. */
    pinDividerAt: number;
    isLoading: boolean;
    selection: IPlannerSelection;
    entryContext: IPlanEntryContext;
    onDeleteSelected: () => void;
}

/** The Plans tab: group filter chips, select-all with bulk delete, and a card per plan. */
export function PlanList({ plans, groups, pinDividerAt, isLoading, selection, entryContext, onDeleteSelected }: IPlanListProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");

    if (isLoading) {
        return (
            <div className="flex flex-col gap-4">
                {["sk-1", "sk-2", "sk-3"].map((key) => (
                    <div key={key} className="flex items-center gap-3 rounded-xl border border-border/40 p-4">
                        <Skeleton className="size-4 rounded" />
                        <Skeleton className="size-10 rounded-xl" />
                        <div className="flex flex-1 flex-col gap-1.5">
                            <Skeleton className="h-3.5 w-24 rounded" />
                            <Skeleton className="h-3 w-16 rounded" />
                        </div>
                    </div>
                ))}
            </div>
        );
    }

    const { groupFilter } = selection;
    const filterCounts = groupFilterCounts(plans);
    const ungroupedCount = filterCounts.get(UNGROUPED_FILTER_KEY) ?? 0;
    const selectedCount = plans.filter(selection.isActive).length;
    const allSelected = plans.length > 0 && selectedCount === plans.length;
    const toggleGroupFilter = (key: string) => selection.toggleGroupFilter(plans, key);

    return (
        <div className="flex flex-col gap-4">
            {groups.length > 0 && (
                // biome-ignore lint/a11y/useSemanticElements: a labelled row of toggle chips, not a form fieldset
                <div role="group" aria-label={t("planner.filter.aria")} className="flex flex-wrap items-center gap-2">
                    <FilterChip label={t("planner.filter.all")} active={groupFilter.size === 0} count={plans.length} onSelect={() => selection.applyGroupFilter(plans, new Set())} />
                    {groups.map((g, i) => (
                        <React.Fragment key={g.name}>
                            {i === pinDividerAt && <span aria-hidden="true" className="h-5 w-px bg-border" />}
                            <FilterChip label={g.name} active={groupFilter.has(g.name)} count={filterCounts.get(g.name) ?? 0} onSelect={() => toggleGroupFilter(g.name)} />
                        </React.Fragment>
                    ))}
                    {(ungroupedCount > 0 || groupFilter.has(UNGROUPED_FILTER_KEY)) && <FilterChip label={t("planner.filter.ungrouped")} active={groupFilter.has(UNGROUPED_FILTER_KEY)} count={ungroupedCount} onSelect={() => toggleGroupFilter(UNGROUPED_FILTER_KEY)} />}
                </div>
            )}
            <div className="flex items-center justify-between border-border/40 border-b pb-3">
                {/* biome-ignore lint/a11y/noLabelWithoutControl: Checkbox component internally renders the input control */}
                <label className="flex cursor-pointer items-center gap-2 font-medium text-muted-foreground text-xs hover:text-foreground">
                    <Checkbox checked={allSelected} onCheckedChange={() => selection.setAllActive(plans, !allSelected)} />
                    <span>{allSelected ? t("planner.unselectAll") : t("planner.selectAll")}</span>
                </label>
                {selectedCount > 0 && (
                    <Button variant="ghost" size="xs" className="fade-in zoom-in-95 h-7 animate-in cursor-pointer border border-red-500/20 bg-red-500/5 text-red-600 duration-150 hover:border-red-500/40 hover:bg-red-500/15 hover:text-red-600 dark:text-red-400 dark:hover:text-red-400" onClick={onDeleteSelected}>
                        <Trash className="mr-1 size-3.5" />
                        {t("planner.deleteSelected", { count: selectedCount })}
                    </Button>
                )}
            </div>
            {plans.map((p) => (
                <PlanEntry key={p.id} plan={p} variant="card" context={entryContext} />
            ))}
        </div>
    );
}
