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
import type { PlanRecipe } from "#/types/generated/PlanRecipe";
import type { PlanRecipeCost } from "#/types/generated/PlanRecipeCost";
import type { PlanRequirementItem } from "#/types/generated/PlanRequirementItem";
import type { TargetModulePlan } from "#/types/generated/TargetModulePlan";
import type { TargetSkillPlan } from "#/types/generated/TargetSkillPlan";
import type { IOperatorListItem } from "#/types/operators";
import type { Refine } from "#/types/refine";

export type IPlanRecipeCost = PlanRecipeCost;

export type IPlanRecipe = PlanRecipe;

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

export interface IUpsertPlanInput {
    operatorId: string;
    targetElite: number;
    targetLevel: number;
    targetSkillLevel: number;
    targetSkills: { skill_index: number; mastery_level: number }[];
    targetModules: { module_id: string; module_stage: number }[];
    displayOnProfile: boolean;
    groups?: string[];
}

export const upsertPlanFn = createServerFn({ method: "POST" })
    .inputValidator((data: IUpsertPlanInput) => data)
    .handler(async ({ data }) => {
        const token = getCookie("site_token");
        if (!token) throw new Error("Not signed in.");
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
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to save plan: ${res.status}`);
        }
        const plan = (await res.json()) as IOperatorPlanResponse;
        if (plan.operator) {
            plan.operator = deepCamelize(plan.operator);
        }
        return plan;
    });

export interface IPlansQueryInput {
    activeIds?: string[];
    /** Highest material tier to keep; anything above is broken into its recipe ingredients. Unset keeps every tier. */
    maxTier?: number;
}

export const getPlansFn = createServerFn({ method: "GET" })
    .inputValidator((input?: IPlansQueryInput) => input)
    .handler(async ({ data: input }) => {
        const token = getCookie("site_token");
        if (!token) throw new Error("Not signed in.");
        const params = new URLSearchParams();
        if (input?.activeIds && input.activeIds.length > 0) params.set("active", input.activeIds.join(","));
        if (input?.maxTier) params.set("max_tier", String(input.maxTier));
        const query = params.toString();
        const res = await backendFetch(query ? `/plans?${query}` : "/plans", { bearerToken: token });
        if (!res.ok) throw new Error(`Failed to load plans: ${res.status}`);
        const data = (await res.json()) as IPlannerResponse;
        if (data.plans) {
            for (const p of data.plans) {
                if (p.operator) {
                    p.operator = deepCamelize(p.operator);
                }
            }
        }
        return data;
    });

export function plansQueryOptions(activeIds?: string[], maxTier?: number) {
    return queryOptions({
        queryKey: ["user", "plans", activeIds?.join(",") || "", maxTier ?? 0],
        queryFn: () => getPlansFn({ data: { activeIds, maxTier } }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const deletePlanFn = createServerFn({ method: "POST" })
    .inputValidator((operatorId: string) => operatorId)
    .handler(async ({ data: operatorId }) => {
        const token = getCookie("site_token");
        if (!token) throw new Error("Not signed in.");
        const res = await backendFetch(`/plan/${encodeURIComponent(operatorId)}`, {
            method: "DELETE",
            bearerToken: token,
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to delete plan: ${res.status}`);
        }
        return { success: true };
    });

export interface IUpsertGroupInput {
    oldName?: string;
    name: string;
}

export const upsertGroupFn = createServerFn({ method: "POST" })
    .inputValidator((data: IUpsertGroupInput) => data)
    .handler(async ({ data }) => {
        const token = getCookie("site_token");
        if (!token) throw new Error("Not signed in.");
        const url = data.oldName ? `/plan/group/${encodeURIComponent(data.oldName)}` : "/plan/group";
        const method = data.oldName ? "PUT" : "POST";
        const res = await backendFetch(url, {
            method,
            bearerToken: token,
            body: JSON.stringify({ name: data.name }),
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to save group: ${res.status}`);
        }
        return (await res.json()) as IPlanGroup;
    });

export interface IDeleteGroupInput {
    name: string;
}

export const deleteGroupFn = createServerFn({ method: "POST" })
    .inputValidator((data: IDeleteGroupInput) => data)
    .handler(async ({ data }) => {
        const token = getCookie("site_token");
        if (!token) throw new Error("Not signed in.");
        const res = await backendFetch(`/plan/group/${encodeURIComponent(data.name)}`, {
            method: "DELETE",
            bearerToken: token,
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to delete group: ${res.status}`);
        }
        return { success: true };
    });

// ---- Completed plans and pinned groups ----

/** Deletes several plans in one request. Ids with no plan are skipped, so `deleted` can be lower than the input. */
export const deletePlansFn = createServerFn({ method: "POST" })
    .inputValidator((operatorIds: string[]) => operatorIds)
    .handler(async ({ data: operatorIds }) => {
        const token = getCookie("site_token");
        if (!token) throw new Error("Not signed in.");
        const res = await backendFetch("/plans/delete", {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify({ operator_ids: operatorIds }),
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to delete plans: ${res.status}`);
        }
        return (await res.json()) as DeletePlansResponse;
    });

export interface ISetGroupPinnedInput {
    name: string;
    pinned: boolean;
}

export const setGroupPinnedFn = createServerFn({ method: "POST" })
    .inputValidator((data: ISetGroupPinnedInput) => data)
    .handler(async ({ data }) => {
        const token = getCookie("site_token");
        if (!token) throw new Error("Not signed in.");
        const res = await backendFetch(`/plan/group/${encodeURIComponent(data.name)}`, {
            method: "PUT",
            bearerToken: token,
            body: JSON.stringify({ pinned: data.pinned }),
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to pin group: ${res.status}`);
        }
        return (await res.json()) as IPlanGroup;
    });

// ---- end completed plans and pinned groups ----

export const getPublicPlansFn = createServerFn({ method: "GET" })
    .inputValidator((uid: string) => uid)
    .handler(async ({ data: uid }) => {
        const res = await backendFetch(`/plans/public?uid=${encodeURIComponent(uid)}`);
        if (!res.ok) throw new Error(`Failed to load public plans: ${res.status}`);
        const data = (await res.json()) as IOperatorPlanResponse[];
        for (const p of data) {
            if (p.operator) {
                p.operator = deepCamelize(p.operator);
            }
        }
        return data;
    });

export function publicPlansQueryOptions(uid: string) {
    return queryOptions({
        queryKey: ["user", "public-plans", uid],
        queryFn: () => getPublicPlansFn({ data: uid }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

// ---------------------------------------------------------------------------
// Plan presets: named, rarity-free targets the bulk-add dialog applies to many
// operators at once. Each operator is clamped client-side before its upsert.
// ---------------------------------------------------------------------------

/** Generated from `backend/src/database/models/planner.rs`. */
export type IPlanPreset = import("#/types/generated/PlanPreset").PlanPreset;

export type IPresetTarget = import("#/types/generated/PresetTarget").PresetTarget;

/** Not under `PLANS_QUERY_PREFIX`: saving a plan leaves the presets alone. */
export const PLAN_PRESETS_QUERY_KEY = ["user", "plan-presets"] as const;

export const getPlanPresetsFn = createServerFn({ method: "GET" }).handler(async () => {
    const token = getCookie("site_token");
    if (!token) throw new Error("Not signed in.");
    const res = await backendFetch("/plan/presets", { bearerToken: token });
    if (!res.ok) throw new Error(`Failed to load presets: ${res.status}`);
    return (await res.json()) as IPlanPreset[];
});

export function planPresetsQueryOptions() {
    return queryOptions({
        queryKey: PLAN_PRESETS_QUERY_KEY,
        queryFn: () => getPlanPresetsFn(),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
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
        const token = getCookie("site_token");
        if (!token) throw new Error("Not signed in.");
        const res = await backendFetch("/plan/preset", {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify({ name: data.name, target: data.target }),
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to save preset: ${res.status}`);
        }
        return (await res.json()) as IPlanPreset;
    });

export const deletePlanPresetFn = createServerFn({ method: "POST" })
    .inputValidator((name: string) => name)
    .handler(async ({ data: name }) => {
        const token = getCookie("site_token");
        if (!token) throw new Error("Not signed in.");
        const res = await backendFetch(`/plan/preset/${encodeURIComponent(name)}`, {
            method: "DELETE",
            bearerToken: token,
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to delete preset: ${res.status}`);
        }
        return { success: true };
    });
