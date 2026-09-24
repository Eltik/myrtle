import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { StoryArchive } from "#/types/generated/StoryArchive";
import type { StoryCommunity } from "#/types/generated/StoryCommunity";
import type { StoryIllustrations } from "#/types/generated/StoryIllustrations";
import type { StoryIndex } from "#/types/generated/StoryIndex";
import type { StoryScript } from "#/types/generated/StoryScript";
import { DEFAULT_GAMEDATA_SERVER, gamedataPath, resolveGamedataServer } from "./gamedata";

/** The Archives library: every story group and every operator's records. See `docs/story-reader.md`. */
export const getStoryIndexFn = createServerFn({ method: "GET" })
    .inputValidator((server: string) => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(gamedataPath(server, "/story/index"));
        if (!res.ok) throw new Error(`Failed to load story index: ${res.status}`);
        return (await res.json()) as StoryIndex;
    });

export function storyIndexQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["story", "index", resolveGamedataServer(server)],
        queryFn: () => getStoryIndexFn({ data: resolveGamedataServer(server) }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/** One parsed script, or `null` when the library lists the id but no script file backs it (404). */
export const getStoryFn = createServerFn({ method: "GET" })
    .inputValidator((data: { id: string; server: string }) => data)
    .handler(async ({ data: { id, server } }) => {
        const res = await backendFetch(gamedataPath(server, `/story/${encodeURIComponent(id)}`));
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load story: ${res.status}`);
        return (await res.json()) as StoryScript;
    });

export function storyQueryOptions(id: string, server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["story", "script", resolveGamedataServer(server), id],
        queryFn: () => getStoryFn({ data: { id, server: resolveGamedataServer(server) } }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/**
 * One group's illustrations, or one operator's: the backend takes a
 * `StoryGroup.id` and an operator `charId` on the SAME route, because an
 * operator's records are split over one or two story sets and have no single
 * group id of their own. The shape is the generated binding, fixed in
 * `docs/story-reader.md` section 2.
 */
/**
 * A group's illustrations, or `null` when the running backend does not serve
 * the route (404). Null is a first-class answer here: the endpoint shipped
 * after this tab did, and a page that threw would take the whole library down
 * with it until a restart.
 */
export const getStoryIllustrationsFn = createServerFn({ method: "GET" })
    .inputValidator((data: { groupId: string; server: string }) => data)
    .handler(async ({ data: { groupId, server } }) => {
        const res = await backendFetch(gamedataPath(server, `/story/group/${encodeURIComponent(groupId)}/illustrations`));
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load illustrations: ${res.status}`);
        return (await res.json()) as StoryIllustrations;
    });

export function storyIllustrationsQueryOptions(groupId: string, server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["story", "illustrations", resolveGamedataServer(server), groupId],
        queryFn: () => getStoryIllustrationsFn({ data: { groupId, server: resolveGamedataServer(server) } }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/**
 * One group's ARCHIVE, or `null` when the running backend does not serve the
 * route (404). Null is the same first-class answer the illustrations query
 * gives, and for the same reason: the route shipped after the library did, so
 * a backend that predates it must yield a sheet with no archive segment rather
 * than a page that throws.
 *
 * A group that simply kept no archive is NOT a 404: the backend answers 200
 * with an empty `sections`, and 81 of the 87 EN groups look like that.
 */
export const getStoryArchiveFn = createServerFn({ method: "GET" })
    .inputValidator((data: { groupId: string; server: string }) => data)
    .handler(async ({ data: { groupId, server } }) => {
        const res = await backendFetch(gamedataPath(server, `/story/group/${encodeURIComponent(groupId)}/archive`));
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load archive: ${res.status}`);
        return (await res.json()) as StoryArchive;
    });

export function storyArchiveQueryOptions(groupId: string, server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["story", "archive", resolveGamedataServer(server), groupId],
        queryFn: () => getStoryArchiveFn({ data: { groupId, server: resolveGamedataServer(server) } }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/**
 * What the COMMUNITY has read, or `null` when the running backend does not
 * serve the route (404). It is the same first-class null the archive and the
 * illustrations queries give, and it is not hypothetical here: the binary on
 * :3060 while this shipped answers 404, so the tab must render its own empty
 * state rather than throw a query error at the page.
 *
 * The backend caches the aggregate for 6 hours, so a shorter staleTime here
 * would only re-fetch a document that cannot have changed.
 */
export const getStoryCommunityFn = createServerFn({ method: "GET" })
    .inputValidator((server: string) => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(gamedataPath(server, "/story/community"));
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load community reading: ${res.status}`);
        return (await res.json()) as StoryCommunity;
    });

export function storyCommunityQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["story", "community", resolveGamedataServer(server)],
        queryFn: () => getStoryCommunityFn({ data: resolveGamedataServer(server) }),
        staleTime: 6 * 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
