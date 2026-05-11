import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { env } from "#/env";
import { backendFetch } from "#/lib/fetch";
import type { IActivity, IRetroAct, IStage, IZone, StageClearsMap } from "#/types/stages";

const PREVIEW_BASE = "/api/assets/textures/arts/ui";
const PREVIEW_SUFFIX_RE = /#[a-z]#?$/i;

/** Folder-variant suffixes that appear between the stage family and the trailing `_0`. */
const FOLDER_VARIANTS = ["", "_a", "_b", "_c", "_d", "_1", "_2", "_3"] as const;

/**
 * Stage IDs that might point at the same map preview. Stages reuse the same
 * map across Normal / CM / Hard / Challenge variants; this walks common aliases
 * so the consumer can try them in order until one loads.
 */
export function stagePreviewCandidates(stage: IStage): string[] {
    const out: string[] = [];
    const push = (id: string | null | undefined) => {
        if (!id) return;
        if (out.includes(id)) return;
        out.push(id);
    };

    const base = stage.stageId.replace(PREVIEW_SUFFIX_RE, "");
    // CM stages (`main_xx-yy#f#`) reuse the non-CM map preview, so prefer the
    // stripped base id and fall back to the suffixed form only as a last resort.
    if (stage.difficulty === "FOUR_STAR") push(base);
    push(stage.stageId);
    if (base !== stage.stageId) push(base);

    for (const [from, to] of [
        ["easy_", "main_"],
        ["tough_", "main_"],
        ["hard_", "main_"],
    ] as const) {
        if (base.startsWith(from)) push(`${to}${base.slice(from.length)}`);
    }

    push(stage.mainStageId);
    push(stage.hardStagedId);

    return out;
}

/**
 * Strips the per-stage sub-index from a stageId, leaving the zone/episode family.
 * `main_14-06` → `main_14`, `act29side_ex01` → `act29side`, `a001_01` → `a001`.
 */
function stageFamily(stageId: string): string {
    const dashIdx = stageId.indexOf("-");
    if (dashIdx > 0) return stageId.slice(0, dashIdx);

    const parts = stageId.split("_");
    // Drop trailing segments that look like a stage sub-index ("01", "ex02", "s03").
    while (parts.length > 1 && /^[a-z]{0,3}\d+$/i.test(parts[parts.length - 1] ?? "")) {
        parts.pop();
    }
    return parts.join("_") || stageId;
}

/**
 * Build all candidate `/api/assets/...` URLs for a stage's map preview.
 * Stage preview assets live under
 * `textures/arts/ui/stage_mappreview_h2_{family}{variant}_0/{stageId}.png` where
 * `variant` is one of "", "_a", "_b", "_1", "_2", … and is not derivable from
 * the stage data, so we emit a small product of candidates and let the consumer
 * fall through on 404.
 */
export function stagePreviewURLs(stage: IStage): string[] {
    const ids = stagePreviewCandidates(stage);
    const families = new Set<string>();
    for (const id of ids) families.add(stageFamily(id));

    const base = `${env.VITE_BACKEND_URL ?? ""}${PREVIEW_BASE}`;
    const seen = new Set<string>();
    const urls: string[] = [];

    for (const id of ids) {
        for (const family of families) {
            for (const variant of FOLDER_VARIANTS) {
                const folder = `stage_mappreview_h2_${family}${variant}_0`;
                const url = `${base}/${folder}/${encodeURIComponent(id)}.png`;
                if (!seen.has(url)) {
                    seen.add(url);
                    urls.push(url);
                }
            }
        }
    }

    return urls;
}

export const getStagesFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/stages");
    if (!res.ok) throw new Error(`Failed to load stages: ${res.status}`);
    const raw = (await res.json()) as Record<string, IStage>;
    return Object.values(raw);
});

export function stagesQueryOptions() {
    return queryOptions({
        queryKey: ["static", "stages"],
        queryFn: () => getStagesFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const getZonesFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/zones");
    if (!res.ok) throw new Error(`Failed to load zones: ${res.status}`);
    const raw = (await res.json()) as Record<string, IZone>;
    return Object.values(raw);
});

export function zonesQueryOptions() {
    return queryOptions({
        queryKey: ["static", "zones"],
        queryFn: () => getZonesFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const getActivitiesFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/activities");
    if (!res.ok) throw new Error(`Failed to load activities: ${res.status}`);
    const raw = (await res.json()) as Record<string, IActivity>;
    return Object.values(raw);
});

export function activitiesQueryOptions() {
    return queryOptions({
        queryKey: ["static", "activities"],
        queryFn: () => getActivitiesFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const getRetroActsFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/retro_acts");
    if (!res.ok) throw new Error(`Failed to load retro acts: ${res.status}`);
    const raw = (await res.json()) as Record<string, IRetroAct>;
    return Object.values(raw);
});

export function retroActsQueryOptions() {
    return queryOptions({
        queryKey: ["static", "retro_acts"],
        queryFn: () => getRetroActsFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const getUserStageClearsFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string; bearerToken?: string }) => data)
    .handler(async ({ data: { uid, bearerToken } }) => {
        const res = await backendFetch(`/stage-clears?uid=${encodeURIComponent(uid)}`, { bearerToken });
        if (!res.ok) {
            if (res.status === 404) return {} as StageClearsMap;
            if (res.status === 403) return {} as StageClearsMap;
            throw new Error(`Failed to load stage clears: ${res.status}`);
        }
        return (await res.json()) as StageClearsMap;
    });

export function userStageClearsQueryOptions(uid: string | null, bearerToken?: string) {
    return queryOptions({
        queryKey: ["user", "stage-clears", uid, bearerToken ? "auth" : "anon"],
        queryFn: () => (uid ? getUserStageClearsFn({ data: { uid, bearerToken } }) : Promise.resolve({} as StageClearsMap)),
        enabled: !!uid,
        staleTime: 5 * 60 * 1000,
        gcTime: 30 * 60 * 1000,
    });
}
