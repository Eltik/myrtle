import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { IVoices } from "#/types/voices";

export const getVoicesFn = createServerFn({ method: "GET" })
    .inputValidator((server: "en" | "cn") => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(server === "cn" ? "/cn/static/voices" : "/static/voices");
        if (!res.ok) throw new Error(`Failed to load voices: ${res.status}`);
        return (await res.json()) as IVoices;
    });

export function voicesQueryOptions(server: "en" | "cn" = "en") {
    return queryOptions({
        queryKey: ["voices", server],
        queryFn: () => getVoicesFn({ data: server }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/// One operator's voice lines (KB) instead of the whole voices table.
export const getOperatorVoicesFn = createServerFn({ method: "GET" })
    .inputValidator((data: { id: string; server: "en" | "cn" }) => data)
    .handler(async ({ data: { id, server } }) => {
        const prefix = server === "cn" ? "/cn" : "";
        const res = await backendFetch(`${prefix}/voices/${encodeURIComponent(id)}`);
        if (!res.ok) throw new Error(`Failed to load operator voices: ${res.status}`);
        return (await res.json()) as IVoices;
    });

export function operatorVoicesQueryOptions(id: string, server: "en" | "cn" = "en") {
    return queryOptions({
        queryKey: ["voices", "operator", server, id],
        queryFn: () => getOperatorVoicesFn({ data: { id, server } }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
