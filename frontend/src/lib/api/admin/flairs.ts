import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import { parseError } from "../_shared";
import { requireSiteToken } from "../_shared.server";
import type { ITierListFlair } from "../tier-lists";

export interface ICreateTierListFlairInput {
    code: string;
    label: string;
    color?: string | null;
    displayOrder?: number;
}

interface IBackendFlair {
    id: number;
    code: string;
    label: string;
    color: string | null;
    display_order: number;
    is_active: boolean;
}

export const createTierListFlairFn = createServerFn({ method: "POST" })
    .inputValidator((data: ICreateTierListFlairInput) => data)
    .handler(async ({ data }): Promise<ITierListFlair> => {
        const token = requireSiteToken();
        const res = await backendFetch("/tier-list-flairs", {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify({
                code: data.code,
                label: data.label,
                color: data.color ?? null,
                display_order: data.displayOrder ?? 0,
            }),
        });
        if (!res.ok) throw await parseError(res);
        const raw = (await res.json()) as IBackendFlair;
        return {
            id: raw.id,
            code: raw.code,
            label: raw.label,
            color: raw.color,
            displayOrder: raw.display_order,
            isActive: raw.is_active,
        };
    });
