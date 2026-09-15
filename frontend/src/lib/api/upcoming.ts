import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { IOperatorIndexEntry } from "#/types/operators";
import { DEFAULT_GAMEDATA_SERVER, gamedataKey, gamedataPath, resolveGamedataServer } from "./gamedata";

export const getUpcomingFn = createServerFn({ method: "GET" })
    .inputValidator((server: string | undefined) => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(gamedataPath(server, "/upcoming"));
        if (!res.ok) throw new Error(`Failed to load upcoming operators: ${res.status}`);
        return (await res.json()) as IOperatorIndexEntry[];
    });

export function upcomingQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["upcoming", ...gamedataKey(server)],
        queryFn: () => getUpcomingFn({ data: resolveGamedataServer(server) }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
