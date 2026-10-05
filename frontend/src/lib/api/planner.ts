import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { getCookie } from "@tanstack/react-start/server";
import { deepCamelize } from "#/lib/api/operators";
import { backendFetch } from "#/lib/fetch";
// Generated from `backend/src/database/models/planner.rs`.
// To change a field, edit the Rust struct and run `bun run gen:types`.
import type { DeletePlansResponse } from "#/types/generated/DeletePlansResponse";
import type { OperatorPlanResponse } from "#/types/generated/OperatorPlanResponse";
import type { PlanGroup } from "#/types/generated/PlanGroup";
import type { PlannerResponse } from "#/types/generated/PlannerResponse";
import type { PlanPreset } from "#/types/generated/PlanPreset";
import type { PlanRequirementItem } from "#/types/generated/PlanRequirementItem";
import type { PresetTarget } from "#/types/generated/PresetTarget";
import type { TargetModulePlan } from "#/types/generated/TargetModulePlan";
import type { TargetSkillPlan } from "#/types/generated/TargetSkillPlan";
import type { IOperatorListItem } from "#/types/operators";
import type { Refine } from "#/types/refine";

export type IPlanRequirementItem = PlanRequirementItem;

/**
 * `target_skills` / `target_modules` are jsonb columns, so Rust types them as
 * `serde_json::Value`; their real shape is `TargetSkillPlan` / `TargetModulePlan`,
 * which the same module already generates. `operator` is an operator payload the
 * fetch site runs through `deepCamelize` before anything reads it.
 */
export type IOperatorPlanResponse = Refine<OperatorPlanResponse, { operator: IOperatorListItem; target_skills: TargetSkillPlan[]; target_modules: TargetModulePlan[] }>;

export type IPlanGroup = PlanGroup;

export type IPlannerResponse = Refine<PlannerResponse, { plans: IOperatorPlanResponse[] }>;

/** A named, rarity-free target the bulk-add dialog applies to many operators at once. */
export type IPlanPreset = PlanPreset;

export type IPresetTarget = PresetTarget;

/**
 * The prefix every planner mutation invalidates. It is a prefix on purpose:
 * `plansQueryOptions` appends the active-plan filter, and every filtered copy
 * of the plan list has to refetch.
 */
export const PLANS_QUERY_PREFIX = ["user", "plans"] as const;

/** Not under `PLANS_QUERY_PREFIX`: saving a plan leaves the presets alone. */
export const PLAN_PRESETS_QUERY_KEY = ["user", "plan-presets"] as const;

const STALE_TIME = 60 * 1000;
const GC_TIME = 5 * 60 * 1000;

/** The signed-in player's session token; every planner call except the public list needs one. */
function siteToken(): string {
    const token = getCookie("site_token");
    if (!token) throw new Error("Not signed in.");
    return token;
}

/** Throws the backend's error body, or `<failure>: <status>` when the body is empty. */
async function throwUnlessOk(res: Response, failure: string): Promise<void> {
    if (res.ok) return;
    const text = await res.text().catch(() => "");
    throw new Error(text || `${failure}: ${res.status}`);
}

/** The backend sends the operator in snake_case; everything that reads a plan expects camelCase. */
function camelizeOperator(plan: IOperatorPlanResponse): IOperatorPlanResponse {
    if (plan.operator) plan.operator = deepCamelize(plan.operator);
    return plan;
}

// ---- Plans ----

export interface IUpsertPlanInput {
    operatorId: string;
    targetElite: number;
    targetLevel: number;
    targetSkillLevel: number;
    targetSkills: TargetSkillPlan[];
    targetModules: TargetModulePlan[];
    displayOnProfile: boolean;
    groups?: string[];
}

export const upsertPlanFn = createServerFn({ method: "POST" })
    .inputValidator((data: IUpsertPlanInput) => data)
    .handler(async ({ data }) => {
        const token = siteToken();
        const payload = {
            target_elite: data.targetElite,
            target_level: data.targetLevel,
            target_skill_level: data.targetSkillLevel,
            target_skills: data.targetSkills,
            target_modules: data.targetModules,
            display_on_profile: data.displayOnProfile,
            groups: data.groups,
        };
        const res = await backendFetch(`/plan/${encodeURIComponent(data.operatorId)}`, {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify(payload),
        });
        await throwUnlessOk(res, "Failed to save plan");
        return camelizeOperator((await res.json()) as IOperatorPlanResponse);
    });

export interface IPlansQueryInput {
    activeIds?: string[];
    /** Highest material tier to keep; anything above is broken into its recipe ingredients. Unset keeps every tier. */
    maxTier?: number;
}

export const getPlansFn = createServerFn({ method: "GET" })
    .inputValidator((input?: IPlansQueryInput) => input)
    .handler(async ({ data: input }) => {
        const token = siteToken();
        const params = new URLSearchParams();
        if (input?.activeIds && input.activeIds.length > 0) params.set("active", input.activeIds.join(","));
        if (input?.maxTier) params.set("max_tier", String(input.maxTier));
        const query = params.toString();
        const res = await backendFetch(query ? `/plans?${query}` : "/plans", { bearerToken: token });
        if (!res.ok) throw new Error(`Failed to load plans: ${res.status}`);
        const data = (await res.json()) as IPlannerResponse;
        data.plans?.forEach(camelizeOperator);
        return data;
    });

export function plansQueryOptions(activeIds?: string[], maxTier?: number) {
    return queryOptions({
        queryKey: [...PLANS_QUERY_PREFIX, activeIds?.join(",") || "", maxTier ?? 0],
        queryFn: () => getPlansFn({ data: { activeIds, maxTier } }),
        staleTime: STALE_TIME,
        gcTime: GC_TIME,
    });
}

export const deletePlanFn = createServerFn({ method: "POST" })
    .inputValidator((operatorId: string) => operatorId)
    .handler(async ({ data: operatorId }) => {
        const token = siteToken();
        const res = await backendFetch(`/plan/${encodeURIComponent(operatorId)}`, {
            method: "DELETE",
            bearerToken: token,
        });
        await throwUnlessOk(res, "Failed to delete plan");
        return { success: true };
    });

/** Deletes several plans in one request. Ids with no plan are skipped, so `deleted` can be lower than the input. */
export const deletePlansFn = createServerFn({ method: "POST" })
    .inputValidator((operatorIds: string[]) => operatorIds)
    .handler(async ({ data: operatorIds }) => {
        const token = siteToken();
        const res = await backendFetch("/plans/delete", {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify({ operator_ids: operatorIds }),
        });
        await throwUnlessOk(res, "Failed to delete plans");
        return (await res.json()) as DeletePlansResponse;
    });

export const getPublicPlansFn = createServerFn({ method: "GET" })
    .inputValidator((uid: string) => uid)
    .handler(async ({ data: uid }) => {
        const res = await backendFetch(`/plans/public?uid=${encodeURIComponent(uid)}`);
        if (!res.ok) throw new Error(`Failed to load public plans: ${res.status}`);
        const data = (await res.json()) as IOperatorPlanResponse[];
        data.forEach(camelizeOperator);
        return data;
    });

export function publicPlansQueryOptions(uid: string) {
    return queryOptions({
        queryKey: ["user", "public-plans", uid],
        queryFn: () => getPublicPlansFn({ data: uid }),
        staleTime: STALE_TIME,
        gcTime: GC_TIME,
    });
}

// ---- Groups ----

export interface IUpsertGroupInput {
    /** Set to rename that group; unset creates `name`. */
    oldName?: string;
    name: string;
}

export const upsertGroupFn = createServerFn({ method: "POST" })
    .inputValidator((data: IUpsertGroupInput) => data)
    .handler(async ({ data }) => {
        const token = siteToken();
        const url = data.oldName ? `/plan/group/${encodeURIComponent(data.oldName)}` : "/plan/group";
        const method = data.oldName ? "PUT" : "POST";
        const res = await backendFetch(url, {
            method,
            bearerToken: token,
            body: JSON.stringify({ name: data.name }),
        });
        await throwUnlessOk(res, "Failed to save group");
        return (await res.json()) as IPlanGroup;
    });

export interface ISetGroupPinnedInput {
    name: string;
    pinned: boolean;
}

export const setGroupPinnedFn = createServerFn({ method: "POST" })
    .inputValidator((data: ISetGroupPinnedInput) => data)
    .handler(async ({ data }) => {
        const token = siteToken();
        const res = await backendFetch(`/plan/group/${encodeURIComponent(data.name)}`, {
            method: "PUT",
            bearerToken: token,
            body: JSON.stringify({ pinned: data.pinned }),
        });
        await throwUnlessOk(res, "Failed to pin group");
        return (await res.json()) as IPlanGroup;
    });

export interface IDeleteGroupInput {
    name: string;
}

export const deleteGroupFn = createServerFn({ method: "POST" })
    .inputValidator((data: IDeleteGroupInput) => data)
    .handler(async ({ data }) => {
        const token = siteToken();
        const res = await backendFetch(`/plan/group/${encodeURIComponent(data.name)}`, {
            method: "DELETE",
            bearerToken: token,
        });
        await throwUnlessOk(res, "Failed to delete group");
        return { success: true };
    });

// ---- Presets ----
// Each operator is clamped client-side (`bulkTargets.ts`) before its upsert.

export const getPlanPresetsFn = createServerFn({ method: "GET" }).handler(async () => {
    const token = siteToken();
    const res = await backendFetch("/plan/presets", { bearerToken: token });
    if (!res.ok) throw new Error(`Failed to load presets: ${res.status}`);
    return (await res.json()) as IPlanPreset[];
});

export function planPresetsQueryOptions() {
    return queryOptions({
        queryKey: PLAN_PRESETS_QUERY_KEY,
        queryFn: () => getPlanPresetsFn(),
        staleTime: STALE_TIME,
        gcTime: GC_TIME,
    });
}

export interface IUpsertPlanPresetInput {
    name: string;
    target: IPresetTarget;
}

/** Saves a preset; a preset with the same name is replaced. */
export const upsertPlanPresetFn = createServerFn({ method: "POST" })
    .inputValidator((data: IUpsertPlanPresetInput) => data)
    .handler(async ({ data }) => {
        const token = siteToken();
        const res = await backendFetch("/plan/preset", {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify({ name: data.name, target: data.target }),
        });
        await throwUnlessOk(res, "Failed to save preset");
        return (await res.json()) as IPlanPreset;
    });

export const deletePlanPresetFn = createServerFn({ method: "POST" })
    .inputValidator((name: string) => name)
    .handler(async ({ data: name }) => {
        const token = siteToken();
        const res = await backendFetch(`/plan/preset/${encodeURIComponent(name)}`, {
            method: "DELETE",
            bearerToken: token,
        });
        await throwUnlessOk(res, "Failed to delete preset");
        return { success: true };
    });
