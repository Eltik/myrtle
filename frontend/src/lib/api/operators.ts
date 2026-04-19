import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";

export type OperatorProfession = "MEDIC" | "CASTER" | "WARRIOR" | "PIONEER" | "SNIPER" | "SPECIAL" | "SUPPORT" | "TANK" | "TOKEN" | "TRAP";
export type OperatorPosition = "MELEE" | "RANGED" | "ALL" | "NONE";

export interface OperatorIndexEntry {
    id: string;
    name: string;
    appellation: string;
    rarity: 1 | 2 | 3 | 4 | 5 | 6;
    profession: OperatorProfession;
    subProfessionId: string;
    position: OperatorPosition;
    tagList: string[];
    nationId: string;
    isNotObtainable: boolean;
}

export const getOperatorsIndexFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/operators/index");
    if (!res.ok) throw new Error(`Failed to load operators index: ${res.status}`);
    return (await res.json()) as OperatorIndexEntry[];
});

export function operatorsIndexQueryOptions() {
    return queryOptions({
        queryKey: ["operators", "index"],
        queryFn: () => getOperatorsIndexFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
