import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";

export interface IChibiSpineFiles {
    atlas: string | null;
    skel: string | null;
    png: string | null;
}

export type ChibiAnimationType = "front" | "back" | "dorm";

export interface IChibiSkin {
    name: string;
    path: string;
    hasSpineData: boolean;
    animationTypes: Record<string, IChibiSpineFiles>;
}

export interface IChibiCharacter {
    operatorCode: string;
    name: string;
    path: string;
    skins: IChibiSkin[];
}

export interface IChibiResponse {
    characters: IChibiCharacter[];
}

export const getChibisFn = createServerFn({ method: "GET" })
    .inputValidator((server: "en" | "cn") => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(server === "cn" ? "/cn/static/chibis" : "/static/chibis");
        if (!res.ok) throw new Error(`Failed to load chibis: ${res.status}`);
        return (await res.json()) as IChibiResponse;
    });

export function chibisQueryOptions(server: "en" | "cn" = "en") {
    return queryOptions({
        queryKey: ["chibis", server],
        queryFn: () => getChibisFn({ data: server }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
