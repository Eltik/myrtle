import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { getCookie } from "@tanstack/react-start/server";
import { backendFetch } from "#/lib/fetch";

export interface IPlanRecipeCost {
    count: number;
    item: IPlanRequirementItem;
}

export interface IPlanRecipe {
    count: number;
    costs: IPlanRecipeCost[];
}

export interface IPlanRequirementItem {
    id: string;
    name: string;
    iconId: string | null;
    image: string | null;
    itemType: string;
    rarity: number;
    sortGroup: number;
    sortSubrank: number;
    requiredCount: number;
    inventoryCount: number;
    craftableCount: number;
    missingCount: number;
    canCraft: boolean;
    craftReason: string;
    recipe: IPlanRecipe | null;
}

export interface IOperatorPlanResponse {
    id: string;
    user_id: string;
    operator_id: string;
    target_elite: number;
    target_level: number;
    target_skill_level: number;
    target_skills: { skill_index: number; mastery_level: number }[];
    target_modules: { module_id: string; module_stage: number }[];
    display_on_profile: boolean;
    created_at: string;
    updated_at: string;
    groups: string[];
}

export interface IPlanGroup {
    id: string;
    user_id: string;
    name: string;
    created_at: string;
    updated_at: string;
}

export interface IPlannerResponse {
    plans: IOperatorPlanResponse[];
    aggregatedRequirements: IPlanRequirementItem[];
    groups: IPlanGroup[];
}

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
        return (await res.json()) as IOperatorPlanResponse;
    });

export const getPlansFn = createServerFn({ method: "GET" })
    .inputValidator((activeIds?: string[]) => activeIds)
    .handler(async ({ data: activeIds }) => {
        const token = getCookie("site_token");
        if (!token) throw new Error("Not signed in.");
        const url = activeIds && activeIds.length > 0 ? `/plans?active=${encodeURIComponent(activeIds.join(","))}` : "/plans";
        const res = await backendFetch(url, { bearerToken: token });
        if (!res.ok) throw new Error(`Failed to load plans: ${res.status}`);
        return (await res.json()) as IPlannerResponse;
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
        return (await res.json()) as IOperatorPlanResponse[];
    });

export function publicPlansQueryOptions(uid: string) {
    return queryOptions({
        queryKey: ["user", "public-plans", uid],
        queryFn: () => getPublicPlansFn({ data: uid }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}
