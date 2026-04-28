import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";

export interface IMaterialItem {
    itemId: string;
    name: string;
    description: string;
    rarity: string;
    iconId: string;
    sortId: number;
    usage: string;
}

export interface IMaterials {
    items: Record<string, IMaterialItem>;
}

export const getMaterialsFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/materials");
    if (!res.ok) throw new Error(`Failed to load materials: ${res.status}`);
    return (await res.json()) as IMaterials;
});

export function materialsQueryOptions() {
    return queryOptions({
        queryKey: ["materials"],
        queryFn: () => getMaterialsFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
