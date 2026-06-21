import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { env } from "#/env";
import { backendFetch } from "#/lib/fetch";
import type { StageGroupKey } from "#/lib/registry/stage-groups";

export type IEnemyLevel = "NORMAL" | "ELITE" | "BOSS";

/** Backend-served square icon for an enemy, keyed by `enemyId`. */
export function enemyIconURL(enemyId: string): string {
    return `${env.VITE_BACKEND_URL ?? ""}/api/enemy-icon/${encodeURIComponent(enemyId)}`;
}

export type IEnemyDamageType = "PHYSIC" | "MAGIC" | "NO_DAMAGE" | "HEAL";

export type IEnemyJsonValue = string | number | boolean | null | IEnemyJsonValue[] | { [key: string]: IEnemyJsonValue };

export interface IStatRange {
    min: number;
    max: number;
}

export interface IEnemyInfoList {
    classLevel: string;
    attack: IStatRange;
    def: IStatRange;
    magicRes: IStatRange;
    maxHp: IStatRange;
    moveSpeed: IStatRange;
    attackSpeed: IStatRange;
    enemyDamageRes: IStatRange;
    enemyRes: IStatRange;
}

export interface IEnemyRaceData {
    id: string;
    raceName: string;
    sortId: number;
}

export interface IEnemyAbilityInfo {
    text: string;
    textFormat: string;
}

export interface IEnemySkillBlackboardEntry {
    key: string;
    value: number;
    valueStr: string | null;
}

export interface IEnemyAttributes {
    maxHp: number;
    atk: number;
    def: number;
    magicResistance: number;
    moveSpeed: number;
    attackSpeed: number;
    baseAttackTime: number;
    massLevel: number;
    hpRecoveryPerSec: number;
    stunImmune: boolean;
    silenceImmune: boolean;
    sleepImmune: boolean;
    frozenImmune: boolean;
    levitateImmune: boolean;
}

export interface IEnemySkill {
    prefabKey: string;
    priority: number;
    cooldown: number;
    initCooldown: number;
    spCost: number;
    blackboard: IEnemySkillBlackboardEntry[];
}

export interface IEnemyLevelStats {
    level: number;
    attributes: IEnemyAttributes;
    applyWay: string | null;
    motion: string | null;
    rangeRadius: number | null;
    lifePointReduce: number;
    skills: IEnemySkill[];
}

export interface IEnemyStats {
    levels: IEnemyLevelStats[];
}

export interface IEnemy {
    enemyId: string;
    enemyIndex: string;
    enemyTags: string[] | null;
    sortId: number;
    name: string;
    enemyLevel: IEnemyLevel;
    description: string;
    attackType: string | null;
    ability: string | null;
    isInvalidKilled: boolean;
    overrideKillCntInfos: { [key: string]: IEnemyJsonValue } | null;
    hideInHandbook: boolean;
    hideInStage: boolean;
    abilityList: IEnemyAbilityInfo[];
    linkEnemies: string[];
    damageType: IEnemyDamageType[];
    invisibleDetail: boolean;
    stats: IEnemyStats | null;
    portrait: string | null;
}

export interface IEnemyHandbook {
    levelInfoList: IEnemyInfoList[];
    enemyData: Record<string, IEnemy>;
    raceData: Record<string, IEnemyRaceData>;
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
    return Object.values(handbook.enemyData).sort((a, b) => a.sortId - b.sortId);
});

export function enemiesListQueryOptions() {
    return queryOptions({
        queryKey: ["enemies", "list"],
        queryFn: () => getEnemiesListFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/** A single appearance of an enemy in a stage. */
export interface IEnemyStageRef {
    stageId: string;
    /** Human-readable code, e.g. "0-1", "WD-8". */
    code: string;
    zoneId: string;
    /** Resolved zone/event display name; falls back to zoneId when null. */
    zoneName: string | null;
    /** Coarse UI bucket: "stages" (main story), "events", or "modes". */
    category: "stages" | "events" | "modes";
    /** Fine group key (see `StageGroupKey`) for grouped UIs; resolved by the backend. */
    group: StageGroupKey;
    stageName: string | null;
    /** True for hard-mode / Adverse variants. */
    isHard: boolean;
    /** Total spawned across all waves; 0 = declared but summon-only/conditional. */
    count: number;
}

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

export interface IEnemyCommunityAverage {
    /** Mean number of distinct enemies encountered across every synced user. */
    averageEncountered: number;
    /** Number of users in the average's denominator. */
    userCount: number;
    /** Visible handbook size, matching the per-user response's denominator. */
    handbookTotal: number;
    computedAt: string;
}

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

export const getEnemyFn = createServerFn({ method: "GET" })
    .inputValidator((id: string) => id)
    .handler(async ({ data: id }) => {
        const res = await backendFetch("/static/enemies");
        if (!res.ok) throw new Error(`Failed to load enemies: ${res.status}`);
        const handbook = (await res.json()) as IEnemyHandbook;
        return handbook.enemyData[id] ?? null;
    });

export function enemyQueryOptions(id: string) {
    return queryOptions({
        queryKey: ["enemies", "detail", id],
        queryFn: () => getEnemyFn({ data: id }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
        enabled: !!id,
    });
}
