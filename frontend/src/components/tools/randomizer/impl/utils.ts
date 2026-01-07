import type { Stage } from "~/types/api/impl/stage";
import type { User } from "~/types/api/impl/user";
import type { Zone } from "~/types/api/impl/zone";
import type { RandomizerOperator } from "../index";
import { getActivityIdFromZoneId, getPermanentEventInfo, getPermanentZonePrefix, isActivityCurrentlyOpen } from "./activity-names";
import type { Challenge } from "./types";

// Challenge pool based on common Arknights challenge run types
const CHALLENGES: Challenge[] = [
    // Restrictions
    { type: "restriction", title: "No Guards", description: "Cannot deploy any Guard operators" },
    { type: "restriction", title: "No Medics", description: "Cannot deploy any Medic operators" },
    { type: "restriction", title: "No Casters", description: "Cannot deploy any Caster operators" },
    { type: "restriction", title: "No Snipers", description: "Cannot deploy any Sniper operators" },
    { type: "restriction", title: "No Defenders", description: "Cannot deploy any Defender operators" },
    { type: "restriction", title: "Low Rarity Only", description: "Only 1★-3★ operators allowed" },
    { type: "restriction", title: "4★ Max", description: "No operators above 4★ rarity" },
    { type: "restriction", title: "No Ranged", description: "Only melee operators allowed" },
    { type: "restriction", title: "No Melee", description: "Only ranged operators allowed" },
    { type: "restriction", title: "Half Squad", description: "Deploy at most 6 operators" },

    // Modifiers
    { type: "modifier", title: "Speed Run", description: "Complete as fast as possible" },
    { type: "modifier", title: "Minimal Deployment", description: "Use the fewest operators possible" },
    { type: "modifier", title: "No Retreating", description: "Cannot retreat any operator once deployed" },
    {
        type: "modifier",
        title: "Deploy Order Challenge",
        description: "Must deploy operators in roster order from left to right",
    },
    { type: "modifier", title: "First Try Only", description: "Must complete on first attempt, no retry" },
    { type: "modifier", title: "Auto Deploy", description: "Must use auto-deploy after first clear" },

    // Objectives
    { type: "objective", title: "No Leaks", description: "Do not let any enemy through" },
    { type: "objective", title: "3-Star Clear", description: "Must achieve 3-star rating" },
    { type: "objective", title: "Trust Farm", description: "Fill unused slots with operators for trust farming" },
];

export function selectRandomStage(
    stages: Stage[],
    zones: Zone[],
    allowedZoneTypes: string[] = [],
    user?: User | null,
    onlyCompleted = false,
    selectedStages: string[] = [],
    onlyAvailableStages = false, // Added onlyAvailableStages parameter
): Stage | null {
    if (stages.length === 0) return null;

    let filteredStages = stages;

    if (selectedStages.length > 0) {
        filteredStages = filteredStages.filter((stage) => selectedStages.includes(stage.stageId));
    } else {
        if (allowedZoneTypes.length > 0) {
            filteredStages = filteredStages.filter((stage) => {
                const zone = zones.find((z) => z.zoneId === stage.zoneId);
                return zone && allowedZoneTypes.includes(zone.type);
            });
        }
    }

    if (onlyCompleted && user) {
        filteredStages = filteredStages.filter((stage) => {
            const stageData = user.dungeon.stages[stage.stageId];
            return stageData && stageData.completeTimes > 0;
        });
    }

    if (onlyAvailableStages) {
        filteredStages = filteredStages.filter((stage) => {
            const zone = zones.find((z) => z.zoneId === stage.zoneId);
            if (!zone) return false;

            // Main story stages are always available
            if (zone.type === "MAINLINE") return true;

            // Check if this is a permanent zone (e.g., permanent_sidestory_1_zone1)
            const permanentPrefix = getPermanentZonePrefix(stage.zoneId);
            if (permanentPrefix) return true;

            // For activity zones, check if event is open or has become permanent
            const activityId = getActivityIdFromZoneId(stage.zoneId);
            if (!activityId) return false;

            // Check if this activity is now a permanent side story/intermezzo
            const permanentInfo = getPermanentEventInfo(activityId);
            if (permanentInfo) return true;

            // Check if the event is currently running
            return isActivityCurrentlyOpen(activityId);
        });
    }

    if (filteredStages.length === 0) return null;

    const randomIndex = Math.floor(Math.random() * filteredStages.length);
    return filteredStages[randomIndex] ?? null;
}

export function generateRandomSquad(operators: RandomizerOperator[], squadSize: number, allowDuplicates: boolean): RandomizerOperator[] {
    if (operators.length === 0) return [];

    const squad: RandomizerOperator[] = [];
    const availablePool = [...operators];

    for (let i = 0; i < squadSize; i++) {
        if (availablePool.length === 0) {
            if (allowDuplicates && operators.length > 0) {
                // Reset pool if duplicates allowed
                availablePool.push(...operators);
            } else {
                break;
            }
        }

        const randomIndex = Math.floor(Math.random() * availablePool.length);
        const selectedOperator = availablePool[randomIndex];

        if (selectedOperator) {
            squad.push(selectedOperator);
            if (!allowDuplicates) {
                availablePool.splice(randomIndex, 1);
            }
        }
    }

    return squad;
}

export function generateChallenge(): Challenge {
    const randomIndex = Math.floor(Math.random() * CHALLENGES.length);
    return (
        CHALLENGES[randomIndex] ?? {
            type: "restriction",
            title: "Random Challenge",
            description: "Complete the stage with any restrictions you choose",
        }
    );
}

export function getRarityNumber(rarity: string): number {
    return Number.parseInt(rarity.replace("TIER_", ""), 10);
}
