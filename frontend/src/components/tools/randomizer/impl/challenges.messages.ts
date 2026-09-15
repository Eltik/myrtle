import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The challenge registry in `challenges.ts` is plain data in a module with no
 * React, so each entry carries message KEYS and `ModifierSlab` resolves them
 * with `t()`.
 *
 * These are this site's own challenge rules, not game content, so they are
 * translated here. Operator names, class names and mechanic names quoted
 * inside them come from the game data and keep the game's own wording.
 */
export const namespace = "tools";

export const messages = {
    "randomizer.challenge.speedRun.title": {
        text: "Speed run",
        description: "Challenge name: finish the stage fast.",
    },
    "randomizer.challenge.speedRun.desc": {
        text: "Complete the stage as quickly as you can.",
        description: "Rule text under the challenge's own name.",
    },
    "randomizer.challenge.minimalDeployment.title": {
        text: "Minimal deployment",
        description: "Challenge name: deploy as few operators as possible. 'Deployment' is the game's word for placing an operator on the map.",
    },
    "randomizer.challenge.minimalDeployment.desc": {
        text: "Use the fewest operators possible.",
        description: "Rule text under the challenge's own name.",
    },
    "randomizer.challenge.noRetreat.title": {
        text: "No retreating",
        description: "Challenge name: operators may not be withdrawn. 'Retreat' is the game's own word for removing a deployed operator.",
    },
    "randomizer.challenge.noRetreat.desc": {
        text: "Once deployed, no operator may be retreated.",
        description: "'Retreat' is the game's own word for removing a deployed operator.",
    },
    "randomizer.challenge.rosterOrder.title": {
        text: "Roster order",
        description: "Challenge name: deploy in the order the squad is listed.",
    },
    "randomizer.challenge.rosterOrder.desc": {
        text: "Deploy operators left-to-right in the assigned order.",
        description: "In a right-to-left language, this means 'in the order the squad is listed'.",
    },
    "randomizer.challenge.littleLoneliness.title": {
        text: "Little Loneliness",
        description: "Challenge name, styled like an in-game achievement: operators must stand apart.",
    },
    "randomizer.challenge.littleLoneliness.desc": {
        text: "No other operators may be deployed within the arrow-rain range (eg. Texas skill2 range) of an operator.",
        description: "'Arrow rain' is the community name for the large diamond-shaped range shown as an example; Texas is an operator name and 'skill2' her second skill. Keep 'eg.' as the abbreviation it is.",
    },
    "randomizer.challenge.straightLine.title": {
        text: "Straight Line",
        description: "Challenge name, styled like an in-game achievement: every operator on one line.",
    },
    "randomizer.challenge.straightLine.desc": {
        text: "Operators may only be deployed along a single straight line.",
        description: "Rule text under the challenge's own name.",
    },
    "randomizer.challenge.comeToMySide.title": {
        text: "Come to My Side",
        description: "Challenge name, styled like an in-game achievement: operators must stand close together.",
    },
    "randomizer.challenge.comeToMySide.desc": {
        text: "Operators may only be deployed within the arrow-rain range (eg. Texas skill2 range) of other operators (the first operator is exempt).",
        description: "'Arrow rain' is the community name for a large diamond-shaped attack range; Texas is an operator name.",
    },
    "randomizer.challenge.visionInvitation.title": {
        text: "Vision Invitation",
        description: "Challenge name, styled like an in-game achievement: operators must stand inside each other's attack range.",
    },
    "randomizer.challenge.visionInvitation.desc": {
        text: "Operators must be deployed within the current attack range of all deployed operators (the first operator is exempt).",
        description: "Rule text under the challenge's own name.",
    },
    "randomizer.challenge.noDamage.title": {
        text: "No Damage",
        description: "Challenge name, styled like an in-game achievement: take no damage at all.",
    },
    "randomizer.challenge.noDamage.desc": {
        text: "Friendly units must not lose HP from enemy attacks.",
        description: "'HP' is the game's own abbreviation for health.",
    },
    "randomizer.challenge.withDamage.title": {
        text: "With Damage",
        description: "Challenge name, styled like an in-game achievement: every operator must take some damage.",
    },
    "randomizer.challenge.withDamage.desc": {
        text: "All deployed operators must take damage.",
        description: "Rule text under the challenge's own name.",
    },
    "randomizer.challenge.crowdedEra.title": {
        text: "Crowded Era",
        description: "Challenge name, styled like an in-game achievement: only three deployment slots.",
    },
    "randomizer.challenge.crowdedEra.desc": {
        text: "Only 3 deployment slots may be used.",
        description: "A deployment slot is one space in the squad bar at the bottom of the screen.",
    },
    "randomizer.challenge.ghostLeaderArmy.title": {
        text: "Ghost Leader Army",
        description: "Challenge name, styled like an in-game achievement: nothing may block enemies.",
    },
    "randomizer.challenge.ghostLeaderArmy.desc": {
        text: "Friendly units are forbidden from blocking enemies.",
        description: "'Block' is the game's own mechanic of stopping an enemy from advancing.",
    },
    "randomizer.challenge.burnedOut.title": {
        text: "Burned Out",
        description: "Challenge name, styled like an in-game achievement: retreat each operator as its skill ends.",
    },
    "randomizer.challenge.burnedOut.desc": {
        text: "Operators must be retreated when their skill ends.",
        description: "Rule text under the challenge's own name.",
    },
    "randomizer.challenge.northSouthEastWest.title": {
        text: "North, South, East, West",
        description: "Challenge name, styled like an in-game achievement: every operator faces the same way.",
    },
    "randomizer.challenge.northSouthEastWest.desc": {
        text: "All operators must face the same direction.",
        description: "Rule text under the challenge's own name.",
    },
    "randomizer.challenge.lowRarity.title": {
        text: "Low rarity only",
        description: "Challenge name: only the lowest star ratings.",
    },
    "randomizer.challenge.lowRarity.desc": {
        text: "Only 1\u2605-3\u2605 operators allowed.",
        description: "Rule text under the challenge's own name.",
    },
    "randomizer.challenge.fourStarCeiling.title": {
        text: "Four-star ceiling",
        description: "Challenge name: nothing above four stars.",
    },
    "randomizer.challenge.fourStarCeiling.desc": {
        text: "No operators above 4\u2605 rarity.",
        description: "Rule text under the challenge's own name.",
    },
    "randomizer.challenge.fiveStarCeiling.title": {
        text: "Five-star ceiling",
        description: "Challenge name: nothing above five stars.",
    },
    "randomizer.challenge.fiveStarCeiling.desc": {
        text: "No operators above 5\u2605 rarity.",
        description: "Rule text under the challenge's own name.",
    },
    "randomizer.challenge.fourStarRarity.title": {
        text: "4 stars only",
        description: "Challenge name: exactly four-star operators.",
    },
    "randomizer.challenge.fourStarRarity.desc": {
        text: "Only 4\u2605 operators allowed.",
        description: "Rule text under the challenge's own name.",
    },
    "randomizer.challenge.fiveStarRarity.title": {
        text: "5 stars only",
        description: "Challenge name: exactly five-star operators.",
    },
    "randomizer.challenge.fiveStarRarity.desc": {
        text: "Only 5\u2605 operators allowed.",
        description: "Rule text under the challenge's own name.",
    },
    "randomizer.challenge.rangedOnly.title": {
        text: "Ranged only",
        description: "Challenge name: only operators deployed on ranged tiles. 'Ranged' is the game's own deployment position.",
    },
    "randomizer.challenge.rangedOnly.desc": {
        text: "Only ranged operators allowed.",
        description: "'Ranged' is the game's own deployment position.",
    },
    "randomizer.challenge.meleeOnly.title": {
        text: "Melee only",
        description: "Challenge name: only operators deployed on melee tiles. 'Melee' is the game's own deployment position.",
    },
    "randomizer.challenge.meleeOnly.desc": {
        text: "Only melee operators allowed.",
        description: "'Melee' is the game's own deployment position.",
    },
    "randomizer.challenge.noMedics.title": {
        text: "No Medics",
        description: "Challenge name: no healers. 'Medic' is the game's own class name and comes from the game data.",
    },
    "randomizer.challenge.noMedics.desc": {
        text: "Cannot deploy any Medic operators.",
        description: "'Medic' is the game's own class name.",
    },
    "randomizer.challenge.offensiveRecovery.title": {
        text: "Offensive Recovery only",
        description: "Challenge name: only skills that charge by attacking. 'Offensive Recovery' is the game's own name for that skill-point type.",
    },
    "randomizer.challenge.offensiveRecovery.desc": {
        text: "You may only use offensive recovery skills.",
        description: "'Offensive recovery' is the game's own name for a skill that charges by attacking.",
    },
    "randomizer.challenge.defensiveRecovery.title": {
        text: "Defensive Recovery only",
        description: "Challenge name: only skills that charge by being hit. 'Defensive Recovery' is the game's own name for that skill-point type.",
    },
    "randomizer.challenge.defensiveRecovery.desc": {
        text: "You may only use defensive recovery skills.",
        description: "'Defensive recovery' is the game's own name for a skill that charges when the operator is hit.",
    },
    "randomizer.challenge.elfOperators.title": {
        text: "Elf Operators",
        description: "Challenge name, styled like an in-game achievement: everyone uses their first skill.",
    },
    "randomizer.challenge.elfOperators.desc": {
        text: "Operators must equip Skill 1.",
        description: "'Skill 1' is the game's own label for an operator's first skill.",
    },
    "randomizer.challenge.crisisAwareness.title": {
        text: "Crisis Awareness",
        description: "Challenge name, styled like an in-game achievement: hold skills until badly hurt.",
    },
    "randomizer.challenge.crisisAwareness.desc": {
        text: "Operators may not activate skills until their HP drops below 50%.",
        description: "'HP' is the game's own abbreviation for health.",
    },
    "randomizer.challenge.hoshigumaMimiMeowMeow.title": {
        text: "Hoshiguma Mimi Meow Meow",
        description: "Challenge name, a community joke naming the operator Hoshiguma: cat-like operators only. Leave the name as-is.",
    },
    "randomizer.challenge.hoshigumaMimiMeowMeow.desc": {
        text: "Only \u201cFeline\u201d operators may be brought.",
        description: "'Feline' is an operator race from the game data - keep the game's own word for it, and keep the curly quotation marks.",
    },
    "randomizer.challenge.bossSolo.title": {
        text: "Boss solo",
        description: "Challenge name: face the boss with one operator.",
    },
    "randomizer.challenge.bossSolo.desc": {
        text: "Only one operator may be deployed during the boss phase.",
        description: "Rule text under the challenge's own name.",
    },
    "randomizer.challenge.annihilationOneOperator.title": {
        text: "Annihilation: 1P Relay",
        description: "Challenge name for the Annihilation game mode: one operator at a time. 'Annihilation' is the mode's own name; '1P' means one player unit.",
    },
    "randomizer.challenge.annihilationOneOperator.desc": {
        text: "You must do a 1 operator relay (eg. only one operator deployed at a time).",
        description: "A 'relay' means retreating one operator before deploying the next. Keep 'eg.' as the abbreviation it is.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on a constants entry and resolved by the
// consuming component as `t(item.labelKey)`, so the extractor has no literal
// call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
