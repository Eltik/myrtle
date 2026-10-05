import * as React from "react";

import type { IOperatorPlanResponse } from "#/lib/api/planner";
import { activePlansForFilter, type GroupFilter, isPlanActive, renameGroupFilterKey, toggleGroupFilterKey } from "./planFilters";

/**
 * Which plans feed the requirement roll-up: a per-plan active map plus the
 * group filter that rewrites it (see `planFilters.ts`). The plan list is not
 * held here because the page fetches it with this selection, so the actions
 * that rewrite every plan's entry take the current list as an argument.
 */
export function usePlannerSelection() {
    const [activePlans, setActivePlans] = React.useState<Record<string, boolean>>({});
    const [groupFilter, setGroupFilter] = React.useState<GroupFilter>(() => new Set());

    const isActive = (plan: IOperatorPlanResponse) => isPlanActive(plan, activePlans, groupFilter);

    const togglePlan = (plan: IOperatorPlanResponse) => {
        setActivePlans((prev) => ({
            ...prev,
            [plan.operator_id]: !isPlanActive(plan, prev, groupFilter),
        }));
    };

    /** Clears the group filter and sets every plan to `active`. */
    const setAllActive = (plans: readonly IOperatorPlanResponse[], active: boolean) => {
        const next: Record<string, boolean> = {};
        for (const p of plans) {
            next[p.operator_id] = active;
        }
        setGroupFilter(new Set());
        setActivePlans(next);
    };

    /** Engaging the filter rewrites every plan's entry, so per-plan picks made before it do not survive it. */
    const applyGroupFilter = (plans: readonly IOperatorPlanResponse[], next: GroupFilter) => {
        setGroupFilter(next);
        setActivePlans(activePlansForFilter(plans, next));
    };

    /** The chip row and the Groups tab's checkboxes are one control: both toggle a key of the group filter. */
    const toggleGroupFilter = (plans: readonly IOperatorPlanResponse[], key: string) => applyGroupFilter(plans, toggleGroupFilterKey(groupFilter, key));

    const renameGroup = (oldName: string, newName: string) => setGroupFilter((prev) => renameGroupFilterKey(prev, oldName, newName));

    const dropGroup = (plans: readonly IOperatorPlanResponse[], name: string) => {
        const next = renameGroupFilterKey(groupFilter, name, null);
        if (next !== groupFilter) applyGroupFilter(plans, next);
    };

    /** Forgets deleted plans, so a later plan for the same operator starts from the filter again. */
    const dropPlans = (ids: readonly string[]) => {
        setActivePlans((prev) => {
            const next = { ...prev };
            for (const id of ids) {
                delete next[id];
            }
            return next;
        });
    };

    return { activePlans, groupFilter, isActive, togglePlan, setAllActive, applyGroupFilter, toggleGroupFilter, renameGroup, dropGroup, dropPlans };
}

export type IPlannerSelection = ReturnType<typeof usePlannerSelection>;
