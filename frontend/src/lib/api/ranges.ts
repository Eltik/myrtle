import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";

export interface IRangeGrid {
    col: number;
    row: number;
}

export interface IRange {
    id: string;
    direction: number;
    grids: IRangeGrid[];
}

export type IRangesMap = Record<string, IRange>;

export const getRangesFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/ranges");
    if (!res.ok) throw new Error(`Failed to load ranges: ${res.status}`);
    return (await res.json()) as IRangesMap;
});

export function rangesQueryOptions() {
    return queryOptions({
        queryKey: ["ranges"],
        queryFn: () => getRangesFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
