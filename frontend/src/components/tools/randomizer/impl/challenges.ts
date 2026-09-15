/**
 * Challenge registry. Each challenge is one of three kinds (PLAIN / SQUAD_FILTER / STAGE).
 *
 * To add a new challenge, just push an entry into the appropriate section.
 *  - Plain: cosmetic / honor-system rule.
 *  - Squad filter: predicate over the slim IRandomizerOperator; restricts the squad pool.
 *  - Stage: predicate over IStage; only eligible when the rolled stage matches.
 *
 * The picker handles eligibility (stage match, sufficient pool size for filters)
 * and weighted random selection automatically - adding an entry here is the only
 * step needed.
 */

import type { messages as challengeMessages } from "./challenges.messages";
import type { IChallenge } from "./types";

/** A key in `challenges.messages.ts`; resolved by whichever component renders it. */
export type ChallengeMessageKey = keyof typeof challengeMessages & string;

const PLAIN_CHALLENGES: IChallenge[] = [
    { id: "speed-run", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.speedRun.title", descKey: "randomizer.challenge.speedRun.desc" },
    { id: "minimal-deployment", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.minimalDeployment.title", descKey: "randomizer.challenge.minimalDeployment.desc" },
    { id: "no-retreat", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.noRetreat.title", descKey: "randomizer.challenge.noRetreat.desc" },
    { id: "roster-order", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.rosterOrder.title", descKey: "randomizer.challenge.rosterOrder.desc" },
    { id: "little-loneliness", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.littleLoneliness.title", descKey: "randomizer.challenge.littleLoneliness.desc" },
    { id: "straight-line", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.straightLine.title", descKey: "randomizer.challenge.straightLine.desc" },
    { id: "come-to-my-side", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.comeToMySide.title", descKey: "randomizer.challenge.comeToMySide.desc" },
    { id: "vision-invitation", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.visionInvitation.title", descKey: "randomizer.challenge.visionInvitation.desc" },
    { id: "no-damage", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.noDamage.title", descKey: "randomizer.challenge.noDamage.desc" },
    { id: "with-damage", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.withDamage.title", descKey: "randomizer.challenge.withDamage.desc" },
    { id: "crowded-era", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.crowdedEra.title", descKey: "randomizer.challenge.crowdedEra.desc" },
    { id: "ghost-leader-army", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.ghostLeaderArmy.title", descKey: "randomizer.challenge.ghostLeaderArmy.desc" },
    { id: "burned-out", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.burnedOut.title", descKey: "randomizer.challenge.burnedOut.desc" },
    { id: "north-south-east-west", type: "PLAIN", kind: "modifier", titleKey: "randomizer.challenge.northSouthEastWest.title", descKey: "randomizer.challenge.northSouthEastWest.desc" },
];

const SQUAD_FILTER_CHALLENGES: IChallenge[] = [
    {
        id: "low-rarity",
        type: "SQUAD_FILTER",
        kind: "restriction",
        titleKey: "randomizer.challenge.lowRarity.title",
        descKey: "randomizer.challenge.lowRarity.desc",
        filter: (op) => op.rarity <= 3,
    },
    {
        id: "four-star-ceiling",
        type: "SQUAD_FILTER",
        kind: "restriction",
        titleKey: "randomizer.challenge.fourStarCeiling.title",
        descKey: "randomizer.challenge.fourStarCeiling.desc",
        filter: (op) => op.rarity <= 4,
    },
    {
        id: "five-star-ceiling",
        type: "SQUAD_FILTER",
        kind: "restriction",
        titleKey: "randomizer.challenge.fiveStarCeiling.title",
        descKey: "randomizer.challenge.fiveStarCeiling.desc",
        filter: (op) => op.rarity <= 5,
    },
    {
        id: "four-star-rarity",
        type: "SQUAD_FILTER",
        kind: "restriction",
        titleKey: "randomizer.challenge.fourStarRarity.title",
        descKey: "randomizer.challenge.fourStarRarity.desc",
        filter: (op) => op.rarity === 4,
    },
    {
        id: "five-star-rarity",
        type: "SQUAD_FILTER",
        kind: "restriction",
        titleKey: "randomizer.challenge.fiveStarRarity.title",
        descKey: "randomizer.challenge.fiveStarRarity.desc",
        filter: (op) => op.rarity === 5,
    },
    {
        id: "ranged-only",
        type: "SQUAD_FILTER",
        kind: "restriction",
        titleKey: "randomizer.challenge.rangedOnly.title",
        descKey: "randomizer.challenge.rangedOnly.desc",
        filter: (op) => op.position === "RANGED",
    },
    {
        id: "melee-only",
        type: "SQUAD_FILTER",
        kind: "restriction",
        titleKey: "randomizer.challenge.meleeOnly.title",
        descKey: "randomizer.challenge.meleeOnly.desc",
        filter: (op) => op.position === "MELEE",
    },
    {
        id: "no-medics",
        type: "SQUAD_FILTER",
        kind: "restriction",
        titleKey: "randomizer.challenge.noMedics.title",
        descKey: "randomizer.challenge.noMedics.desc",
        filter: (op) => op.profession !== "MEDIC",
    },
    {
        id: "offensive-recovery",
        type: "SQUAD_FILTER",
        kind: "restriction",
        titleKey: "randomizer.challenge.offensiveRecovery.title",
        descKey: "randomizer.challenge.offensiveRecovery.desc",
        filter: (op) => op.hasOffensiveRecovery,
    },
    {
        id: "defensive-recovery",
        type: "SQUAD_FILTER",
        kind: "restriction",
        titleKey: "randomizer.challenge.defensiveRecovery.title",
        descKey: "randomizer.challenge.defensiveRecovery.desc",
        filter: (op) => op.hasDefensiveRecovery,
    },
    {
        id: "elf-operators",
        type: "SQUAD_FILTER",
        kind: "modifier",
        titleKey: "randomizer.challenge.elfOperators.title",
        descKey: "randomizer.challenge.elfOperators.desc",
        filter: (op) => op.rarity >= 4,
    },
    {
        id: "crisis-awareness",
        type: "SQUAD_FILTER",
        kind: "modifier",
        titleKey: "randomizer.challenge.crisisAwareness.title",
        descKey: "randomizer.challenge.crisisAwareness.desc",
        filter: (op) => op.allSkillsManual,
    },
    {
        id: "hoshiguma-mimi-meow-meow",
        type: "SQUAD_FILTER",
        kind: "modifier",
        titleKey: "randomizer.challenge.hoshigumaMimiMeowMeow.title",
        descKey: "randomizer.challenge.hoshigumaMimiMeowMeow.desc",
        filter: (op) => op.race === "Feline",
    },
];

const STAGE_CHALLENGES: IChallenge[] = [
    {
        id: "boss-solo",
        type: "STAGE",
        kind: "restriction",
        titleKey: "randomizer.challenge.bossSolo.title",
        descKey: "randomizer.challenge.bossSolo.desc",
        match: (stage) => stage.bossMark,
        weight: 0.5,
    },
    {
        id: "annihilation-one-operator",
        type: "STAGE",
        kind: "objective",
        titleKey: "randomizer.challenge.annihilationOneOperator.title",
        descKey: "randomizer.challenge.annihilationOneOperator.desc",
        // Annihilation stages are the `camp_*` entries in stage_table, all typed
        // CAMPAIGN (their codes are region names like "Ursus"/"Yan", not "N-M"), so
        // match on stageType. Matching `code` chapter "0" was wrong: it never hit
        // Annihilation and instead matched the Prologue (Chapter 0) story stages.
        match: (stage) => stage.stageType === "CAMPAIGN",
    },
];

export const CHALLENGES: IChallenge[] = [...PLAIN_CHALLENGES, ...SQUAD_FILTER_CHALLENGES, ...STAGE_CHALLENGES];
