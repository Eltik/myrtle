import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { env } from "#/env";
import { backendFetch } from "#/lib/fetch";
import type { IActivity, IRetroAct, IStage, IZone, StageClearsMap, StageDifficulty } from "#/types/stages";
import { optionalSiteToken } from "./_shared.server";
import type { IEnemy } from "./enemies";
import type { ILevel } from "./level";
import type { IMaterialItem } from "./materials";

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
 * Build all candidate map-preview asset paths for a stage, relative to the
 * backend `/api/assets` root. Stage preview assets live under
 * `textures/arts/ui/stage_mappreview_h2_{family}{variant}_0/{stageId}.png` where
 * `variant` is one of "", "_a", "_b", "_1", "_2", … and is not derivable from
 * the stage data, so we emit a small product of candidates and let the consumer
 * fall through on 404.
 */
export function stagePreviewAssetPaths(stage: IStage): string[] {
    const ids = stagePreviewCandidates(stage);
    const families = new Set<string>();
    for (const id of ids) families.add(stageFamily(id));

    const seen = new Set<string>();
    const paths: string[] = [];

    for (const id of ids) {
        for (const family of families) {
            for (const variant of FOLDER_VARIANTS) {
                const folder = `stage_mappreview_h2_${family}${variant}_0`;
                const path = `/textures/arts/ui/${folder}/${encodeURIComponent(id)}.png`;
                if (!seen.has(path)) {
                    seen.add(path);
                    paths.push(path);
                }
            }
        }
    }

    return paths;
}

/** Absolute `/api/assets/...` candidate URLs for a stage's map preview. */
export function stagePreviewURLs(stage: IStage): string[] {
    const base = env.VITE_BACKEND_URL ?? "";
    return stagePreviewAssetPaths(stage).map((path) => `${base}/api/assets${path}`);
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

/**
 * One browsable stage in the Stage List, precomputed by the backend across all
 * game modes (story / events / SSS / Annihilation / IS / RA / CC / Paradox).
 */
export interface IStageIndexEntry {
    /** `stage_table` id, or the level file's relative id for procedural nodes. */
    stageId: string;
    levelId?: string | null;
    code: string;
    name?: string | null;
    zoneId: string;
    zoneName?: string | null;
    /** Sort key within a group - episode number for story, else zone index. */
    zoneOrder: number;
    /** Fine group: story / events / annihilation / is / ra / sss / paradox / cc / supplies / other. */
    group: string;
    category: string;
    apCost: number;
    boss: boolean;
    isHard: boolean;
    difficulty: StageDifficulty;
    /** Whether the stage detail / map viewer can render it (level file exists). */
    canView: boolean;
    /** Asset-relative path to the stage map preview (under `/api/assets/`), or null. */
    preview?: string | null;
    /** Asset-relative path to the zone/event banner key-art, or null. */
    banner?: string | null;
}

/** Absolute URL for a backend asset-relative path (e.g. a stage preview). */
export function assetURL(path: string): string {
    const base = env.VITE_BACKEND_URL ?? "";
    return `${base}/api/assets/${path.replace(/^\/+/, "")}`;
}

/**
 * A minimal {@link IStage}/{@link IZone} synthesized from a stage-index entry,
 * for procedural-mode stages (IS / RA / CC / Paradox) that have no `stage_table`
 * entry but are still viewable (their level file resolves via the mode map).
 */
export function syntheticStageFromIndex(e: IStageIndexEntry): { stage: IStage; zone: IZone } {
    return {
        stage: {
            stageId: e.stageId,
            zoneId: e.zoneId,
            code: e.code,
            name: e.name ?? undefined,
            levelId: e.levelId ?? undefined,
            stageType: "ACTIVITY",
            difficulty: e.difficulty,
            apCost: e.apCost,
            canPractice: false,
            canBattleReplay: false,
            canMultipleBattle: false,
            isStoryOnly: false,
            isPredefined: false,
            dangerPoint: 0,
            expGain: 0,
            goldGain: 0,
            unlockCondition: [],
            bossMark: e.boss,
        },
        zone: {
            zoneId: e.zoneId,
            zoneIndex: e.zoneOrder,
            type: "ACTIVITY",
            zoneNameSecond: e.zoneName ?? undefined,
            canPreview: false,
            hasAdditionalPanel: false,
        },
    };
}

export const getStageIndexFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/stage-index");
    if (!res.ok) throw new Error(`Failed to load stage index: ${res.status}`);
    return (await res.json()) as IStageIndexEntry[];
});

export function stageIndexQueryOptions() {
    return queryOptions({
        queryKey: ["static", "stage-index"],
        queryFn: () => getStageIndexFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/**
 * Everything the stage-detail page needs for a single stage, served by
 * `GET /stages/{stageId}/detail`: the stage record, its zone, its level data,
 * and only the enemies / materials referenced by that stage. Replaces the old
 * approach of loading the full stages/zones/enemies/materials tables. Procedural
 * IS/RA/CC nodes (no `stage_table` entry) return 404 -> the route falls back to
 * the stage index.
 */
export interface IStageDetail {
    stage: IStage;
    zone: IZone | null;
    levelData: ILevel | null;
    /** Only the enemies referenced by this stage's level, keyed by enemyId. */
    enemies: Record<string, IEnemy>;
    /** Only the non-CHAR drop items referenced by this stage, keyed by itemId. */
    materials: Record<string, IMaterialItem>;
}

export const getStageDetailFn = createServerFn({ method: "GET" })
    .inputValidator((stageId: string) => stageId)
    .handler(async ({ data: stageId }) => {
        const res = await backendFetch(`/stages/${encodeURIComponent(stageId)}/detail`);
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load stage detail ${stageId}: ${res.status}`);
        return (await res.json()) as IStageDetail;
    });

export function stageDetailQueryOptions(stageId: string) {
    return queryOptions({
        queryKey: ["stages", "detail", stageId],
        queryFn: () => getStageDetailFn({ data: stageId }),
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
        const token = bearerToken ?? optionalSiteToken();
        const res = await backendFetch(`/stage-clears?uid=${encodeURIComponent(uid)}`, { bearerToken: token });
        if (!res.ok) {
            // Absent (404) or private (403) accounts have no clears, not an error.
            if (res.status === 404 || res.status === 403) return {} as StageClearsMap;
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
