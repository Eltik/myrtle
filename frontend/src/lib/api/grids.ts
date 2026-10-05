import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { getCookie } from "@tanstack/react-start/server";
import { GRID_DESCRIPTION_MAX, GRID_LABEL_MAX, GRID_TITLE_MAX, GRIDS_PER_PAGE } from "#/components/grids/shared";
import { backendFetch } from "#/lib/fetch";
import { sanitizeMarkdownForStorage, sanitizePlainName } from "#/lib/markdown/sanitize-input";
import type { Grid } from "#/types/generated/Grid";
import type { GridInput } from "#/types/generated/GridInput";
import type { GridListResponse } from "#/types/generated/GridListResponse";
import type { GridSummary } from "#/types/generated/GridSummary";
import { parseError } from "./_shared";
import { optionalSiteToken, requireSiteToken } from "./_shared.server";
import { DEFAULT_GAMEDATA_SERVER, gamedataKey, resolveGamedataServer } from "./gamedata";

// Grids: an R x C board of labelled cells, each optionally holding one entity
// of any tier-list kind. The wire types are the generated ones; the backend
// resolves each cell's entity against `?server=`, so a cell's `entity` is the
// same `EntitySummary` a tier list placement carries. A grid is read and saved
// WHOLE: there are no per-cell endpoints.

export type IGrid = Grid;
export type IGridSummary = GridSummary;
export type IGridListResponse = GridListResponse;

export type GridSort = "recent" | "popular";

export interface IBrowseGridsParams {
    sort: GridSort;
    q: string;
    page: number;
    server?: string;
}

function serverQuery(server: string | undefined): string {
    return `server=${encodeURIComponent(resolveGamedataServer(server))}`;
}

/** The response body of a successful request; the backend's error otherwise. */
async function readJSON<T>(res: Response): Promise<T> {
    if (!res.ok) throw await parseError(res);
    return (await res.json()) as T;
}

/** The same sanitizers a tier list's name and description go through, applied to the title, the description and every label. */
function sanitizeGridInput(input: GridInput): GridInput {
    return {
        title: sanitizePlainName(input.title, GRID_TITLE_MAX),
        description: sanitizeMarkdownForStorage(input.description, { maxLength: GRID_DESCRIPTION_MAX, nullOnEmpty: true }) || null,
        rows: input.rows,
        cols: input.cols,
        cells: input.cells.map((cell) => ({
            label: sanitizePlainName(cell.label, GRID_LABEL_MAX),
            // Both or neither, as the backend validates.
            entity_kind: cell.entity_kind && cell.entity_id ? cell.entity_kind : null,
            entity_id: cell.entity_kind && cell.entity_id ? cell.entity_id : null,
        })),
        is_listed: input.is_listed,
        entity_kinds: input.entity_kinds,
    };
}

export const browseGridsFn = createServerFn({ method: "GET" })
    .inputValidator((data: IBrowseGridsParams) => data)
    .handler(async ({ data }): Promise<IGridListResponse> => {
        const params = new URLSearchParams({ sort: data.sort, page: String(Math.max(1, data.page)), per_page: String(GRIDS_PER_PAGE), server: resolveGamedataServer(data.server) });
        if (data.q.trim()) params.set("q", data.q.trim());
        const res = await backendFetch(`/grids?${params.toString()}`);
        return readJSON<IGridListResponse>(res);
    });

export function browseGridsQueryOptions(params: IBrowseGridsParams) {
    const server = resolveGamedataServer(params.server);
    return queryOptions({
        queryKey: ["grids", "browse", params.sort, params.q.trim(), params.page, ...gamedataKey(server)],
        queryFn: () => browseGridsFn({ data: { ...params, server } }),
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const getMyGridsFn = createServerFn({ method: "GET" })
    .inputValidator((server: string | undefined) => server)
    .handler(async ({ data: server }): Promise<IGridSummary[]> => {
        const token = getCookie("site_token");
        if (!token) return [];
        const res = await backendFetch(`/grids/mine?${serverQuery(server)}`, { bearerToken: token });
        if (res.status === 401) return [];
        return readJSON<IGridSummary[]>(res);
    });

/** `viewerId` is the signed-in user's id, or `null`: it keys the cache so one account's list never shows to another. */
export function myGridsQueryOptions(viewerId: string | null, server: string = DEFAULT_GAMEDATA_SERVER) {
    const authed = viewerId !== null;
    return queryOptions({
        queryKey: ["grids", "mine", viewerId ?? "anon", ...gamedataKey(server)],
        queryFn: () => (authed ? getMyGridsFn({ data: resolveGamedataServer(server) }) : Promise.resolve([] as IGridSummary[])),
        enabled: authed,
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

/** Sends the viewer's token when there is one, so `can_edit` is the viewer's. `null` on 404. */
export const getGridFn = createServerFn({ method: "GET" })
    .inputValidator((data: { slug: string; server?: string }) => data)
    .handler(async ({ data: { slug, server } }): Promise<IGrid | null> => {
        const res = await backendFetch(`/grids/${encodeURIComponent(slug)}?${serverQuery(server)}`, { bearerToken: optionalSiteToken() });
        if (res.status === 404) return null;
        return readJSON<IGrid>(res);
    });

/** `viewerId` keys the cache by who is reading, so `can_edit` refreshes on sign-in and sign-out. */
export function gridQueryOptions(slug: string, viewerId: string | null, server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["grids", "detail", slug, viewerId ?? "anon", ...gamedataKey(server)],
        queryFn: () => getGridFn({ data: { slug, server: resolveGamedataServer(server) } }),
        staleTime: 30 * 1000,
        gcTime: 10 * 60 * 1000,
    });
}

export const createGridFn = createServerFn({ method: "POST" })
    .inputValidator((data: { input: GridInput; server?: string }) => data)
    .handler(async ({ data: { input, server } }): Promise<IGrid> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/grids?${serverQuery(server)}`, { method: "POST", bearerToken: token, body: JSON.stringify(sanitizeGridInput(input)) });
        return readJSON<IGrid>(res);
    });

export const updateGridFn = createServerFn({ method: "POST" })
    .inputValidator((data: { slug: string; input: GridInput; server?: string }) => data)
    .handler(async ({ data: { slug, input, server } }): Promise<IGrid> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/grids/${encodeURIComponent(slug)}?${serverQuery(server)}`, { method: "PUT", bearerToken: token, body: JSON.stringify(sanitizeGridInput(input)) });
        return readJSON<IGrid>(res);
    });

export const deleteGridFn = createServerFn({ method: "POST" })
    .inputValidator((slug: string) => slug)
    .handler(async ({ data: slug }): Promise<null> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/grids/${encodeURIComponent(slug)}`, { method: "DELETE", bearerToken: token });
        if (!res.ok) throw await parseError(res);
        return null;
    });

/** "Use this template": a copy owned by the caller, same title, description, size and labels, every pick cleared. */
export const forkGridFn = createServerFn({ method: "POST" })
    .inputValidator((data: { slug: string; server?: string }) => data)
    .handler(async ({ data: { slug, server } }): Promise<IGrid> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/grids/${encodeURIComponent(slug)}/fork?${serverQuery(server)}`, { method: "POST", bearerToken: token });
        return readJSON<IGrid>(res);
    });
