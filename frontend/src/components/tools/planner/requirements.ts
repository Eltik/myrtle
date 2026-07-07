import type { IOperatorPlanResponse, IPlanRequirementItem } from "#/lib/api/planner";

export type RequirementStatus = "complete" | "craft" | "missing";
export type RequirementCategory = "expLmd" | "skills" | "chips" | "modules" | "materials";
export type RequirementsView = "grouped" | "flat" | "by-operator";
export type CategoryFilter = "all" | RequirementCategory;
export type StatusFilter = "all" | RequirementStatus;

export interface IRequirementStatusInfo {
    status: RequirementStatus;
    /** Amount that must be crafted because inventory falls short of the requirement (0 when fully covered). */
    shortfall: number;
}

/**
 * Single source of truth for a requirement's status. The row rendering and the
 * filter/count logic both derive status here so the visible badges and the filter
 * chips can never disagree. Precedence matches the original inline row logic:
 * missing (has an unbuildable shortfall) > craft (shortfall, but craftable) > complete.
 */
export function requirementStatus(item: IPlanRequirementItem): IRequirementStatusInfo {
    const shortfall = Math.max(item.requiredCount - item.inventoryCount, 0);
    if (item.missingCount > 0) return { status: "missing", shortfall };
    if (shortfall > 0) return { status: "craft", shortfall };
    return { status: "complete", shortfall: 0 };
}

export interface ICategoryDef {
    key: RequirementCategory;
    label: string;
    sortGroups: number[];
}

/** Category display order for grouped view and filter chips (mirrors the backend sortGroup axis). */
export const REQUIREMENT_CATEGORIES: ICategoryDef[] = [
    { key: "expLmd", label: "EXP & LMD", sortGroups: [0, 1] },
    { key: "skills", label: "Skill Summaries", sortGroups: [2] },
    { key: "chips", label: "Chips", sortGroups: [3] },
    { key: "modules", label: "Module Data", sortGroups: [4] },
    { key: "materials", label: "Materials", sortGroups: [5] },
];

export function requirementCategory(item: IPlanRequirementItem): RequirementCategory {
    for (const def of REQUIREMENT_CATEGORIES) {
        if (def.sortGroups.includes(item.sortGroup)) return def.key;
    }
    return "materials"; // sortGroup 5 (and any unexpected value) is tiered materials
}

export const STATUS_FILTER_LABELS: Record<RequirementStatus, string> = {
    missing: "Missing",
    craft: "Craft needed",
    complete: "Complete",
};

export const STATUS_FILTER_ORDER: RequirementStatus[] = ["missing", "craft", "complete"];

export function inCategory(item: IPlanRequirementItem, filter: CategoryFilter): boolean {
    return filter === "all" || requirementCategory(item) === filter;
}

export function inStatus(item: IPlanRequirementItem, filter: StatusFilter): boolean {
    return filter === "all" || requirementStatus(item).status === filter;
}

export interface IRequirementSubtotal {
    items: number;
    missing: number;
    craft: number;
}

export function subtotal(items: IPlanRequirementItem[]): IRequirementSubtotal {
    let missing = 0;
    let craft = 0;
    for (const item of items) {
        const { status } = requirementStatus(item);
        if (status === "missing") missing++;
        else if (status === "craft") craft++;
    }
    return { items: items.length, missing, craft };
}

export function formatSubtotal(counts: IRequirementSubtotal): string {
    const parts = [`${counts.items} item${counts.items === 1 ? "" : "s"}`];
    if (counts.missing > 0) parts.push(`${counts.missing} missing`);
    if (counts.craft > 0) parts.push(`${counts.craft} to craft`);
    return parts.join(" · ");
}

/** Terse target summary for a plan, e.g. "E2 Lv60 · S2 M3 · 2 modules". */
export function formatPlanTarget(plan: IOperatorPlanResponse): string {
    const parts = [`E${plan.target_elite} Lv${plan.target_level}`];
    const masteries = (plan.target_skills ?? []).filter((s) => s.mastery_level > 0);
    if (masteries.length > 0) {
        parts.push(masteries.map((s) => `S${s.skill_index + 1} M${s.mastery_level}`).join(" "));
    } else if (plan.target_skill_level > 1) {
        parts.push(`SL${plan.target_skill_level}`);
    }
    const moduleCount = (plan.target_modules ?? []).filter((m) => m.module_stage > 0).length;
    if (moduleCount > 0) {
        parts.push(`${moduleCount} module${moduleCount === 1 ? "" : "s"}`);
    }
    return parts.join(" · ");
}

export interface IRequirementsViewSettings {
    view: RequirementsView;
    category: CategoryFilter;
    status: StatusFilter;
}

export const REQUIREMENTS_VIEW_STORAGE_KEY = "planner:requirements-view";

export const DEFAULT_REQUIREMENTS_VIEW: IRequirementsViewSettings = {
    view: "grouped",
    category: "all",
    status: "all",
};
