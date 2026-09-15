import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { checkSample, firstValue, voiceContract } from "#/lib/api/contract";
import { backendFetch } from "#/lib/fetch";
import type { IVoices } from "#/types/voices";
import { DEFAULT_GAMEDATA_SERVER, gamedataPath, resolveGamedataServer } from "./gamedata";

export const getVoicesFn = createServerFn({ method: "GET" })
    .inputValidator((server: string) => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(gamedataPath(server, "/static/voices"));
        if (!res.ok) throw new Error(`Failed to load voices: ${res.status}`);
        const payload = (await res.json()) as IVoices;
        checkSample("static/voices", voiceContract, firstValue(payload.charWords));
        return payload;
    });

export function voicesQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["voices", resolveGamedataServer(server)],
        queryFn: () => getVoicesFn({ data: resolveGamedataServer(server) }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/// One operator's voice lines (KB) instead of the whole voices table.
export const getOperatorVoicesFn = createServerFn({ method: "GET" })
    .inputValidator((data: { id: string; server: string }) => data)
    .handler(async ({ data: { id, server } }) => {
        const res = await backendFetch(gamedataPath(server, `/voices/${encodeURIComponent(id)}`));
        if (!res.ok) throw new Error(`Failed to load operator voices: ${res.status}`);
        const payload = (await res.json()) as IVoices;
        checkSample("voices/:id", voiceContract, firstValue(payload.charWords));
        return payload;
    });

export function operatorVoicesQueryOptions(id: string, server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["voices", "operator", resolveGamedataServer(server), id],
        queryFn: () => getOperatorVoicesFn({ data: { id, server: resolveGamedataServer(server) } }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
