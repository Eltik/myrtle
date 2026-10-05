/**
 * The planner's group filter. Which plans feed the requirement roll-up is a
 * per-plan map; the filter is a set of group names that, when engaged,
 * rewrites that map for every plan at once.
 */

/**
 * Filter key for plans in no group. A NUL cannot be typed into a group name,
 * so it never collides with a real one.
 */
export const UNGROUPED_FILTER_KEY = "\u0000ungrouped";

export type GroupFilter = ReadonlySet<string>;
export type ActivePlans = Readonly<Record<string, boolean>>;

interface IFilterablePlan {
    operator_id: string;
    groups?: readonly string[] | null;
}

/** Whether a plan passes the filter: any of its groups is selected, or Ungrouped is for a plan with none. An empty filter passes everything. */
export function matchesGroupFilter(plan: IFilterablePlan, filter: GroupFilter): boolean {
    if (filter.size === 0) return true;
    const groups = plan.groups ?? [];
    if (groups.length === 0) return filter.has(UNGROUPED_FILTER_KEY);
    return groups.some((g) => filter.has(g));
}

/**
 * The one read of the active map. A plan with an explicit entry uses it; one
 * without (made after the last filter action) follows the filter, which with
 * nothing selected means active.
 */
export function isPlanActive(plan: IFilterablePlan, activePlans: ActivePlans, filter: GroupFilter): boolean {
    return activePlans[plan.operator_id] ?? matchesGroupFilter(plan, filter);
}

/** An entry for every plan, so nothing is left to the default once a filter action runs. */
export function activePlansForFilter(plans: readonly IFilterablePlan[], filter: GroupFilter): Record<string, boolean> {
    const next: Record<string, boolean> = {};
    for (const p of plans) {
        next[p.operator_id] = matchesGroupFilter(p, filter);
    }
    return next;
}

export function toggleGroupFilterKey(filter: GroupFilter, key: string): Set<string> {
    const next = new Set(filter);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    return next;
}

/** Keeps a selected group selected across a rename, and drops it on delete (`newName` null). */
export function renameGroupFilterKey(filter: GroupFilter, oldName: string, newName: string | null): GroupFilter {
    if (!filter.has(oldName)) return filter;
    const next = new Set(filter);
    next.delete(oldName);
    if (newName !== null) next.add(newName);
    return next;
}

/** Plans per filter key, for the chip counts. A plan in several groups counts toward each. */
export function groupFilterCounts(plans: readonly IFilterablePlan[]): Map<string, number> {
    const counts = new Map<string, number>();
    for (const p of plans) {
        const groups = p.groups ?? [];
        const keys = groups.length === 0 ? [UNGROUPED_FILTER_KEY] : groups;
        for (const k of keys) {
            counts.set(k, (counts.get(k) ?? 0) + 1);
        }
    }
    return counts;
}

/**
 * The `active` ids the requirements query is fetched with. Undefined when every
 * plan is active, so the page shares the unfiltered query; a `none` sentinel
 * when no plan is, because an empty list would send no filter and count them all.
 */
export function activePlanIds(plans: readonly IFilterablePlan[], activePlans: ActivePlans, filter: GroupFilter): string[] | undefined {
    const active = plans.filter((p) => isPlanActive(p, activePlans, filter)).map((p) => p.operator_id);
    if (active.length === plans.length) return undefined;
    if (active.length === 0) return ["none"];
    return active;
}
