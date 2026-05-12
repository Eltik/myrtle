import type { IRosterEntry } from "#/lib/api/user";
import { rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import type { IStage, IZone, StageClearsMap } from "#/types/stages";
import { type ActivityLookup, getActivityIdFromZoneId, getPermanentEventInfo, getPermanentZonePrefix, isActivityCurrentlyOpen } from "./activity-lookup";
import { UNPLAYABLE_OPERATOR_IDS } from "./constants";
import type { IChallenge, IRandomizerOperator, IRandomizerSettings, IRosterIndex } from "./types";

const HEART_OF_SURGING_FLAME_NAME = "Heart of Surging Flame";

function isHeartOfSurgingFlameStage(zoneId: string, lookup: ActivityLookup): boolean {
    const permanentPrefix = getPermanentZonePrefix(zoneId);
    if (permanentPrefix) {
        const retro = lookup.retroByZonePrefix.get(permanentPrefix);
        if (retro?.name === HEART_OF_SURGING_FLAME_NAME) return true;
    }

    const activityId = getActivityIdFromZoneId(zoneId);
    if (!activityId) return false;

    const activityName = lookup.activityById.get(activityId)?.name;
    if (activityName === HEART_OF_SURGING_FLAME_NAME || activityName === `${HEART_OF_SURGING_FLAME_NAME} - Rerun`) return true;

    const linkedRetro = lookup.retroByActivityId.get(activityId);
    if (linkedRetro?.name === HEART_OF_SURGING_FLAME_NAME) return true;

    return false;
}

function shouldIncludeBySanityCost(stage: IStage, lookup: ActivityLookup): boolean {
    if (stage.apCost > 0) return true;

    if (isHeartOfSurgingFlameStage(stage.zoneId, lookup)) {
        const code = stage.code.toUpperCase();
        if (code.includes("ST") || code.includes("TR")) return false;
        return true;
    }

    return false;
}

/** Drops 0-AP tutorial/story-only stages (except Heart of Surging Flame playables). */
export function filterPlayableStages(stages: IStage[], lookup: ActivityLookup): IStage[] {
    return stages.filter((stage) => shouldIncludeBySanityCost(stage, lookup));
}

export function buildRosterIndex(roster: IRosterEntry[] | null | undefined): IRosterIndex {
    const owned = new Set<string>();
    const e2 = new Set<string>();
    if (!roster) return { owned, e2 };
    for (const entry of roster) {
        if (!entry.operator_id) continue;
        owned.add(entry.operator_id);
        if (entry.elite >= 2) e2.add(entry.operator_id);
    }
    return { owned, e2 };
}

export function toRandomizerOperator(op: IOperatorListItem): IRandomizerOperator | null {
    if (!op.id) return null;
    return {
        id: op.id,
        name: op.name,
        rarity: rarityToNumber(op.rarity),
        profession: op.profession,
        subProfessionId: op.subProfessionId,
        position: op.position,
    };
}

export function selectAvailableOperators(operators: IRandomizerOperator[], settings: IRandomizerSettings, rosterIndex: IRosterIndex): IRandomizerOperator[] {
    return operators.filter((op) => {
        if (!settings.allowedRarities.includes(op.rarity)) return false;
        if (!settings.allowedClasses.includes(op.profession)) return false;
        if (settings.hideUnplayableOperators && UNPLAYABLE_OPERATOR_IDS.has(op.id)) return false;
        if (settings.onlyOwnedOperators && !rosterIndex.owned.has(op.id)) return false;
        if (settings.onlyE2Operators && !rosterIndex.e2.has(op.id)) return false;
        return true;
    });
}

/** Zone types whose stages are always available (mainline, retro, daily farming, annihilation). */
const PERMANENT_ZONE_TYPES = new Set(["MAINLINE", "MAINLINE_ACTIVITY", "MAINLINE_RETRO", "WEEKLY", "CAMPAIGN"]);

export function selectAvailableStages(stages: IStage[], zones: IZone[], settings: IRandomizerSettings, stageClears: StageClearsMap | null | undefined, lookup: ActivityLookup): IStage[] {
    const zoneById = new Map(zones.map((z) => [z.zoneId, z]));
    const deselectedStages = new Set(settings.deselectedStageIds);
    const now = Math.floor(Date.now() / 1000);

    return stages.filter((stage) => {
        const zone = zoneById.get(stage.zoneId);
        if (!zone) return false;
        if (!settings.allowedZoneTypes.includes(zone.type)) return false;
        if (deselectedStages.has(stage.stageId)) return false;

        if (settings.onlyCompletedStages) {
            if (!isStageCleared(stage.stageId, stageClears)) return false;
        }

        if (settings.onlyAvailableStages) {
            if (PERMANENT_ZONE_TYPES.has(zone.type)) return true;
            if (getPermanentZonePrefix(stage.zoneId)) return true;
            const activityId = getActivityIdFromZoneId(stage.zoneId);
            if (!activityId) return false;
            if (getPermanentEventInfo(activityId, lookup)) return true;
            return isActivityCurrentlyOpen(activityId, lookup, now);
        }

        return true;
    });
}

export function pickRandomStage(stages: IStage[]): IStage | null {
    if (stages.length === 0) return null;
    return stages[Math.floor(Math.random() * stages.length)] ?? null;
}

export function pickRandomSquad(operators: IRandomizerOperator[], squadSize: number, allowDuplicates: boolean): IRandomizerOperator[] {
    if (operators.length === 0) return [];

    const out: IRandomizerOperator[] = [];
    const pool = [...operators];

    for (let i = 0; i < squadSize; i++) {
        if (pool.length === 0) {
            if (allowDuplicates) {
                pool.push(...operators);
            } else {
                break;
            }
        }
        const idx = Math.floor(Math.random() * pool.length);
        const chosen = pool[idx];
        if (!chosen) break;
        out.push(chosen);
        if (!allowDuplicates) pool.splice(idx, 1);
    }

    return out;
}

const CHALLENGES: IChallenge[] = [
    { kind: "restriction", title: "No Guards", description: "Cannot deploy any Guard operators." },
    { kind: "restriction", title: "No Medics", description: "Cannot deploy any Medic operators." },
    { kind: "restriction", title: "No Casters", description: "Cannot deploy any Caster operators." },
    { kind: "restriction", title: "No Snipers", description: "Cannot deploy any Sniper operators." },
    { kind: "restriction", title: "No Defenders", description: "Cannot deploy any Defender operators." },
    { kind: "restriction", title: "Low rarity only", description: "Only 1★–3★ operators allowed." },
    { kind: "restriction", title: "Four-star ceiling", description: "No operators above 4★ rarity." },
    { kind: "restriction", title: "Ranged only", description: "Only ranged operators allowed." },
    { kind: "restriction", title: "Melee only", description: "Only melee operators allowed." },
    { kind: "restriction", title: "Half squad", description: "Deploy at most 6 operators total." },

    { kind: "modifier", title: "Speed run", description: "Complete the stage as quickly as you can." },
    { kind: "modifier", title: "Minimal deployment", description: "Use the fewest operators possible." },
    { kind: "modifier", title: "No retreating", description: "Once deployed, no operator may be retreated." },
    { kind: "modifier", title: "Roster order", description: "Deploy operators left-to-right in the assigned order." },
    { kind: "modifier", title: "First attempt", description: "Must clear on the first try — no retries." },
    { kind: "modifier", title: "Auto only", description: "Use auto-deploy after the initial clear." },

    { kind: "objective", title: "No leaks", description: "Do not let any enemy through." },
    { kind: "objective", title: "Three-star clear", description: "Must achieve a 3-star rating." },
    { kind: "objective", title: "Trust farm", description: "Fill remaining slots with operators for trust." },
];

export function pickRandomChallenge(): IChallenge {
    return (
        CHALLENGES[Math.floor(Math.random() * CHALLENGES.length)] ?? {
            kind: "restriction" as const,
            title: "Random Challenge",
            description: "Complete the stage with any restriction you choose.",
        }
    );
}

export function getZoneDisplayName(zone: IZone | undefined, stageZoneId: string): string {
    if (!zone) return stageZoneId;
    return zone.zoneNameSecond ?? zone.zoneNameFirst ?? stageZoneId;
}

/**
 * A stage is cleared when its `state` is 3 (passed). Note that for adverse/tough
 * variants, clearing the harder version auto-passes the Normal/Easy variant with
 * `state: 3` and `completeTimes: 0`, so checking `completeTimes` alone would miss them.
 */
export function isStageCleared(stageId: string, stageClears: StageClearsMap | null | undefined): boolean {
    const clear = stageClears?.[stageId];
    return !!clear && (clear.state ?? 0) >= 3;
}

export type StageGroupSection = "MAIN" | "EVENT" | "OTHER";

export interface IStageGroup {
    id: string;
    label: string;
    sublabel?: string;
    section: StageGroupSection;
    /** Underlying zone type bucket (MAINLINE vs ACTIVITY) for cross-referencing source filters. */
    zoneType: string;
    /** Whether the event is currently open / always-available (permanent or mainline). */
    isOpen: boolean;
    sortKey: number;
    stages: IStage[];
}

function natCompareCode(a: string, b: string): number {
    return a.localeCompare(b, undefined, { numeric: true, sensitivity: "base" });
}

/** Activity ID suffixes that mark non-mainstream content (mini events, sandbox, fun, etc.). */
const OTHER_ACTIVITY_SUFFIXES = ["mini", "fun", "festival", "melee", "rune", "sandbox", "april", "vsint", "trial", "training"];

function isOtherActivity(activityId: string): boolean {
    const m = activityId.match(/^act\d+([a-z]+)$/i);
    if (!m) return false;
    const suffix = (m[1] ?? "").toLowerCase();
    return OTHER_ACTIVITY_SUFFIXES.includes(suffix);
}

/**
 * Extract the activity id from a stageId prefix. Some legacy event stages
 * (e.g. `act7d5_04`, `act4d0_05`) are attached to a `main_X` zone in the
 * game data despite being event content; the stageId itself is the reliable
 * source of truth for which activity they actually belong to.
 */
function getActivityIdFromStageId(stageId: string): string | null {
    const underscore = stageId.indexOf("_");
    if (underscore <= 0) return null;
    const prefix = stageId.slice(0, underscore);
    return /^act\d/i.test(prefix) ? prefix : null;
}

/**
 * Bucket the supplied stages into user-facing groups partitioned into three sections:
 * Main Story (one per mainline episode), Events (full sidestories / permanent SS/BL),
 * and Other (mini events, sandbox, festivals, etc.).
 */
export function buildStageGroups(stages: IStage[], zones: IZone[], lookup: ActivityLookup): IStageGroup[] {
    const zoneById = new Map(zones.map((z) => [z.zoneId, z]));
    const now = Math.floor(Date.now() / 1000);

    const groups = new Map<string, IStageGroup>();

    for (const stage of stages) {
        const zone = zoneById.get(stage.zoneId);
        if (!zone) continue;

        let groupId: string;
        let label: string;
        let sublabel: string | undefined;
        let section: StageGroupSection;
        let isOpen: boolean;
        let sortKey: number;

        const stageActivityId = getActivityIdFromStageId(stage.stageId);
        const stageActivity = stageActivityId ? lookup.activityById.get(stageActivityId) : undefined;
        const stageRetro = stageActivityId ? lookup.retroByActivityId.get(stageActivityId) : undefined;
        const stageBelongsToActivity = !!(stageActivity || stageRetro);

        const isMainlineZoneType = zone.type === "MAINLINE" || zone.type === "MAINLINE_ACTIVITY" || zone.type === "MAINLINE_RETRO";
        // Stage IDs for genuine mainline content are `main_/tough_/easy_/hard_/st_` prefixed;
        // legacy event stages (e.g. `act7d5_04`) sometimes share a `main_X` zone but should
        // NOT be considered main story.
        const stageIdPrefix = stage.stageId.split("_", 1)[0] ?? "";
        const isMainlineStageId = /^(main|tough|easy|hard|st)$/i.test(stageIdPrefix);
        const isTrueMainStory = isMainlineZoneType && isMainlineStageId;

        if (isTrueMainStory) {
            const chapterNumber = zone.zoneNameTitleCurrent?.replace(/^0+/, "") || String(zone.zoneIndex ?? 0);
            const chapterName = zone.zoneNameSecond ?? zone.zoneNameFirst ?? zone.zoneId;
            // Group by chapter number so MAINLINE / MAINLINE_ACTIVITY / MAINLINE_RETRO
            // variants of the same chapter merge into one entry.
            groupId = `mainline:${chapterNumber || zone.zoneId}`;
            label = `Chapter ${chapterNumber} - ${chapterName}`;
            sublabel = zone.zoneNameFirst && zone.zoneNameFirst !== chapterName ? zone.zoneNameFirst : undefined;
            section = "MAIN";
            isOpen = true;
            sortKey = Number.parseInt(zone.zoneNameTitleCurrent ?? "", 10);
            if (!Number.isFinite(sortKey)) sortKey = zone.zoneIndex ?? 0;
        } else if (stageBelongsToActivity && stageActivityId) {
            const activityId = stageActivityId;
            const activity = stageActivity;
            const retro = stageRetro;
            groupId = `activity:${activityId}`;
            label = activity?.name ?? retro?.name ?? activityId;

            const isPermanentSideOrBranch = !!retro && (retro.type === "SIDESTORY" || retro.type === "BRANCHLINE");
            if (isPermanentSideOrBranch) {
                sublabel = "Permanent";
                isOpen = true;
            } else if (activity) {
                isOpen = activity.startTime <= now && now <= activity.endTime;
                sublabel = activity.isReplicate ? "Rerun" : undefined;
            } else {
                isOpen = false;
            }

            section = isOtherActivity(activityId) && !isPermanentSideOrBranch ? "OTHER" : "EVENT";
            sortKey = activity?.startTime ?? retro?.startTime ?? 0;
        } else {
            const permanentPrefix = getPermanentZonePrefix(stage.zoneId);
            const zoneActivityId = getActivityIdFromZoneId(stage.zoneId);

            if (permanentPrefix) {
                const retro = lookup.retroByZonePrefix.get(permanentPrefix);
                groupId = `permanent:${permanentPrefix}`;
                label = retro?.name ?? getZoneDisplayName(zone, zone.zoneId);
                sublabel = "Permanent";
                section = "EVENT";
                isOpen = true;
                sortKey = retro?.startTime ?? 0;
            } else if (zoneActivityId) {
                const activity = lookup.activityById.get(zoneActivityId);
                const retro = lookup.retroByActivityId.get(zoneActivityId);
                groupId = `activity:${zoneActivityId}`;
                label = activity?.name ?? retro?.name ?? getZoneDisplayName(zone, zone.zoneId);

                const isPermanentSideOrBranch = !!retro && (retro.type === "SIDESTORY" || retro.type === "BRANCHLINE");
                if (isPermanentSideOrBranch) {
                    sublabel = "Permanent";
                    isOpen = true;
                } else if (activity) {
                    isOpen = activity.startTime <= now && now <= activity.endTime;
                    sublabel = activity.isReplicate ? "Rerun" : undefined;
                } else {
                    isOpen = false;
                }

                section = isOtherActivity(zoneActivityId) && !isPermanentSideOrBranch ? "OTHER" : "EVENT";
                sortKey = activity?.startTime ?? retro?.startTime ?? 0;
            } else {
                groupId = `zone:${stage.zoneId}`;
                label = getZoneDisplayName(zone, zone.zoneId);
                section = "OTHER";
                isOpen = PERMANENT_ZONE_TYPES.has(zone.type);
                sortKey = zone.zoneIndex ?? 0;
            }
        }

        let group = groups.get(groupId);
        if (!group) {
            group = { id: groupId, label, sublabel, section, zoneType: zone.type, isOpen, sortKey, stages: [] };
            groups.set(groupId, group);
        }
        group.stages.push(stage);
    }

    const out = Array.from(groups.values());

    for (const g of out) {
        g.stages.sort((a, b) => natCompareCode(a.code, b.code) || a.stageId.localeCompare(b.stageId));
    }

    const SECTION_ORDER: Record<StageGroupSection, number> = { MAIN: 0, EVENT: 1, OTHER: 2 };
    out.sort((a, b) => {
        const sec = SECTION_ORDER[a.section] - SECTION_ORDER[b.section];
        if (sec !== 0) return sec;
        if (a.section === "MAIN") return a.sortKey - b.sortKey;
        return b.sortKey - a.sortKey;
    });

    return out;
}

export const STAGE_SECTION_LABEL: Record<StageGroupSection, string> = {
    MAIN: "Main Story",
    EVENT: "Events",
    OTHER: "Other",
};
