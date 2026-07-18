import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";

export interface IChibiSpineFiles {
    atlas: string | null;
    skel: string | null;
    png: string | null;
}

/** Whether a spine set is complete enough to load (skel + atlas + png all present). */
export function isCompleteSpineFiles(files: IChibiSpineFiles | undefined | null): files is IChibiSpineFiles {
    return !!files?.skel && !!files.atlas && !!files.png;
}

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

/**
 * A single operator's chibi entry, served by `GET /chibis/{operatorId}` (where
 * `operatorId` is the catalog `operatorCode` = charId). Avoids downloading the
 * entire chibi catalog to render one operator. 404 -> `null`.
 */
export const getChibiByOperatorFn = createServerFn({ method: "GET" })
    .inputValidator((data: { operatorId: string; server: "en" | "cn" }) => data)
    .handler(async ({ data: { operatorId, server } }) => {
        const path = server === "cn" ? `/cn/chibis/${encodeURIComponent(operatorId)}` : `/chibis/${encodeURIComponent(operatorId)}`;
        const res = await backendFetch(path);
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load chibi ${operatorId}: ${res.status}`);
        return (await res.json()) as IChibiCharacter;
    });

export function chibiByOperatorQueryOptions(operatorId: string, server: "en" | "cn" = "en") {
    return queryOptions({
        queryKey: ["chibis", "operator", server, operatorId],
        queryFn: () => (operatorId ? getChibiByOperatorFn({ data: { operatorId, server } }) : Promise.resolve(null)),
        enabled: !!operatorId,
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/**
 * The full operator chibi catalog (`GET /static/chibis`). One request covers
 * every operator's skin -> animationTypes map, so a page that needs dynamic-art
 * lookups for many operators at once (e.g. the roster) fetches this once instead
 * of hitting `/chibis/{id}` per operator.
 */
export const getOperatorChibisFn = createServerFn({ method: "GET" })
    .inputValidator((data: { server: "en" | "cn" }) => data)
    .handler(async ({ data: { server } }) => {
        const path = server === "cn" ? "/cn/static/chibis" : "/static/chibis";
        const res = await backendFetch(path);
        if (!res.ok) throw new Error(`Failed to load chibi catalog: ${res.status}`);
        return (await res.json()) as IChibiResponse;
    });

export function operatorChibisQueryOptions(server: "en" | "cn" = "en") {
    return queryOptions({
        queryKey: ["chibis", "catalog", server],
        queryFn: () => getOperatorChibisFn({ data: { server } }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

// Enemy chibis reuse the same response shape; `operatorCode` carries the
// enemy id (e.g. "enemy_1000_gopro") and skins are alternate in-fight forms.
export const getEnemyChibisFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/enemy-chibis");
    if (!res.ok) throw new Error(`Failed to load enemy chibis: ${res.status}`);
    return (await res.json()) as IChibiResponse;
});

export function enemyChibisQueryOptions() {
    return queryOptions({
        queryKey: ["enemy-chibis"],
        queryFn: () => getEnemyChibisFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
