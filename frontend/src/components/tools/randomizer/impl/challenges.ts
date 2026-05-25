/**
 * Challenge registry. Each challenge is one of three kinds (PLAIN / SQUAD_FILTER / STAGE).
 *
 * To add a new challenge, just push an entry into the appropriate section.
 *  - Plain: cosmetic / honor-system rule.
 *  - Squad filter: predicate over the full IOperatorListItem; restricts the squad pool.
 *  - Stage: predicate over IStage; only eligible when the rolled stage matches.
 *
 * The picker handles eligibility (stage match, sufficient pool size for filters)
 * and weighted random selection automatically - adding an entry here is the only
 * step needed.
 */

import { rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import type { IStage } from "#/types/stages";
import type { IChallenge } from "./types";

const SP_OFFENSIVE = "INCREASE_WHEN_ATTACK";
const SP_DEFENSIVE = "INCREASE_WHEN_TAKEN_DAMAGE";

/** True if any of the operator's skills carry the given spType at any level. */
function hasSpType(op: IOperatorListItem, spType: string): boolean {
    for (const skill of op.skills ?? []) {
        const levels = skill.static?.levels ?? [];
        for (const lvl of levels) {
            if (lvl.spData?.spType === spType) return true;
        }
    }
    return false;
}

/** Stage codes like "8-16", "13-21", "S4-9". Splits to ["8","16"], ["S4","9"]. */
function codeParts(stage: IStage): { chapter: string; part: string } | null {
    const m = stage.code.match(/^([^-]+)-(.+)$/);
    if (!m?.[1] || !m?.[2]) return null;
    return { chapter: m[1], part: m[2] };
}

const PLAIN_CHALLENGES: IChallenge[] = [
    { id: "speed-run", type: "PLAIN", kind: "modifier", title: "Speed run", description: "Complete the stage as quickly as you can." },
    { id: "minimal-deployment", type: "PLAIN", kind: "modifier", title: "Minimal deployment", description: "Use the fewest operators possible." },
    { id: "no-retreat", type: "PLAIN", kind: "modifier", title: "No retreating", description: "Once deployed, no operator may be retreated." },
    { id: "roster-order", type: "PLAIN", kind: "modifier", title: "Roster order", description: "Deploy operators left-to-right in the assigned order." },
    { id: "little-loneliness", type: "PLAIN", kind: "modifier", title: "Little Loneliness", description: "No other operators may be deployed within the arrow-rain range (eg. Texas skill2 range) of an operator." },
    { id: "straight-line", type: "PLAIN", kind: "modifier", title: "Straight Line", description: "Operators may only be deployed along a single straight line." },
    { id: "come-to-my-side", type: "PLAIN", kind: "modifier", title: "Come to My Side", description: "Operators may only be deployed within the arrow-rain range (eg. Texas skill2 range) of other operators (the first operator is exempt)." },
    { id: "vision-invitation", type: "PLAIN", kind: "modifier", title: "Vision Invitation", description: "Operators must be deployed within the current attack range of all deployed operators (the first operator is exempt)." },
    { id: "no-damage", type: "PLAIN", kind: "modifier", title: "No Damage", description: "Friendly units must not lose HP from enemy attacks." },
    { id: "with-damage", type: "PLAIN", kind: "modifier", title: "With Damage", description: "All deployed operators must take damage." },
    { id: "crowded-era", type: "PLAIN", kind: "modifier", title: "Crowded Era", description: "Only 3 deployment slots may be used." },
    { id: "ghost-leader-army", type: "PLAIN", kind: "modifier", title: "Ghost Leader Army", description: "Friendly units are forbidden from blocking enemies." },
    { id: "burned-out", type: "PLAIN", kind: "modifier", title: "Burned Out", description: "Operators must be retreated when their skill ends." },
    { id: "north-south-east-west", type: "PLAIN", kind: "modifier", title: "North, South, East, West", description: "All operators must face the same direction." },
];

const SQUAD_FILTER_CHALLENGES: IChallenge[] = [
    {
        id: "low-rarity",
        type: "SQUAD_FILTER",
        kind: "restriction",
        title: "Low rarity only",
        description: "Only 1★–3★ operators allowed.",
        filter: (op) => rarityToNumber(op.rarity) <= 3,
    },
    {
        id: "four-star-ceiling",
        type: "SQUAD_FILTER",
        kind: "restriction",
        title: "Four-star ceiling",
        description: "No operators above 4★ rarity.",
        filter: (op) => rarityToNumber(op.rarity) <= 4,
    },
    {
        id: "five-star-ceiling",
        type: "SQUAD_FILTER",
        kind: "restriction",
        title: "Five-star ceiling",
        description: "No operators above 5★ rarity.",
        filter: (op) => rarityToNumber(op.rarity) <= 5,
    },
    {
        id: "four-star-rarity",
        type: "SQUAD_FILTER",
        kind: "restriction",
        title: "4 stars only",
        description: "Only 4★ operators allowed.",
        filter: (op) => rarityToNumber(op.rarity) === 4,
    },
    {
        id: "five-star-rarity",
        type: "SQUAD_FILTER",
        kind: "restriction",
        title: "5 stars only",
        description: "Only 5★ operators allowed.",
        filter: (op) => rarityToNumber(op.rarity) === 5,
    },
    {
        id: "ranged-only",
        type: "SQUAD_FILTER",
        kind: "restriction",
        title: "Ranged only",
        description: "Only ranged operators allowed.",
        filter: (op) => op.position === "RANGED",
    },
    {
        id: "melee-only",
        type: "SQUAD_FILTER",
        kind: "restriction",
        title: "Melee only",
        description: "Only melee operators allowed.",
        filter: (op) => op.position === "MELEE",
    },
    {
        id: "no-medics",
        type: "SQUAD_FILTER",
        kind: "restriction",
        title: "No Medics",
        description: "Cannot deploy any Medic operators.",
        filter: (op) => op.profession !== "MEDIC",
    },
    {
        id: "offensive-recovery",
        type: "SQUAD_FILTER",
        kind: "restriction",
        title: "Offensive Recovery only",
        description: "You may only use offensive recovery skills.",
        filter: (op) => hasSpType(op, SP_OFFENSIVE),
    },
    {
        id: "defensive-recovery",
        type: "SQUAD_FILTER",
        kind: "restriction",
        title: "Defensive Recovery only",
        description: "You may only use defensive recovery skills.",
        filter: (op) => hasSpType(op, SP_DEFENSIVE),
    },
    {
        id: "elf-operators",
        type: "SQUAD_FILTER",
        kind: "modifier",
        title: "Elf Operators",
        description: "Operators must equip Skill 1.",
        filter: (op) => rarityToNumber(op.rarity) >= 4,
    },
    {
        id: "crisis-awareness",
        type: "SQUAD_FILTER",
        kind: "modifier",
        title: "Crisis Awareness",
        description: "Operators may not activate skills until their HP drops below 50%.",
        filter: (op) => {
            const skills = op.skills ?? [];
            if (skills.length === 0) return false;
            return skills.every((skill) => {
                const type = skill.static?.levels[0]?.skillType;
                return type === "MANUAL" || type === "1";
            });
        },
    },
    {
        id: "hoshiguma-mimi-meow-meow",
        type: "SQUAD_FILTER",
        kind: "modifier",
        title: "Hoshiguma Mimi Meow Meow",
        description: "Only “Feline” operators may be brought.",
        filter: (op) => op.profile?.basicInfo.race === "Feline",
    },
];

const STAGE_CHALLENGES: IChallenge[] = [
    {
        id: "boss-solo",
        type: "STAGE",
        kind: "restriction",
        title: "Boss solo",
        description: "Only one operator may be deployed during the boss phase.",
        match: (stage) => stage.bossMark,
        weight: 0.5,
    },
    {
        id: "annihilation-one-operator",
        type: "STAGE",
        kind: "objective",
        title: "Annihilation: 1P Relay",
        description: "You must do a 1 operator relay (eg. only one operator deployed at a time).",
        match: (stage) => {
            const parts = codeParts(stage);
            return parts?.chapter === "0";
        },
    },
];

export const CHALLENGES: IChallenge[] = [...PLAIN_CHALLENGES, ...SQUAD_FILTER_CHALLENGES, ...STAGE_CHALLENGES];
