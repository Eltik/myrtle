import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { type LiveItem, liveCandidates, namedFeatured } from "#/components/home/impl/live";
import { backendFetch } from "#/lib/fetch";
import { parseOperatorName } from "#/lib/utils";
import type { ActivityBasicInfo } from "#/types/generated/ActivityBasicInfo";
import type { GachaPoolClient } from "#/types/generated/GachaPoolClient";
import type { IOperatorIndexEntry } from "#/types/operators";
import { DEFAULT_GAMEDATA_SERVER, gamedataKey, gamedataPath, resolveGamedataServer } from "./gamedata";

/** What the home page's "Happening now" needs, and nothing else. */
export interface ILiveFeed {
    /** Every not-yet-ended event and banner, soonest end first; the client picks the running ones. */
    items: LiveItem[];
    /** Display names for the featured operators those cards name, by char id. A missing id shows as itself. */
    names: Record<string, string>;
}

async function fetchJson<T>(server: string, path: string): Promise<T> {
    const res = await backendFetch(gamedataPath(server, path));
    if (!res.ok) throw new Error(`Failed to load ${path}: ${res.status}`);
    return (await res.json()) as T;
}

/**
 * The live items, trimmed server-side. The full activity and banner tables run
 * to about 0.4 to 0.6 MB per server and the operator index to about 0.45 MB;
 * shipping them to the home page only to keep three cards would put all of it
 * in the dehydrated HTML. The names are resolved here so the client needs no
 * index; an index that fails to load costs the names, not the section.
 */
export const getLiveFeedFn = createServerFn({ method: "GET" })
    .inputValidator((server: string | undefined) => resolveGamedataServer(server))
    .handler(async ({ data: server }): Promise<ILiveFeed> => {
        const [activities, pools, index] = await Promise.all([fetchJson<Record<string, ActivityBasicInfo>>(server, "/static/activities"), fetchJson<GachaPoolClient[]>(server, "/static/banners"), fetchJson<IOperatorIndexEntry[]>(server, "/operators/index").catch(() => [] as IOperatorIndexEntry[])]);
        const items = liveCandidates(Object.values(activities), pools, server, Date.now() / 1000);
        const wanted = new Set(items.flatMap((item) => namedFeatured(item)));
        const names: Record<string, string> = {};
        for (const entry of index) {
            if (wanted.has(entry.id)) names[entry.id] = parseOperatorName(entry.name).displayName;
        }
        return { items, names };
    });

export function liveFeedQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["home", "live", ...gamedataKey(server)],
        queryFn: () => getLiveFeedFn({ data: resolveGamedataServer(server) }),
        // The feed holds every window that has not ended, so a stale copy stays
        // right as time passes; a refetch only picks up gamedata changes.
        staleTime: 10 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}
