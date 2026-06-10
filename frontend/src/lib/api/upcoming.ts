import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { IOperatorIndexEntry } from "#/types/operators";

export const getUpcomingFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/upcoming");
    if (!res.ok) throw new Error(`Failed to load upcoming operators: ${res.status}`);
    return (await res.json()) as IOperatorIndexEntry[];
});

export function upcomingQueryOptions() {
    return queryOptions({
        queryKey: ["upcoming"],
        queryFn: () => getUpcomingFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
