import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

/**
 * Display labels for the engine's conditional toggles. The engine names each one with a raw
 * label from the transpiled Python (`conditionals[].name`); `conditionals.ts` maps those
 * names, and per-operator overrides where the name is a truncated label, onto these keys.
 */
export const messages = {
    "calc.cond.20Stacks": {
        text: "20 stacks",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '20Stacks'). Short; shown beside a toggle.",
    },
    "calc.cond.2plusRangedTilesNearby": {
        text: "2+ ranged tiles nearby",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '2'). Short; shown beside a toggle.",
    },
    "calc.cond.3InRange": {
        text: "3 in range",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '3inRange'). Short; shown beside a toggle.",
    },
    "calc.cond.3Kills": {
        text: "3 kills",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '3kills'). Short; shown beside a toggle.",
    },
    "calc.cond.3ShieldsBroken": {
        text: "3 shields broken",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '3shieldsBroken'). Short; shown beside a toggle.",
    },
    "calc.cond.3plusEnemiesNearby": {
        text: "3+ enemies nearby",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '3'). Short; shown beside a toggle.",
    },
    "calc.cond.4Hits": {
        text: "4 hits",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '4hits'). Short; shown beside a toggle.",
    },
    "calc.cond.5ModuleStacks": {
        text: "5 module stacks",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '5modStacks'). Short; shown beside a toggle.",
    },
    "calc.cond.abyssalHunterInRange": {
        text: "Abyssal Hunter in range",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'AHinRange'). Short; shown beside a toggle.",
    },
    "calc.cond.adjacentAllies": {
        text: "Adjacent allies",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'adjacentAllies'). Short; shown beside a toggle.",
    },
    "calc.cond.adjacentAlly": {
        text: "Adjacent ally",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'adjacentAlly'). Short; shown beside a toggle.",
    },
    "calc.cond.aerialTarget": {
        text: "Aerial target",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'aerial', 'aerialTarget'). Short; shown beside a toggle.",
    },
    "calc.cond.after10S": {
        text: "After 10 s",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'after10s'). Short; shown beside a toggle.",
    },
    "calc.cond.after3BigHits": {
        text: "After 3 big hits",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'after3BigHits'). Short; shown beside a toggle.",
    },
    "calc.cond.after5S": {
        text: "After 5 s",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'after5sec'). Short; shown beside a toggle.",
    },
    "calc.cond.afterADodge": {
        text: "After a dodge",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'afterDodge'). Short; shown beside a toggle.",
    },
    "calc.cond.afterRevive": {
        text: "After revive",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'afterRevive'). Short; shown beside a toggle.",
    },
    "calc.cond.afterStealing": {
        text: "After stealing",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'afterSteal'). Short; shown beside a toggle.",
    },
    "calc.cond.afterTheTalentSHits": {
        text: "After the talent's hits",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'After' (char_4117_ray), 'after'). Short; shown beside a toggle.",
    },
    "calc.cond.afterTheTalentSKills": {
        text: "After the talent's kills",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'After' (char_333_sidero)). Short; shown beside a toggle.",
    },
    "calc.cond.afterTheTalentTrigger": {
        text: "After the talent trigger",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'After'). Short; shown beside a toggle.",
    },
    "calc.cond.allyInRange": {
        text: "Ally in range",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'AllyInRange'). Short; shown beside a toggle.",
    },
    "calc.cond.alone": {
        text: "Alone",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'alone'). Short; shown beside a toggle.",
    },
    "calc.cond.aspdBuffWithSkill": {
        text: "ASPD buff with skill",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'Apsd'). Short; shown beside a toggle.",
    },
    "calc.cond.atMaxRange": {
        text: "At max range",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'atMaxRange'). Short; shown beside a toggle.",
    },
    "calc.cond.atRange": {
        text: "At range",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'atRange'). Short; shown beside a toggle.",
    },
    "calc.cond.averageBurn": {
        text: "Average Burn",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'avgBurn'). Short; shown beside a toggle.",
    },
    "calc.cond.averageNecrosis": {
        text: "Average Necrosis",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'avgNecrosis'). Short; shown beside a toggle.",
    },
    "calc.cond.averageNervousImpairment": {
        text: "Average Nervous Impairment",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'avgNerv'). Short; shown beside a toggle.",
    },
    "calc.cond.beforeTheKill": {
        text: "Before the kill",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'preKill'). Short; shown beside a toggle.",
    },
    "calc.cond.beforeTheTalentSTimeLimit": {
        text: "Before the talent's time limit",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'f' (char_4193_lemuen)). Short; shown beside a toggle.",
    },
    "calc.cond.blocked": {
        text: "Blocked",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'blocked'). Short; shown beside a toggle.",
    },
    "calc.cond.blockedTarget": {
        text: "Blocked target",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'blockedTarget'). Short; shown beside a toggle.",
    },
    "calc.cond.blocking": {
        text: "Blocking",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'blocking'). Short; shown beside a toggle.",
    },
    "calc.cond.blocking2Plus": {
        text: "Blocking 2+",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'blocking2'). Short; shown beside a toggle.",
    },
    "calc.cond.bottlesHitTwice": {
        text: "Bottles hit twice",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '2HitBottles'). Short; shown beside a toggle.",
    },
    "calc.cond.catOnly": {
        text: "Cat only",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'onlyCat'). Short; shown beside a toggle.",
    },
    "calc.cond.centralHit": {
        text: "Central hit",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'central'). Short; shown beside a toggle.",
    },
    "calc.cond.charged": {
        text: "Charged",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'charged'). Short; shown beside a toggle.",
    },
    "calc.cond.conditionMet": {
        text: "Condition met",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'f'). Short; shown beside a toggle.",
    },
    "calc.cond.damageTaken": {
        text: "Damage taken",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'dmgTaken'). Short; shown beside a toggle.",
    },
    "calc.cond.directlyInFront": {
        text: "Directly in front",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'DirectFront', 'directFront'). Short; shown beside a toggle.",
    },
    "calc.cond.distant": {
        text: "Distant",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'distant'). Short; shown beside a toggle.",
    },
    "calc.cond.enemyClose": {
        text: "Enemy close",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'EnemyClose'). Short; shown beside a toggle.",
    },
    "calc.cond.fever": {
        text: "Fever",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'Fever', 'f' (char_4182_oblvns), 'fever'). Short; shown beside a toggle.",
    },
    "calc.cond.firstActivation": {
        text: "First activation",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '1stActivation', 'firstActivation'). Short; shown beside a toggle.",
    },
    "calc.cond.fourthActivation": {
        text: "Fourth activation",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '4thActivation'). Short; shown beside a toggle.",
    },
    "calc.cond.fullSquad": {
        text: "Full squad",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'FullSquad'). Short; shown beside a toggle.",
    },
    "calc.cond.gotHit": {
        text: "Got hit",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'gotHit'). Short; shown beside a toggle.",
    },
    "calc.cond.groundEnemies": {
        text: "Ground enemies",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'groundEnemies'). Short; shown beside a toggle.",
    },
    "calc.cond.groundEnemy": {
        text: "Ground enemy",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'GroundEnemy'). Short; shown beside a toggle.",
    },
    "calc.cond.groundTargets": {
        text: "Ground targets",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'GroundTargets'). Short; shown beside a toggle.",
    },
    "calc.cond.groundUnit": {
        text: "Ground unit",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'groundUnit'). Short; shown beside a toggle.",
    },
    "calc.cond.halfStacks": {
        text: "Half stacks",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'halfStacks'). Short; shown beside a toggle.",
    },
    "calc.cond.halfVigor": {
        text: "Half Vigor",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'half'). Short; shown beside a toggle.",
    },
    "calc.cond.healingMelee": {
        text: "Healing melee",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'healingMelee'). Short; shown beside a toggle.",
    },
    "calc.cond.healsRatherThanAttacks": {
        text: "Heals rather than attacks",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'heals'). Short; shown beside a toggle.",
    },
    "calc.cond.highGroundTarget": {
        text: "High-ground target",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'highGroundTarget'). Short; shown beside a toggle.",
    },
    "calc.cond.highHPTarget": {
        text: "High-HP target",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'f' (char_4056_titi)). Short; shown beside a toggle.",
    },
    "calc.cond.iceOnly": {
        text: "Ice only",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'iceOnly'). Short; shown beside a toggle.",
    },
    "calc.cond.idealCase": {
        text: "Ideal case",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'idealCase'). Short; shown beside a toggle.",
    },
    "calc.cond.idealTalentUsage": {
        text: "Ideal talent usage",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'idealTalentUsage'). Short; shown beside a toggle.",
    },
    "calc.cond.inIS3": {
        text: "In IS3",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'in'). Short; shown beside a toggle.",
    },
    "calc.cond.inIntegratedStrategies": {
        text: "In Integrated Strategies",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'inIS'). Short; shown beside a toggle.",
    },
    "calc.cond.initialHit": {
        text: "Initial hit",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'initialHit'). Short; shown beside a toggle.",
    },
    "calc.cond.killingHitsOnly": {
        text: "Killing hits only",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'KillingHitsOnly'). Short; shown beside a toggle.",
    },
    "calc.cond.lastStand": {
        text: "Last stand",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'lastStand'). Short; shown beside a toggle.",
    },
    "calc.cond.liftoffSelfBuff": {
        text: "Liftoff self-buff",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'liftoffSelfBuff'). Short; shown beside a toggle.",
    },
    "calc.cond.lowHP": {
        text: "Low HP",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'lowHP', 'lowHp'). Short; shown beside a toggle.",
    },
    "calc.cond.lowHPTarget": {
        text: "Low-HP target",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'f' (char_4071_peper), 'lowHpTarget'). Short; shown beside a toggle.",
    },
    "calc.cond.lowTalentBonus": {
        text: "Low talent bonus",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'LowTalent'). Short; shown beside a toggle.",
    },
    "calc.cond.manyTargets": {
        text: "Many targets",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'manyTargets'). Short; shown beside a toggle.",
    },
    "calc.cond.maxFirstTalent": {
        text: "Max first talent",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'maxTalent1'). Short; shown beside a toggle.",
    },
    "calc.cond.maxModuleBonus": {
        text: "Max module bonus",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'maxModuleBonus'). Short; shown beside a toggle.",
    },
    "calc.cond.maxModuleStacks": {
        text: "Max module stacks",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'maxModStacks'). Short; shown beside a toggle.",
    },
    "calc.cond.maxNotes": {
        text: "Max notes",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'maxNotes'). Short; shown beside a toggle.",
    },
    "calc.cond.maxRange": {
        text: "Max range",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'maxRange'). Short; shown beside a toggle.",
    },
    "calc.cond.maxSecondTalent": {
        text: "Max second talent",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'maxTalent2'). Short; shown beside a toggle.",
    },
    "calc.cond.maxSkillDamage": {
        text: "Max skill damage",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'maxSkillDmg'). Short; shown beside a toggle.",
    },
    "calc.cond.maxStacks": {
        text: "Max stacks",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'maxStacks'). Short; shown beside a toggle.",
    },
    "calc.cond.maxSteal": {
        text: "Max steal",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'maxSteal'). Short; shown beside a toggle.",
    },
    "calc.cond.maxTenacity": {
        text: "Max Tenacity",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'maxTenacity'). Short; shown beside a toggle.",
    },
    "calc.cond.melee": {
        text: "Melee",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'meelee', 'melee'). Short; shown beside a toggle.",
    },
    "calc.cond.mineDebuff": {
        text: "Mine debuff",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'MineDebuff'). Short; shown beside a toggle.",
    },
    "calc.cond.mineInRange": {
        text: "Mine in range",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'MineInRange'). Short; shown beside a toggle.",
    },
    "calc.cond.minimumASPDBonus": {
        text: "Minimum ASPD bonus",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'minAspd'). Short; shown beside a toggle.",
    },
    "calc.cond.minimumDEFShred": {
        text: "Minimum DEF shred",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'minDefshred'). Short; shown beside a toggle.",
    },
    "calc.cond.minimumDroneDamage": {
        text: "Minimum drone damage",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'minDroneDmg'). Short; shown beside a toggle.",
    },
    "calc.cond.minimumNotes": {
        text: "Minimum notes",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'minNotes'). Short; shown beside a toggle.",
    },
    "calc.cond.minimumRESShred": {
        text: "Minimum RES shred",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'minResshred'). Short; shown beside a toggle.",
    },
    "calc.cond.minimumTraitBonus": {
        text: "Minimum trait bonus",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'minTrait'). Short; shown beside a toggle.",
    },
    "calc.cond.moduleRevive": {
        text: "Module revive",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'ModRevive'). Short; shown beside a toggle.",
    },
    "calc.cond.multipleTargets": {
        text: "Multiple targets",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'f' (char_1023_ghost2), 'f' (char_248_mgllan)). Short; shown beside a toggle.",
    },
    "calc.cond.nextToCannon": {
        text: "Next to cannon",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'nextToCannon'). Short; shown beside a toggle.",
    },
    "calc.cond.noBurn": {
        text: "No Burn",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'no' (char_1040_blaze2), 'no' (char_4081_warmy)). Short; shown beside a toggle.",
    },
    "calc.cond.noClones": {
        text: "No clones",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'no' (char_249_mlyss)). Short; shown beside a toggle.",
    },
    "calc.cond.noCoinStacks": {
        text: "No coin stacks",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noCoinStacks'). Short; shown beside a toggle.",
    },
    "calc.cond.noConstruct": {
        text: "No construct",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noConstruct'). Short; shown beside a toggle.",
    },
    "calc.cond.noCrits": {
        text: "No crits",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noCrits'). Short; shown beside a toggle.",
    },
    "calc.cond.noDP": {
        text: "No DP",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noDP'). Short; shown beside a toggle.",
    },
    "calc.cond.noDamageTaken": {
        text: "No damage taken",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noDmgTaken'). Short; shown beside a toggle.",
    },
    "calc.cond.noDragons": {
        text: "No dragons",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noDragons'). Short; shown beside a toggle.",
    },
    "calc.cond.noDrones": {
        text: "No drones",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noDrones'). Short; shown beside a toggle.",
    },
    "calc.cond.noEnemies": {
        text: "No enemies",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noEnemies'). Short; shown beside a toggle.",
    },
    "calc.cond.noGroundTiles": {
        text: "No ground tiles",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noGroundTiles'). Short; shown beside a toggle.",
    },
    "calc.cond.noMana": {
        text: "No mana",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noMana'). Short; shown beside a toggle.",
    },
    "calc.cond.noMines": {
        text: "No mines",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noMines'). Short; shown beside a toggle.",
    },
    "calc.cond.noModuleBonus": {
        text: "No module bonus",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noModBonus'). Short; shown beside a toggle.",
    },
    "calc.cond.noModuleStacks": {
        text: "No module stacks",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noModStacks'). Short; shown beside a toggle.",
    },
    "calc.cond.noNecrosis": {
        text: "No Necrosis",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noNecrosis'). Short; shown beside a toggle.",
    },
    "calc.cond.noRangedTiles": {
        text: "No ranged tiles",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'NoRangedTiles'). Short; shown beside a toggle.",
    },
    "calc.cond.noSecondMedic": {
        text: "No second Medic",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'No'). Short; shown beside a toggle.",
    },
    "calc.cond.noSecondTalentStacks": {
        text: "No second-talent stacks",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noTalent2Stacks'). Short; shown beside a toggle.",
    },
    "calc.cond.noSkillCrits": {
        text: "No skill crits",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'NoSkillCrit'). Short; shown beside a toggle.",
    },
    "calc.cond.noStacks": {
        text: "No stacks",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noStacks'). Short; shown beside a toggle.",
    },
    "calc.cond.noSummon": {
        text: "No summon",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noSummon'). Short; shown beside a toggle.",
    },
    "calc.cond.noSummons": {
        text: "No summons",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noSummons'). Short; shown beside a toggle.",
    },
    "calc.cond.noTalentBuffs": {
        text: "No talent buffs",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noTalentBuffs'). Short; shown beside a toggle.",
    },
    "calc.cond.noTalentStacks": {
        text: "No talent stacks",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'noTalentStacks'). Short; shown beside a toggle.",
    },
    "calc.cond.notBlocking": {
        text: "Not blocking",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'NotBlocking'). Short; shown beside a toggle.",
    },
    "calc.cond.notCharged": {
        text: "Not charged",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'NotCharged'). Short; shown beside a toggle.",
    },
    "calc.cond.notInHealRange": {
        text: "Not in heal range",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'NotInHealRange'). Short; shown beside a toggle.",
    },
    "calc.cond.oneDragon": {
        text: "One dragon",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '1Dragon'). Short; shown beside a toggle.",
    },
    "calc.cond.oneDrone": {
        text: "One drone",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '1Drone'). Short; shown beside a toggle.",
    },
    "calc.cond.oneMinePerSkillCycle": {
        text: "One mine per skill cycle",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '1MinePerSPcost'). Short; shown beside a toggle.",
    },
    "calc.cond.oneStack": {
        text: "One stack",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '1Stack'). Short; shown beside a toggle.",
    },
    "calc.cond.oneSummon": {
        text: "One summon",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '1Summon'). Short; shown beside a toggle.",
    },
    "calc.cond.oneWolf": {
        text: "One wolf",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '1wolf'). Short; shown beside a toggle.",
    },
    "calc.cond.onethirdOfHitsVsFrozen": {
        text: "1/3 of hits vs. frozen",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '1'). Short; shown beside a toggle.",
    },
    "calc.cond.openingSeconds": {
        text: "Opening seconds",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'first', 'first' (char_187_ccheal), 'first' (char_286_cast3)). Short; shown beside a toggle.",
    },
    "calc.cond.openingSecondsVsDrones": {
        text: "Opening seconds vs. drones",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'first' (char_4000_jnight)). Short; shown beside a toggle.",
    },
    "calc.cond.organ": {
        text: "Organ",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'Organ'). Short; shown beside a toggle.",
    },
    "calc.cond.outOfAmmo": {
        text: "Out of ammo",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'outOfAmmo'). Short; shown beside a toggle.",
    },
    "calc.cond.overcharged": {
        text: "Overcharged",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'overcharged'). Short; shown beside a toggle.",
    },
    "calc.cond.overdrive": {
        text: "Overdrive",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'overdrive'). Short; shown beside a toggle.",
    },
    "calc.cond.overlappingHits": {
        text: "Overlapping hits",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'overlap', 'overlapp'). Short; shown beside a toggle.",
    },
    "calc.cond.rangedAttack": {
        text: "Ranged attack",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'rangedAtk'). Short; shown beside a toggle.",
    },
    "calc.cond.s3SelfBuff": {
        text: "S3 self-buff",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'selfbuffS3'). Short; shown beside a toggle.",
    },
    "calc.cond.sandwiched": {
        text: "Sandwiched",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'Sandwiched'). Short; shown beside a toggle.",
    },
    "calc.cond.secondActivation": {
        text: "Second activation",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: '2ndActivation'). Short; shown beside a toggle.",
    },
    "calc.cond.selfAppliedSleep": {
        text: "Self-applied Sleep",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'selfAppliedSleep'). Short; shown beside a toggle.",
    },
    "calc.cond.selfBuff": {
        text: "Self-buff",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'selfBuff'). Short; shown beside a toggle.",
    },
    "calc.cond.selfStun": {
        text: "Self-stun",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'selfstun'). Short; shown beside a toggle.",
    },
    "calc.cond.singleTargetOnly": {
        text: "Single target only",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'singleTargetOnly'). Short; shown beside a toggle.",
    },
    "calc.cond.skillSucceeds": {
        text: "Skill succeeds",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'SkillSuccess'). Short; shown beside a toggle.",
    },
    "calc.cond.talentActive": {
        text: "Talent active",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'TalentUp'). Short; shown beside a toggle.",
    },
    "calc.cond.talentAttackSpeed": {
        text: "Talent attack speed bonus",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'talentASPD'). Short; shown beside a toggle.",
    },
    "calc.cond.talentInactive": {
        text: "Talent inactive",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'talentInactive'). Short; shown beside a toggle.",
    },
    "calc.cond.talentOnOneShotPerMagazine": {
        text: "Talent on one shot per magazine",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'TalentOn1'). Short; shown beside a toggle.",
    },
    "calc.cond.talentSEnemyTypeBonus": {
        text: "Talent's enemy-type bonus",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'withRacism'). Short; shown beside a toggle.",
    },
    "calc.cond.talentStacks": {
        text: "Talent stacks",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'f' (char_1026_gvial2), 'f' (char_2015_dusk), 'f' (char_4146_nymph)). Short; shown beside a toggle.",
    },
    "calc.cond.talentSummons": {
        text: "Talent summons",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'f' (char_4164_tecno)). Short; shown beside a toggle.",
    },
    "calc.cond.tripleHit": {
        text: "Triple hit",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'tripleHit'). Short; shown beside a toggle.",
    },
    "calc.cond.turretOnly": {
        text: "Turret only",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'TurretOnly'). Short; shown beside a toggle.",
    },
    "calc.cond.typewriterAttacksSeparately": {
        text: "Typewriter attacks separately",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'TW'). Short; shown beside a toggle.",
    },
    "calc.cond.unitOnly": {
        text: "Unit only",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'unitOnly'). Short; shown beside a toggle.",
    },
    "calc.cond.versusTheHPThreshold": {
        text: "Versus the HP threshold",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vs'). Short; shown beside a toggle.",
    },
    "calc.cond.vs2PlusEnemies": {
        text: "vs. 2+ enemies",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vs2'). Short; shown beside a toggle.",
    },
    "calc.cond.vsAerial": {
        text: "vs. aerial",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsAerial'). Short; shown beside a toggle.",
    },
    "calc.cond.vsBindAndSlow": {
        text: "vs. Bind and Slow",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsBindAndSlow'). Short; shown beside a toggle.",
    },
    "calc.cond.vsBindOrSlow": {
        text: "vs. Bind or Slow",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsBindOrSlow'). Short; shown beside a toggle.",
    },
    "calc.cond.vsBlocked": {
        text: "vs. blocked",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsBlocked'). Short; shown beside a toggle.",
    },
    "calc.cond.vsBoss": {
        text: "vs. boss",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsBoss'). Short; shown beside a toggle.",
    },
    "calc.cond.vsBurn": {
        text: "vs. Burn",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsBurn'). Short; shown beside a toggle.",
    },
    "calc.cond.vsDrones": {
        text: "vs. drones",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsDrones'). Short; shown beside a toggle.",
    },
    "calc.cond.vsElite": {
        text: "vs. elite",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsElite'). Short; shown beside a toggle.",
    },
    "calc.cond.vsFallout": {
        text: "vs. Fallout",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsFallout'). Short; shown beside a toggle.",
    },
    "calc.cond.vsFrighten": {
        text: "vs. Frighten",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsFrighten'). Short; shown beside a toggle.",
    },
    "calc.cond.vsFrozen": {
        text: "vs. frozen",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsFrozen'). Short; shown beside a toggle.",
    },
    "calc.cond.vsHeavy": {
        text: "vs. heavy",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsHeavy'). Short; shown beside a toggle.",
    },
    "calc.cond.vsHighHP": {
        text: "vs. high HP",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsHighHp'). Short; shown beside a toggle.",
    },
    "calc.cond.vsHigherHP": {
        text: "vs. higher HP",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsHigherHP'). Short; shown beside a toggle.",
    },
    "calc.cond.vsInfected": {
        text: "vs. Infected",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsInfected'). Short; shown beside a toggle.",
    },
    "calc.cond.vsInvisible": {
        text: "vs. invisible",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsInvis'). Short; shown beside a toggle.",
    },
    "calc.cond.vsLevitateImmune": {
        text: "vs. Levitate-immune",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsLevitateImmune'). Short; shown beside a toggle.",
    },
    "calc.cond.vsLight": {
        text: "vs. light",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsLight'). Short; shown beside a toggle.",
    },
    "calc.cond.vsLowHP": {
        text: "vs. low HP",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsLowHp'). Short; shown beside a toggle.",
    },
    "calc.cond.vsMachine": {
        text: "vs. machine",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsMachine'). Short; shown beside a toggle.",
    },
    "calc.cond.vsMarked": {
        text: "vs. marked",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsMarked'). Short; shown beside a toggle.",
    },
    "calc.cond.vsMoving": {
        text: "vs. moving",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsMoving'). Short; shown beside a toggle.",
    },
    "calc.cond.vsNegativeStatus": {
        text: "vs. negative status",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsNegStat'). Short; shown beside a toggle.",
    },
    "calc.cond.vsNonSarkaz": {
        text: "vs. non-Sarkaz",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsNonSarkaz'). Short; shown beside a toggle.",
    },
    "calc.cond.vsParalysed": {
        text: "vs. paralysed",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsPara'). Short; shown beside a toggle.",
    },
    "calc.cond.vsRanged": {
        text: "vs. ranged",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsRanged'). Short; shown beside a toggle.",
    },
    "calc.cond.vsSeaborn": {
        text: "vs. Seaborn",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsSeaborn'). Short; shown beside a toggle.",
    },
    "calc.cond.vsSelfAppliedStun": {
        text: "vs. self-applied Stun",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsSelfAppliedStun'). Short; shown beside a toggle.",
    },
    "calc.cond.vsSleep": {
        text: "vs. Sleep",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsSleep'). Short; shown beside a toggle.",
    },
    "calc.cond.vsSlow": {
        text: "vs. Slow",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsSlow'). Short; shown beside a toggle.",
    },
    "calc.cond.vsSlowWithSelfAppliedBind": {
        text: "vs. Slow with self-applied Bind",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsSlow' (char_4082_qiubai)). Short; shown beside a toggle.",
    },
    "calc.cond.vsStun": {
        text: "vs. Stun",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsStun'). Short; shown beside a toggle.",
    },
    "calc.cond.vsStunOrBind": {
        text: "vs. Stun or Bind",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsStun' (char_4088_hodrer)). Short; shown beside a toggle.",
    },
    "calc.cond.vsStunned": {
        text: "vs. stunned",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vsStunned'). Short; shown beside a toggle.",
    },
    "calc.cond.vsTargetAbove50HP": {
        text: "vs. target above 50% HP",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vs' (char_4196_reckpr)). Short; shown beside a toggle.",
    },
    "calc.cond.vsTargetBelow1HP": {
        text: "vs. target below 1% HP",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'vs' (char_145_prove)). Short; shown beside a toggle.",
    },
    "calc.cond.waterMap": {
        text: "Water map",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'watermap'). Short; shown beside a toggle.",
    },
    "calc.cond.weightlessTarget": {
        text: "Weightless target",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'weightlessTarget'). Short; shown beside a toggle.",
    },
    "calc.cond.whileBlocking": {
        text: "While blocking",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'WhileBlocking'). Short; shown beside a toggle.",
    },
    "calc.cond.withAverageNecrosis": {
        text: "With average Necrosis",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'withAvgNecrosis'). Short; shown beside a toggle.",
    },
    "calc.cond.withBonus": {
        text: "With bonus",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'withBonus'). Short; shown beside a toggle.",
    },
    "calc.cond.withClone": {
        text: "With clone",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'with' (char_250_phatom)). Short; shown beside a toggle.",
    },
    "calc.cond.withPet": {
        text: "With pet",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'with' (char_4117_ray)). Short; shown beside a toggle.",
    },
    "calc.cond.withPillarDEFShred": {
        text: "With pillar DEF shred",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'withPillarDefshred'). Short; shown beside a toggle.",
    },
    "calc.cond.withSPBoost": {
        text: "With SP boost",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'withSPboost'). Short; shown beside a toggle.",
    },
    "calc.cond.withTheBonus": {
        text: "With the bonus",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'with'). Short; shown beside a toggle.",
    },
    "calc.cond.withoutASecondSniper": {
        text: "Without a second Sniper",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'w' (char_340_shwaz)). Short; shown beside a toggle.",
    },
    "calc.cond.withoutCinder": {
        text: "Without Cinder",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'w' (char_1020_reed2)). Short; shown beside a toggle.",
    },
    "calc.cond.withoutDEFIgnore": {
        text: "Without DEF ignore",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'w' (char_103_angel)). Short; shown beside a toggle.",
    },
    "calc.cond.withoutDoll": {
        text: "Without doll",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'w' (char_4016_kazema)). Short; shown beside a toggle.",
    },
    "calc.cond.withoutFirstTalent": {
        text: "Without first talent",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'w' (char_4026_vulpis), 'w' (char_4082_qiubai)). Short; shown beside a toggle.",
    },
    "calc.cond.withoutSecondTalent": {
        text: "Without second talent",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'w' (char_017_huang), 'w' (char_4011_lessng)). Short; shown beside a toggle.",
    },
    "calc.cond.withoutTalent": {
        text: "Without talent",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'w' (char_252_bibeak), 'w' (char_4040_rockr), 'w' (char_430_fartth)). Short; shown beside a toggle.",
    },
    "calc.cond.withoutTheBonus": {
        text: "Without the bonus",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'w'). Short; shown beside a toggle.",
    },
    "calc.cond.withoutTheEffect": {
        text: "Without the effect",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'no'). Short; shown beside a toggle.",
    },
    "calc.cond.withoutTrait": {
        text: "Without trait",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'w' (char_485_pallas)). Short; shown beside a toggle.",
    },
    "calc.cond.withoutVigor": {
        text: "Without Vigor",
        description: "Label of a calculator switch for an operator's conditional damage or healing bonus (engine names: 'w' (char_300_phenxi)). Short; shown beside a toggle.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
