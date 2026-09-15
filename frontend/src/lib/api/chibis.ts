import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
// Generated from `backend/src/core/gamedata/types/chibi.rs`. To change a field,
// edit the Rust struct and run `bun run gen:types` - do not redeclare it here.
import type { ChibiCharacter } from "#/types/generated/ChibiCharacter";
import type { ChibiData } from "#/types/generated/ChibiData";
import type { ChibiSkin } from "#/types/generated/ChibiSkin";
import type { SpineFiles } from "#/types/generated/SpineFiles";
import { DEFAULT_GAMEDATA_SERVER, gamedataKey, gamedataPath, resolveGamedataServer } from "./gamedata";

export type IChibiSpineFiles = SpineFiles;

/** Whether a spine set is complete enough to load (skel + atlas + png all present). */
export function isCompleteSpineFiles(files: IChibiSpineFiles | undefined | null): files is IChibiSpineFiles {
    return !!files?.skel && !!files.atlas && !!files.png;
}

export type IChibiSkin = ChibiSkin;
export type IChibiCharacter = ChibiCharacter;
export type IChibiResponse = ChibiData;

/**
 * A single operator's chibi entry, served by `GET /chibis/{operatorId}` (where
 * `operatorId` is the catalog `operatorCode` = charId). Avoids downloading the
 * entire chibi catalog to render one operator. 404 -> `null`.
 */
export const getChibiByOperatorFn = createServerFn({ method: "GET" })
    .inputValidator((data: { operatorId: string; server: string }) => data)
    .handler(async ({ data: { operatorId, server } }) => {
        const res = await backendFetch(gamedataPath(server, `/chibis/${encodeURIComponent(operatorId)}`));
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load chibi ${operatorId}: ${res.status}`);
        return (await res.json()) as IChibiCharacter;
    });

export function chibiByOperatorQueryOptions(operatorId: string, server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["chibis", "operator", resolveGamedataServer(server), operatorId],
        queryFn: () => (operatorId ? getChibiByOperatorFn({ data: { operatorId, server: resolveGamedataServer(server) } }) : Promise.resolve(null)),
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
    .inputValidator((data: { server: string }) => data)
    .handler(async ({ data: { server } }) => {
        const res = await backendFetch(gamedataPath(server, "/static/chibis"));
        if (!res.ok) throw new Error(`Failed to load chibi catalog: ${res.status}`);
        return (await res.json()) as IChibiResponse;
    });

export function operatorChibisQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["chibis", "catalog", resolveGamedataServer(server)],
        queryFn: () => getOperatorChibisFn({ data: { server: resolveGamedataServer(server) } }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

// Enemy chibis reuse the same response shape; `operatorCode` carries the
// enemy id (e.g. "enemy_1000_gopro") and skins are alternate in-fight forms.
export const getEnemyChibisFn = createServerFn({ method: "GET" })
    .inputValidator((server: string | undefined) => server)
    .handler(async ({ data: server }) => {
        const res = await backendFetch(gamedataPath(server, "/static/enemy-chibis"));
        if (!res.ok) throw new Error(`Failed to load enemy chibis: ${res.status}`);
        return (await res.json()) as IChibiResponse;
    });

export function enemyChibisQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["enemy-chibis", ...gamedataKey(server)],
        queryFn: () => getEnemyChibisFn({ data: resolveGamedataServer(server) }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
