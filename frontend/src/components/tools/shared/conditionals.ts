import type { TypedT } from "#/lib/i18n/messages";
import type { messages as conditionalMessages } from "./conditionals.messages";
import type { IConditionalInfo } from "./types";

export type ConditionalLabelKey = keyof typeof conditionalMessages & string;
export type ConditionalT = TypedT<typeof conditionalMessages>;

/** Engine conditional name -> label key; every name in `operator_formulas.json` and `heal_formulas.json` on 2026-10-24. */
export const CONDITIONAL_NAME_KEYS: Readonly<Record<string, ConditionalLabelKey>> = {
    "1": "calc.cond.onethirdOfHitsVsFrozen",
    "1Dragon": "calc.cond.oneDragon",
    "1Drone": "calc.cond.oneDrone",
    "1MinePerSPcost": "calc.cond.oneMinePerSkillCycle",
    "1Stack": "calc.cond.oneStack",
    "1Summon": "calc.cond.oneSummon",
    "1stActivation": "calc.cond.firstActivation",
    "1wolf": "calc.cond.oneWolf",
    "2": "calc.cond.2plusRangedTilesNearby",
    "20Stacks": "calc.cond.20Stacks",
    "2HitBottles": "calc.cond.bottlesHitTwice",
    "2ndActivation": "calc.cond.secondActivation",
    "3": "calc.cond.3plusEnemiesNearby",
    "3inRange": "calc.cond.3InRange",
    "3kills": "calc.cond.3Kills",
    "3shieldsBroken": "calc.cond.3ShieldsBroken",
    "4hits": "calc.cond.4Hits",
    "4thActivation": "calc.cond.fourthActivation",
    "5modStacks": "calc.cond.5ModuleStacks",
    AHinRange: "calc.cond.abyssalHunterInRange",
    After: "calc.cond.afterTheTalentTrigger",
    AllyInRange: "calc.cond.allyInRange",
    Apsd: "calc.cond.aspdBuffWithSkill",
    DirectFront: "calc.cond.directlyInFront",
    EnemyClose: "calc.cond.enemyClose",
    Fever: "calc.cond.fever",
    FullSquad: "calc.cond.fullSquad",
    GroundEnemy: "calc.cond.groundEnemy",
    GroundTargets: "calc.cond.groundTargets",
    KillingHitsOnly: "calc.cond.killingHitsOnly",
    LowTalent: "calc.cond.lowTalentBonus",
    MineDebuff: "calc.cond.mineDebuff",
    MineInRange: "calc.cond.mineInRange",
    ModRevive: "calc.cond.moduleRevive",
    No: "calc.cond.noSecondMedic",
    NoRangedTiles: "calc.cond.noRangedTiles",
    NoSkillCrit: "calc.cond.noSkillCrits",
    NotBlocking: "calc.cond.notBlocking",
    NotCharged: "calc.cond.notCharged",
    NotInHealRange: "calc.cond.notInHealRange",
    Organ: "calc.cond.organ",
    Sandwiched: "calc.cond.sandwiched",
    SkillSuccess: "calc.cond.skillSucceeds",
    TW: "calc.cond.typewriterAttacksSeparately",
    TalentOn1: "calc.cond.talentOnOneShotPerMagazine",
    TalentUp: "calc.cond.talentActive",
    TurretOnly: "calc.cond.turretOnly",
    WhileBlocking: "calc.cond.whileBlocking",
    adjacentAllies: "calc.cond.adjacentAllies",
    adjacentAlly: "calc.cond.adjacentAlly",
    aerial: "calc.cond.aerialTarget",
    aerialTarget: "calc.cond.aerialTarget",
    after: "calc.cond.afterTheTalentSHits",
    after10s: "calc.cond.after10S",
    after3BigHits: "calc.cond.after3BigHits",
    after5sec: "calc.cond.after5S",
    afterDodge: "calc.cond.afterADodge",
    afterRevive: "calc.cond.afterRevive",
    afterSteal: "calc.cond.afterStealing",
    alone: "calc.cond.alone",
    atMaxRange: "calc.cond.atMaxRange",
    atRange: "calc.cond.atRange",
    avgBurn: "calc.cond.averageBurn",
    avgNecrosis: "calc.cond.averageNecrosis",
    avgNerv: "calc.cond.averageNervousImpairment",
    blocked: "calc.cond.blocked",
    blockedTarget: "calc.cond.blockedTarget",
    blocking: "calc.cond.blocking",
    blocking2: "calc.cond.blocking2Plus",
    central: "calc.cond.centralHit",
    charged: "calc.cond.charged",
    directFront: "calc.cond.directlyInFront",
    distant: "calc.cond.distant",
    dmgTaken: "calc.cond.damageTaken",
    f: "calc.cond.conditionMet",
    fever: "calc.cond.fever",
    first: "calc.cond.openingSeconds",
    firstActivation: "calc.cond.firstActivation",
    gotHit: "calc.cond.gotHit",
    groundEnemies: "calc.cond.groundEnemies",
    groundUnit: "calc.cond.groundUnit",
    half: "calc.cond.halfVigor",
    halfStacks: "calc.cond.halfStacks",
    healingMelee: "calc.cond.healingMelee",
    heals: "calc.cond.healsRatherThanAttacks",
    highGroundTarget: "calc.cond.highGroundTarget",
    iceOnly: "calc.cond.iceOnly",
    idealCase: "calc.cond.idealCase",
    idealTalentUsage: "calc.cond.idealTalentUsage",
    in: "calc.cond.inIS3",
    inIS: "calc.cond.inIntegratedStrategies",
    initialHit: "calc.cond.initialHit",
    lastStand: "calc.cond.lastStand",
    liftoffSelfBuff: "calc.cond.liftoffSelfBuff",
    lowHP: "calc.cond.lowHP",
    lowHp: "calc.cond.lowHP",
    lowHpTarget: "calc.cond.lowHPTarget",
    manyTargets: "calc.cond.manyTargets",
    maxModStacks: "calc.cond.maxModuleStacks",
    maxModuleBonus: "calc.cond.maxModuleBonus",
    maxNotes: "calc.cond.maxNotes",
    maxRange: "calc.cond.maxRange",
    maxSkillDmg: "calc.cond.maxSkillDamage",
    maxStacks: "calc.cond.maxStacks",
    maxSteal: "calc.cond.maxSteal",
    maxTalent1: "calc.cond.maxFirstTalent",
    maxTalent2: "calc.cond.maxSecondTalent",
    maxTenacity: "calc.cond.maxTenacity",
    meelee: "calc.cond.melee",
    melee: "calc.cond.melee",
    minAspd: "calc.cond.minimumASPDBonus",
    minDefshred: "calc.cond.minimumDEFShred",
    minDroneDmg: "calc.cond.minimumDroneDamage",
    minNotes: "calc.cond.minimumNotes",
    minResshred: "calc.cond.minimumRESShred",
    minTrait: "calc.cond.minimumTraitBonus",
    nextToCannon: "calc.cond.nextToCannon",
    no: "calc.cond.withoutTheEffect",
    noCoinStacks: "calc.cond.noCoinStacks",
    noConstruct: "calc.cond.noConstruct",
    noCrits: "calc.cond.noCrits",
    noDP: "calc.cond.noDP",
    noDmgTaken: "calc.cond.noDamageTaken",
    noDragons: "calc.cond.noDragons",
    noDrones: "calc.cond.noDrones",
    noEnemies: "calc.cond.noEnemies",
    noGroundTiles: "calc.cond.noGroundTiles",
    noMana: "calc.cond.noMana",
    noMines: "calc.cond.noMines",
    noModBonus: "calc.cond.noModuleBonus",
    noModStacks: "calc.cond.noModuleStacks",
    noNecrosis: "calc.cond.noNecrosis",
    noStacks: "calc.cond.noStacks",
    noSummon: "calc.cond.noSummon",
    noSummons: "calc.cond.noSummons",
    noTalent2Stacks: "calc.cond.noSecondTalentStacks",
    noTalentBuffs: "calc.cond.noTalentBuffs",
    noTalentStacks: "calc.cond.noTalentStacks",
    onlyCat: "calc.cond.catOnly",
    outOfAmmo: "calc.cond.outOfAmmo",
    overcharged: "calc.cond.overcharged",
    overdrive: "calc.cond.overdrive",
    overlap: "calc.cond.overlappingHits",
    overlapp: "calc.cond.overlappingHits",
    preKill: "calc.cond.beforeTheKill",
    rangedAtk: "calc.cond.rangedAttack",
    selfAppliedSleep: "calc.cond.selfAppliedSleep",
    selfBuff: "calc.cond.selfBuff",
    selfbuffS3: "calc.cond.s3SelfBuff",
    selfstun: "calc.cond.selfStun",
    singleTargetOnly: "calc.cond.singleTargetOnly",
    talentASPD: "calc.cond.talentAttackSpeed",
    talentInactive: "calc.cond.talentInactive",
    tripleHit: "calc.cond.tripleHit",
    unitOnly: "calc.cond.unitOnly",
    vs: "calc.cond.versusTheHPThreshold",
    vs2: "calc.cond.vs2PlusEnemies",
    vsAerial: "calc.cond.vsAerial",
    vsBindAndSlow: "calc.cond.vsBindAndSlow",
    vsBindOrSlow: "calc.cond.vsBindOrSlow",
    vsBlocked: "calc.cond.vsBlocked",
    vsBoss: "calc.cond.vsBoss",
    vsBurn: "calc.cond.vsBurn",
    vsDrones: "calc.cond.vsDrones",
    vsElite: "calc.cond.vsElite",
    vsFallout: "calc.cond.vsFallout",
    vsFrighten: "calc.cond.vsFrighten",
    vsFrozen: "calc.cond.vsFrozen",
    vsHeavy: "calc.cond.vsHeavy",
    vsHighHp: "calc.cond.vsHighHP",
    vsHigherHP: "calc.cond.vsHigherHP",
    vsInfected: "calc.cond.vsInfected",
    vsInvis: "calc.cond.vsInvisible",
    vsLevitateImmune: "calc.cond.vsLevitateImmune",
    vsLight: "calc.cond.vsLight",
    vsLowHp: "calc.cond.vsLowHP",
    vsMachine: "calc.cond.vsMachine",
    vsMarked: "calc.cond.vsMarked",
    vsMoving: "calc.cond.vsMoving",
    vsNegStat: "calc.cond.vsNegativeStatus",
    vsNonSarkaz: "calc.cond.vsNonSarkaz",
    vsPara: "calc.cond.vsParalysed",
    vsRanged: "calc.cond.vsRanged",
    vsSeaborn: "calc.cond.vsSeaborn",
    vsSelfAppliedStun: "calc.cond.vsSelfAppliedStun",
    vsSleep: "calc.cond.vsSleep",
    vsSlow: "calc.cond.vsSlow",
    vsStun: "calc.cond.vsStun",
    vsStunned: "calc.cond.vsStunned",
    w: "calc.cond.withoutTheBonus",
    watermap: "calc.cond.waterMap",
    weightlessTarget: "calc.cond.weightlessTarget",
    with: "calc.cond.withTheBonus",
    withAvgNecrosis: "calc.cond.withAverageNecrosis",
    withBonus: "calc.cond.withBonus",
    withPillarDefshred: "calc.cond.withPillarDEFShred",
    withRacism: "calc.cond.talentSEnemyTypeBonus",
    withSPboost: "calc.cond.withSPBoost",
};

/**
 * `charId|conditionalType|name` -> label key, for names that are a truncated Python label
 * (`w` from "w/o talent", `f` from an f-string) and so mean different things per operator.
 */
export const CONDITIONAL_OVERRIDE_KEYS: Readonly<Record<string, ConditionalLabelKey>> = {
    "char_017_huang|talent2|w": "calc.cond.withoutSecondTalent",
    "char_1020_reed2|talent|w": "calc.cond.withoutCinder",
    "char_1023_ghost2|trait|f": "calc.cond.multipleTargets",
    "char_1026_gvial2|talent|f": "calc.cond.talentStacks",
    "char_103_angel|talent|w": "calc.cond.withoutDEFIgnore",
    "char_1040_blaze2|trait|no": "calc.cond.noBurn",
    "char_145_prove|skill|vs": "calc.cond.vsTargetBelow1HP",
    "char_187_ccheal|talent|first": "calc.cond.openingSeconds",
    "char_2015_dusk|talent|f": "calc.cond.talentStacks",
    "char_248_mgllan|trait|f": "calc.cond.multipleTargets",
    "char_249_mlyss|talent|no": "calc.cond.noClones",
    "char_250_phatom|talent|with": "calc.cond.withClone",
    "char_252_bibeak|talent|w": "calc.cond.withoutTalent",
    "char_286_cast3|talent|first": "calc.cond.openingSeconds",
    "char_300_phenxi|talent|w": "calc.cond.withoutVigor",
    "char_333_sidero|talent|After": "calc.cond.afterTheTalentSKills",
    "char_340_shwaz|talent2|w": "calc.cond.withoutASecondSniper",
    "char_4000_jnight|talent|first": "calc.cond.openingSecondsVsDrones",
    "char_4011_lessng|talent2|w": "calc.cond.withoutSecondTalent",
    "char_4016_kazema|skill|w": "calc.cond.withoutDoll",
    "char_4026_vulpis|talent|w": "calc.cond.withoutFirstTalent",
    "char_4040_rockr|talent|w": "calc.cond.withoutTalent",
    "char_4056_titi|talent2|f": "calc.cond.highHPTarget",
    "char_4071_peper|talent|f": "calc.cond.lowHPTarget",
    "char_4081_warmy|trait|no": "calc.cond.noBurn",
    "char_4082_qiubai|talent|vsSlow": "calc.cond.vsSlowWithSelfAppliedBind",
    "char_4082_qiubai|talent|w": "calc.cond.withoutFirstTalent",
    "char_4088_hodrer|talent|vsStun": "calc.cond.vsStunOrBind",
    "char_4117_ray|talent2|After": "calc.cond.afterTheTalentSHits",
    "char_4117_ray|talent|with": "calc.cond.withPet",
    "char_4146_nymph|talent2|f": "calc.cond.talentStacks",
    "char_4164_tecno|talent|f": "calc.cond.talentSummons",
    "char_4182_oblvns|skill|f": "calc.cond.fever",
    "char_4193_lemuen|talent2|f": "calc.cond.beforeTheTalentSTimeLimit",
    "char_4196_reckpr|module|vs": "calc.cond.vsTargetAbove50HP",
    "char_430_fartth|talent|w": "calc.cond.withoutTalent",
    "char_485_pallas|trait|w": "calc.cond.withoutTrait",
};

/** The label key for one operator's conditional, or null when the name is not in the tables. */
export function conditionalLabelKey(charId: string, cond: Pick<IConditionalInfo, "conditionalType" | "name">): ConditionalLabelKey | null {
    return CONDITIONAL_OVERRIDE_KEYS[`${charId}|${cond.conditionalType}|${cond.name}`] ?? (Object.hasOwn(CONDITIONAL_NAME_KEYS, cond.name) ? CONDITIONAL_NAME_KEYS[cond.name] : null);
}

/** Display label: the translated label when the name is known, else {@link humaniseConditionalName}. */
export function conditionalLabel(t: ConditionalT, charId: string, cond: Pick<IConditionalInfo, "conditionalType" | "name">): string {
    const key = conditionalLabelKey(charId, cond);
    return key ? t(key) : humaniseConditionalName(cond.name);
}

/**
 * Readable fallback for a name the tables do not know yet (a new operator's config):
 * splits camelCase and digit runs, keeps all-caps tokens (`DP`, `IS3`) as written,
 * lowercases the rest, and capitalises the first word. `vsBlockedTarget` -> "vs. blocked target",
 * `noTalent2Stacks` -> "No talent 2 stacks". English only: an unknown name has no translation.
 */
export function humaniseConditionalName(name: string): string {
    const words = name
        .trim()
        .split(/[\s_-]+|(?<=[a-z])(?=[A-Z0-9])|(?<=[0-9])(?=[A-Za-z])|(?<=[A-Z])(?=[A-Z][a-z])/)
        .filter((w) => w.length > 0)
        .map((w) => (w.length > 1 && w === w.toUpperCase() ? w : w.toLowerCase()));
    if (words.length === 0) return name;
    if (words[0] === "vs") return ["vs.", ...words.slice(1)].join(" ");
    words[0] = words[0].charAt(0).toUpperCase() + words[0].slice(1);
    return words.join(" ");
}

/**
 * Whether a conditional's switch shows on. Its label describes the switch ON, but for an
 * `inverted` conditional the label sits under `if not self.<flag>` in the Python reference
 * (Thorns `rangedAtk`): the label holds while the request flag is FALSE. The backend marks
 * 146 of 389 conditionals inverted (`backend/scripts/dps_conditional_polarity.py`), which
 * supersedes the earlier 116 census. A flag left unset reads as on, as the engine reads it.
 */
export function conditionalSwitchOn(cond: Pick<IConditionalInfo, "inverted">, flag: boolean | undefined): boolean {
    const value = flag ?? true;
    return cond.inverted === true ? !value : value;
}

/** The request flag a switch position stands for: {@link conditionalSwitchOn} inverted. */
export function conditionalFlagFor(cond: Pick<IConditionalInfo, "inverted">, switchOn: boolean): boolean {
    return cond.inverted === true ? !switchOn : switchOn;
}
