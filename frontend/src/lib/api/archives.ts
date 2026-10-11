import { queryOptions } from "@tanstack/react-query";
import { backendFetch } from "#/lib/fetch";
// Generated from `backend/src/core/archives/dto.rs`.
// To change a field, edit the Rust struct and run `bun run gen:types`.
import type { ArchiveDemographics } from "#/types/generated/ArchiveDemographics";
import type { ArchiveDossiers } from "#/types/generated/ArchiveDossiers";
import type { ArchiveHiddenFactions } from "#/types/generated/ArchiveHiddenFactions";
import type { ArchiveNetwork } from "#/types/generated/ArchiveNetwork";
import type { ArchiveOperator } from "#/types/generated/ArchiveOperator";
import { DEFAULT_GAMEDATA_SERVER, gamedataPath, resolveGamedataServer } from "./gamedata";

/** The Archives are built once per game-data load, so an hour is the floor, not a guess at freshness. */
const STALE = 60 * 60 * 1000;
const GC = 24 * 60 * 60 * 1000;

/**
 * Read straight from the backend, by the browser as by SSR (`backendFetch` picks the
 * origin), the way `fetchOperatorsRoster` does. These used to go through server
 * functions: the SSR server fetched the JSON uncompressed, re-encoded it as seroval
 * and sent that to the browser uncompressed, 470,815 bytes for the network where the
 * backend answers 26,733 bytes of brotli. The backend serves each list once per
 * game-data load with an `ETag` and `Cache-Control: public, max-age=300`.
 */
async function getJson<T>(server: string, path: string): Promise<T> {
    const res = await backendFetch(gamedataPath(server, path));
    if (!res.ok) throw new Error(`Failed to load ${path}: ${res.status}`);
    return (await res.json()) as T;
}

/** One personnel file, or `null` for an id with no file (404). */
async function getArchiveOperator(id: string, server: string): Promise<ArchiveOperator | null> {
    const res = await backendFetch(gamedataPath(server, `/archives/operator/${encodeURIComponent(id)}`));
    if (res.status === 404) return null;
    if (!res.ok) throw new Error(`Failed to load personnel file: ${res.status}`);
    return (await res.json()) as ArchiveOperator;
}

export function archiveNetworkQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    const srv = resolveGamedataServer(server);
    return queryOptions({ queryKey: ["archives", "network", srv], queryFn: () => getJson<ArchiveNetwork>(srv, "/archives/network"), staleTime: STALE, gcTime: GC });
}

export function archiveDossiersQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    const srv = resolveGamedataServer(server);
    return queryOptions({ queryKey: ["archives", "dossiers", srv], queryFn: () => getJson<ArchiveDossiers>(srv, "/archives/operators"), staleTime: STALE, gcTime: GC });
}

export function archiveDemographicsQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    const srv = resolveGamedataServer(server);
    return queryOptions({ queryKey: ["archives", "demographics", srv], queryFn: () => getJson<ArchiveDemographics>(srv, "/archives/demographics"), staleTime: STALE, gcTime: GC });
}

export function archiveHiddenFactionsQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    const srv = resolveGamedataServer(server);
    return queryOptions({ queryKey: ["archives", "hidden", srv], queryFn: () => getJson<ArchiveHiddenFactions>(srv, "/archives/hidden-factions"), staleTime: STALE, gcTime: GC });
}

export function archiveOperatorQueryOptions(id: string, server: string = DEFAULT_GAMEDATA_SERVER) {
    const srv = resolveGamedataServer(server);
    return queryOptions({ queryKey: ["archives", "operator", srv, id], queryFn: () => getArchiveOperator(id, srv), staleTime: STALE, gcTime: GC });
}
