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
