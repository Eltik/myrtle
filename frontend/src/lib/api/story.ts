import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { StoryArchive } from "#/types/generated/StoryArchive";
import type { StoryArtGallery } from "#/types/generated/StoryArtGallery";
import type { StoryArtKind } from "#/types/generated/StoryArtKind";
import type { StoryCommunity } from "#/types/generated/StoryCommunity";
import type { StoryGallery } from "#/types/generated/StoryGallery";
import type { StoryIllustrations } from "#/types/generated/StoryIllustrations";
import type { StoryIndex } from "#/types/generated/StoryIndex";
import type { StoryScript } from "#/types/generated/StoryScript";
import type { StorySpriteDetail } from "#/types/generated/StorySpriteDetail";
import type { StorySpriteIndex } from "#/types/generated/StorySpriteIndex";
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
 *
 * `null` when the running backend does not serve the route (404). Null is a first-class answer here: the endpoint shipped
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
 * route (404): a backend that predates the route yields a sheet with no
 * archive segment rather than a page that throws.
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
 * serve the route (404). The binary on :3060 can answer 404, so the tab
 * renders its own empty state rather than throwing a query error at the page.
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

/**
 * The CHARACTER GALLERY's list: every story sprite folder with the names the
 * scripts speak it under, or `null` when the running backend does not serve
 * the route (404): the binary on :3060 can predate the route, and a page that
 * threw would take the whole route down until a restart. The backend builds it
 * once per game data load, so an hour of staleness costs nothing.
 */
export const getStorySpritesFn = createServerFn({ method: "GET" })
    .inputValidator((server: string) => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(gamedataPath(server, "/story/sprites"));
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load story sprites: ${res.status}`);
        return (await res.json()) as StorySpriteIndex;
    });

export function storySpritesQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["story", "sprites", resolveGamedataServer(server)],
        queryFn: () => getStorySpritesFn({ data: resolveGamedataServer(server) }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/** One sprite folder's expression sheet, or `null` when the folder is unknown or the backend predates the route (both 404). */
export const getStorySpriteFn = createServerFn({ method: "GET" })
    .inputValidator((data: { base: string; server: string }) => data)
    .handler(async ({ data: { base, server } }) => {
        const res = await backendFetch(gamedataPath(server, `/story/sprites/${encodeURIComponent(base)}`));
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load story sprite: ${res.status}`);
        return (await res.json()) as StorySpriteDetail;
    });

export function storySpriteQueryOptions(base: string, server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["story", "sprite", resolveGamedataServer(server), base.toLowerCase()],
        queryFn: () => getStorySpriteFn({ data: { base, server: resolveGamedataServer(server) } }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/**
 * Every Archives gallery picture, by the event or Integrated Strategies theme whose
 * archive lists it, for the profile background picker. `null` when the backend
 * predates the route (404), as the sprite gallery is, so the picker can say so
 * rather than throw.
 */
export const getStoryGalleryFn = createServerFn({ method: "GET" })
    .inputValidator((server: string) => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(gamedataPath(server, "/story/gallery"));
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load the story gallery: ${res.status}`);
        return (await res.json()) as StoryGallery;
    });

/**
 * Every story CG or scene plate wide enough for a profile header (`GET /story/art-gallery/{kind}`),
 * filed under the first library group that draws it. Split per kind because the two payloads are
 * 59,576 and 54,787 JSON bytes on EN; a backend without the route answers 404, read as `null`.
 */
export const getStoryArtGalleryFn = createServerFn({ method: "GET" })
    .inputValidator((input: { server: string; kind: StoryArtKind }) => input)
    .handler(async ({ data: { server, kind } }) => {
        const res = await backendFetch(gamedataPath(server, `/story/art-gallery/${kind}`));
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load the story ${kind} gallery: ${res.status}`);
        return (await res.json()) as StoryArtGallery;
    });

export function storyArtGalleryQueryOptions(kind: StoryArtKind, server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["story", "art-gallery", kind, resolveGamedataServer(server)],
        queryFn: () => getStoryArtGalleryFn({ data: { server: resolveGamedataServer(server), kind } }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export function storyGalleryQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["story", "gallery", resolveGamedataServer(server)],
        queryFn: () => getStoryGalleryFn({ data: resolveGamedataServer(server) }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
