import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { IOperatorIndexEntry, IOperatorListItem, IOperatorsStaticMap } from "#/types/operators";

export const getOperatorsIndexFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/operators/index");
    if (!res.ok) throw new Error(`Failed to load operators index: ${res.status}`);
    return (await res.json()) as IOperatorIndexEntry[];
});

export function operatorsIndexQueryOptions() {
    return queryOptions({
        queryKey: ["operators", "index"],
        queryFn: () => getOperatorsIndexFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const getOperatorsListFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/operators");
    if (!res.ok) throw new Error(`Failed to load operators: ${res.status}`);
    const data = (await res.json()) as IOperatorsStaticMap;
    return Object.values(data) as IOperatorListItem[];
});

export function operatorsListQueryOptions() {
    return queryOptions({
        queryKey: ["operators", "list"],
        queryFn: () => getOperatorsListFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
