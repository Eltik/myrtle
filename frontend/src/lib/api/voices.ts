import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { IVoices } from "#/types/voices";

export const getVoicesFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/voices");
    if (!res.ok) throw new Error(`Failed to load voices: ${res.status}`);
    return (await res.json()) as IVoices;
});

export function voicesQueryOptions() {
    return queryOptions({
        queryKey: ["voices"],
        queryFn: () => getVoicesFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
