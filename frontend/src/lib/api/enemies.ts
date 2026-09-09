import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { env } from "#/env";
import { backendFetch } from "#/lib/fetch";
import { values } from "#/lib/records";
import type { StageGroupKey } from "#/lib/registry/stage-groups";
// Generated from `backend/src/core/gamedata/types/{enemy,enemy_stages}.rs` and
// `backend/src/app/routes/enemies.rs`. To change a field, edit the Rust struct
// and run `bun run gen:types` - do not redeclare it here.
import type { AbilityInfo } from "#/types/generated/AbilityInfo";
import type { CommunityEnemyAverageResponse } from "#/types/generated/CommunityEnemyAverageResponse";
import type { DamageType } from "#/types/generated/DamageType";
import type { Enemy } from "#/types/generated/Enemy";
import type { EnemyAttributes } from "#/types/generated/EnemyAttributes";
import type { EnemyHandbook } from "#/types/generated/EnemyHandbook";
import type { EnemyInfoList } from "#/types/generated/EnemyInfoList";
import type { EnemyLevel } from "#/types/generated/EnemyLevel";
import type { EnemyLevelStats } from "#/types/generated/EnemyLevelStats";
import type { EnemySkill } from "#/types/generated/EnemySkill";
import type { EnemyStageRef } from "#/types/generated/EnemyStageRef";
import type { EnemyStats } from "#/types/generated/EnemyStats";
import type { RaceData } from "#/types/generated/RaceData";
import type { SkillBlackboardEntry } from "#/types/generated/SkillBlackboardEntry";
import type { StatRange } from "#/types/generated/StatRange";
import type { Refine } from "#/types/refine";

export type IEnemyLevel = EnemyLevel;
export type IEnemyDamageType = DamageType;
export type IStatRange = StatRange;
export type IEnemyInfoList = EnemyInfoList;
export type IEnemyRaceData = RaceData;
export type IEnemyAbilityInfo = AbilityInfo;
export type IEnemySkillBlackboardEntry = SkillBlackboardEntry;
export type IEnemyAttributes = EnemyAttributes;
export type IEnemySkill = EnemySkill;
export type IEnemyLevelStats = EnemyLevelStats;
export type IEnemyStats = EnemyStats;
export type IEnemy = Enemy;
export type IEnemyHandbook = EnemyHandbook;

/** Backend-served square icon for an enemy, keyed by `enemyId`. */
export function enemyIconURL(enemyId: string): string {
    return `${env.VITE_BACKEND_URL ?? ""}/api/enemy-icon/${encodeURIComponent(enemyId)}`;
}

export const getEnemiesFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/enemies");
    if (!res.ok) throw new Error(`Failed to load enemies: ${res.status}`);
    return (await res.json()) as IEnemyHandbook;
});

export function enemiesQueryOptions() {
    return queryOptions({
        queryKey: ["enemies"],
        queryFn: () => getEnemiesFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const getEnemiesListFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/enemies");
    if (!res.ok) throw new Error(`Failed to load enemies: ${res.status}`);
    const handbook = (await res.json()) as IEnemyHandbook;
    return values(handbook.enemyData).sort((a, b) => a.sortId - b.sortId);
});

export function enemiesListQueryOptions() {
    return queryOptions({
        queryKey: ["enemies", "list"],
        queryFn: () => getEnemiesListFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/**
 * A single appearance of an enemy in a stage.
 *
 * `category` and `group` are `String` on the Rust side - the group keys live in
 * the frontend registry, not in game data - so they are narrowed here.
 */
export type IEnemyStageRef = Refine<
    EnemyStageRef,
    {
        /** Coarse UI bucket: "stages" (main story), "events", or "modes". */
        category: "stages" | "events" | "modes";
        /** Fine group key (see `StageGroupKey`) for grouped UIs; resolved by the backend. */
        group: StageGroupKey;
    }
>;

/** `enemyId -> stages it appears in`. */
export type IEnemyStageIndex = Record<string, IEnemyStageRef[]>;

export const getEnemyStagesFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/enemy-stages");
    if (!res.ok) throw new Error(`Failed to load enemy stages: ${res.status}`);
    return (await res.json()) as IEnemyStageIndex;
});

/** Full enemy -> stages index. Fetch once and look up by enemy id client-side. */
export function enemyStagesQueryOptions() {
    return queryOptions({
        queryKey: ["enemies", "stages"],
        queryFn: () => getEnemyStagesFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export type IEnemyCommunityAverage = CommunityEnemyAverageResponse;

export const getEnemyCommunityAverageFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/encountered-enemies/community-average");
    if (!res.ok) throw new Error(`Failed to load enemy community average: ${res.status}`);
    return (await res.json()) as IEnemyCommunityAverage;
});

export function enemyCommunityAverageQueryOptions() {
    return queryOptions({
        queryKey: ["enemies", "community-average"],
        queryFn: () => getEnemyCommunityAverageFn(),
        staleTime: 30 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}

/**
 * A single enemy's handbook record plus the race lookup needed to resolve its
 * displayed race. Served by `GET /enemies/{id}`; replaces loading the entire
 * enemy handbook to render one enemy.
 */
export interface IEnemyDetail {
    enemy: IEnemy;
    raceData: Record<string, IEnemyRaceData>;
}

export const getEnemyDetailFn = createServerFn({ method: "GET" })
    .inputValidator((id: string) => id)
    .handler(async ({ data: id }) => {
        const res = await backendFetch(`/enemies/${encodeURIComponent(id)}`);
        if (res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load enemy ${id}: ${res.status}`);
        return (await res.json()) as IEnemyDetail;
    });

export function enemyDetailQueryOptions(id: string) {
    return queryOptions({
        queryKey: ["enemies", "detail", id],
        queryFn: () => getEnemyDetailFn({ data: id }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
        enabled: !!id,
    });
}

/** The "Appears In" list for a single enemy, served by `GET /enemies/{id}/stages`. */
export const getEnemyAppearsInFn = createServerFn({ method: "GET" })
    .inputValidator((id: string) => id)
    .handler(async ({ data: id }) => {
        const res = await backendFetch(`/enemies/${encodeURIComponent(id)}/stages`);
        if (res.status === 404) return [] as IEnemyStageRef[];
        if (!res.ok) throw new Error(`Failed to load enemy stages ${id}: ${res.status}`);
        return (await res.json()) as IEnemyStageRef[];
    });

export function enemyAppearsInQueryOptions(id: string) {
    return queryOptions({
        queryKey: ["enemies", "appears-in", id],
        queryFn: () => getEnemyAppearsInFn({ data: id }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
        enabled: !!id,
    });
}
