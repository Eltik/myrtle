import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { getCookie } from "@tanstack/react-start/server";
import { deepCamelize } from "#/lib/api/operators";
import { backendFetch } from "#/lib/fetch";
// Generated from `backend/src/database/models/planner.rs`.
// To change a field, edit the Rust struct and run `bun run gen:types`.
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

export const getPlansFn = createServerFn({ method: "GET" })
    .inputValidator((activeIds?: string[]) => activeIds)
    .handler(async ({ data: activeIds }) => {
        const token = getCookie("site_token");
        if (!token) throw new Error("Not signed in.");
        const url = activeIds && activeIds.length > 0 ? `/plans?active=${encodeURIComponent(activeIds.join(","))}` : "/plans";
        const res = await backendFetch(url, { bearerToken: token });
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

export function plansQueryOptions(activeIds?: string[]) {
    return queryOptions({
        queryKey: ["user", "plans", activeIds?.join(",") || ""],
        queryFn: () => getPlansFn({ data: activeIds }),
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
