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

export function selectAvailableStages(stages: IStage[], zones: IZone[], settings: IRandomizerSettings, stageClears: StageClearsMap | null | undefined, lookup: ActivityLookup): IStage[] {
    const zoneById = new Map(zones.map((z) => [z.zoneId, z]));
    const now = Math.floor(Date.now() / 1000);

    return stages.filter((stage) => {
        const zone = zoneById.get(stage.zoneId);
        if (!zone) return false;
        if (!settings.allowedZoneTypes.includes(zone.type)) return false;

        if (settings.onlyCompletedStages) {
            const clear = stageClears?.[stage.stageId];
            if (!clear || (clear.completeTimes ?? 0) <= 0) return false;
        }

        if (settings.onlyAvailableStages) {
            if (zone.type === "MAINLINE") return true;
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
