import type { IOperatorPlanResponse, IPlanRequirementItem } from "#/lib/api/planner";
import { DEFAULT_LOCALE, formatMessage, sourceMessage } from "#/lib/i18n";
import { fullMessageKey, type TypedT } from "#/lib/i18n/messages";
import type { messages as requirementMessages } from "./requirements.messages";

/** A key in `requirements.messages.ts`; resolved by whichever component renders it. */
export type RequirementMessageKey = keyof typeof requirementMessages & string;

/** The `t` the summary helpers need, narrowed to the keys they can render. */
export type RequirementsT = TypedT<typeof requirementMessages>;

/**
 * Default `t` for a caller outside an `I18nProvider`. It resolves against the
 * bundled source catalog, so the English is the same one the components render
 * and this file carries no second copy of the text.
 */
const sourceT: RequirementsT = (key, values) => formatMessage(sourceMessage(fullMessageKey("tools", key)) ?? key, DEFAULT_LOCALE, values);

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
    labelKey: RequirementMessageKey;
    sortGroups: number[];
}

/** Category display order for grouped view and filter chips (mirrors the backend sortGroup axis). */
export const REQUIREMENT_CATEGORIES: ICategoryDef[] = [
    { key: "expLmd", labelKey: "planner.category.expLmd", sortGroups: [0, 1] },
    { key: "skills", labelKey: "planner.category.skills", sortGroups: [2] },
    { key: "chips", labelKey: "planner.category.chips", sortGroups: [3] },
    { key: "modules", labelKey: "planner.category.modules", sortGroups: [4] },
    { key: "materials", labelKey: "planner.category.materials", sortGroups: [5] },
];

export function requirementCategory(item: IPlanRequirementItem): RequirementCategory {
    for (const def of REQUIREMENT_CATEGORIES) {
        if (def.sortGroups.includes(item.sortGroup)) return def.key;
    }
    return "materials"; // sortGroup 5 (and any unexpected value) is tiered materials
}

export const STATUS_FILTER_LABEL_KEYS: Record<RequirementStatus, RequirementMessageKey> = {
    missing: "planner.status.missing",
    craft: "planner.status.craft",
    complete: "planner.status.complete",
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

export function formatSubtotal(counts: IRequirementSubtotal, t: RequirementsT = sourceT): string {
    const parts = [t("planner.subtotal.items", { count: counts.items })];
    if (counts.missing > 0) parts.push(t("planner.subtotal.missing", { count: counts.missing }));
    if (counts.craft > 0) parts.push(t("planner.subtotal.craft", { count: counts.craft }));
    return parts.join(t("planner.subtotal.separator"));
}

/** Terse target summary for a plan, e.g. "E2 Lv60 · S2 M3 · 2 modules". */
export function formatPlanTarget(plan: IOperatorPlanResponse, t: RequirementsT = sourceT): string {
    const parts = [t("planner.target.level", { elite: plan.target_elite, level: plan.target_level })];
    const masteries = (plan.target_skills ?? []).filter((s) => s.mastery_level > 0);
    if (masteries.length > 0) {
        parts.push(masteries.map((s) => t("planner.target.mastery", { skill: s.skill_index + 1, mastery: s.mastery_level })).join(" "));
    } else if (plan.target_skill_level > 1) {
        parts.push(t("planner.target.skillLevel", { level: plan.target_skill_level }));
    }
    const moduleCount = (plan.target_modules ?? []).filter((m) => m.module_stage > 0).length;
    if (moduleCount > 0) {
        parts.push(t("planner.target.modules", { count: moduleCount }));
    }
    return parts.join(t("planner.target.separator"));
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
