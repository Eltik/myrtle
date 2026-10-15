import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
// Generated from `backend/src/database/models/squad.rs` and
// `backend/src/core/squad/types.rs`. To change a field, edit the Rust struct
// and run `bun run gen:types`.
import type { SavedSquad } from "#/types/generated/SavedSquad";
import type { SavedSquadInput } from "#/types/generated/SavedSquadInput";
import type { SquadAnalysis } from "#/types/generated/SquadAnalysis";
import type { SquadAnalyzeRequest } from "#/types/generated/SquadAnalyzeRequest";
import type { SquadRules } from "#/types/generated/SquadRules";
import type { SquadsResponse } from "#/types/generated/SquadsResponse";
import { parseError } from "./_shared";
import { optionalSiteToken, requireSiteToken } from "./_shared.server";
import { DEFAULT_GAMEDATA_SERVER, gamedataKey, resolveGamedataServer } from "./gamedata";

/** `?server=` for the reader's game-data server: saves are checked, and analyses read, against it. */
function serverQuery(server: string | undefined): string {
    return `server=${encodeURIComponent(resolveGamedataServer(server))}`;
}

/** The prefix every squad write invalidates. */
export const SQUADS_QUERY_KEY = ["user", "squads"] as const;

export const getSquadsFn = createServerFn({ method: "GET" }).handler(async () => {
    const token = optionalSiteToken();
    if (!token) return null;
    const res = await backendFetch("/squads", { bearerToken: token });
    if (res.status === 401) return null;
    if (!res.ok) throw await parseError(res);
    return (await res.json()) as SquadsResponse;
});

/** The signed-in player's four slots and in-game squads; `null` when signed out. */
export function squadsQueryOptions(uid: string | null) {
    return queryOptions({
        queryKey: [...SQUADS_QUERY_KEY, uid ?? "anon"],
        queryFn: () => getSquadsFn(),
        staleTime: 60 * 1000,
        gcTime: 10 * 60 * 1000,
        enabled: uid !== null,
    });
}

export const saveSquadFn = createServerFn({ method: "POST" })
    .inputValidator((data: { slot: number; input: SavedSquadInput; server?: string }) => data)
    .handler(async ({ data: { slot, input, server } }) => {
        const res = await backendFetch(`/squads/${slot}?${serverQuery(server)}`, {
            method: "PUT",
            bearerToken: requireSiteToken(),
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(input),
        });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as SavedSquad;
    });

export const deleteSquadFn = createServerFn({ method: "POST" })
    .inputValidator((data: { slot: number }) => data)
    .handler(async ({ data: { slot } }) => {
        const res = await backendFetch(`/squads/${slot}`, { method: "DELETE", bearerToken: requireSiteToken() });
        if (!res.ok && res.status !== 404) throw await parseError(res);
        return { ok: true as const };
    });

export const analyzeSquadFn = createServerFn({ method: "POST" })
    .inputValidator((data: { request: SquadAnalyzeRequest; server: string }) => data)
    .handler(async ({ data: { request, server } }) => {
        const res = await backendFetch(`/squads/analyze?${serverQuery(server)}`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(request),
        });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as SquadAnalysis;
    });

/**
 * One analysis per distinct request body and server. The key is the body
 * itself, so a squad edited back to a previous state is answered from cache,
 * and the page keeps the last answer while the next one loads
 * (`placeholderData`).
 */
export function squadAnalysisQueryOptions(request: SquadAnalyzeRequest | null, server: string = DEFAULT_GAMEDATA_SERVER) {
    const resolved = resolveGamedataServer(server);
    return queryOptions({
        queryKey: ["squads", "analyze", ...gamedataKey(resolved), request],
        queryFn: () => analyzeSquadFn({ data: { request: request as SquadAnalyzeRequest, server: resolved } }),
        staleTime: 5 * 60 * 1000,
        gcTime: 10 * 60 * 1000,
        enabled: request !== null,
        placeholderData: (previous) => previous,
    });
}

/**
 * Read by the browser straight from the backend, not through a server function,
 * whose relay sent 34,048 bytes of uncompressed seroval for 3,857 bytes of brotli.
 */
async function fetchSquadRules(server: string): Promise<SquadRules> {
    const res = await backendFetch(`/squads/rules?${serverQuery(server)}`);
    if (!res.ok) throw await parseError(res);
    return (await res.json()) as SquadRules;
}

/** The server's stage threat thresholds and challenge-to-normal stage map; they move only with a game data update. */
export function squadRulesQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["squads", "rules", ...gamedataKey(server)],
        queryFn: () => fetchSquadRules(resolveGamedataServer(server)),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
