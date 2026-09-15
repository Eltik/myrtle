import { DEFAULT_LOCALE, formatMessage, sourceMessage } from "#/lib/i18n";
import { fullMessageKey, type TypedT } from "#/lib/i18n/messages";
import { lerpByLevel } from "#/lib/utils";
import type { IAttributeData, IBlackboard, IDrone, IOperatorListItem, IOperatorModule, IOperatorPhase, ISkillLevel, ITalent, ITalentCandidate, IUnlockCondition } from "#/types/operators";
import type { messages as helperMessages } from "./helpers.messages";

/** The `t` these label helpers need, narrowed to the keys they can render. */
export type HelperT = TypedT<typeof helperMessages>;

/**
 * Default `t` for a caller that is not inside an `I18nProvider` - today only
 * the planner dialog, which reads these labels from outside this feature. It
 * resolves against the bundled source catalog, so the English is the same one
 * the components render and this file carries no second copy of the text.
 */
const sourceT: HelperT = (key, values) => formatMessage(sourceMessage(fullMessageKey("operators", key)) ?? key, DEFAULT_LOCALE, values);

export function blackboardKeyMap(blackboard: IBlackboard[]): { key: string; value: number }[] {
    return (blackboard ?? []).filter((b) => b.key != null).map((b) => ({ key: b.key, value: b.value }));
}

export function combinedDescriptionBlackboard(operator: IOperatorListItem): { key: string; value: number }[] {
    const traitCandidate = operator.trait?.candidates?.[operator.trait.candidates.length - 1];
    const traitBlackboard = traitCandidate?.blackboard ?? [];
    if (traitBlackboard.length > 0) return blackboardKeyMap(traitBlackboard);

    const talentBlackboards: IBlackboard[] = [];
    for (const talent of operator.talents ?? []) {
        const candidate = talent.candidates?.[talent.candidates.length - 1];
        if (candidate?.blackboard) talentBlackboards.push(...candidate.blackboard);
    }
    return blackboardKeyMap(talentBlackboards);
}

interface IStatsContext {
    phaseIndex: number;
    favorPoint: number;
    potentialRank: number;
    moduleId: string;
    moduleLevel: number;
}

interface IStatBag {
    atk: number;
    maxHp: number;
    def: number;
    attackSpeed: number;
    magicResistance: number;
    cost: number;
    respawnTime: number;
    blockCnt: number;
}

const ZERO_STATS: IStatBag = { atk: 0, maxHp: 0, def: 0, attackSpeed: 0, magicResistance: 0, cost: 0, respawnTime: 0, blockCnt: 0 };

function moduleStats(mod: IOperatorModule, level: number): IStatBag {
    if (!mod.data?.phases || level <= 0) return { ...ZERO_STATS };
    const phase = mod.data.phases[level - 1];
    if (!phase?.attributeBlackboard) return { ...ZERO_STATS };
    const out = { ...ZERO_STATS };
    for (const a of phase.attributeBlackboard) {
        if (a.key in out) (out as unknown as Record<string, number>)[a.key] += a.value;
    }
    return out;
}

function trustStats(operator: IOperatorListItem, rawTrust: number) {
    const frames = operator.favorKeyFrames ?? [];
    if (frames.length === 0) return { maxHp: 0, atk: 0, def: 0, magicResistance: 0 };
    const trust = Math.min(100, rawTrust);
    const max = frames[frames.length - 1]?.data;
    return {
        maxHp: Math.round((trust * (max?.maxHp ?? 0)) / 100),
        atk: Math.round((trust * (max?.atk ?? 0)) / 100),
        def: Math.round((trust * (max?.def ?? 0)) / 100),
        magicResistance: Math.round((trust * (max?.magicResistance ?? 0)) / 100),
    };
}

function potStats(operator: IOperatorListItem, potential: number) {
    const result = {
        maxHp: 0,
        atk: 0,
        def: 0,
        magicResistance: 0,
        cost: 0,
        attackSpeed: 0,
        respawnTime: 0,
    };
    if (potential === 0) return result;

    for (let p = 1; p <= potential; p++) {
        const rank = operator.potentialRanks[p - 1];
        if (!rank?.buff) continue;
        const modifiers = rank.buff.attributes?.attributeModifiers ?? [];
        for (const m of modifiers) {
            const value = m.value ?? 0;
            switch (m.attributeType) {
                case "MAX_HP":
                    result.maxHp += value;
                    break;
                case "ATK":
                    result.atk += value;
                    break;
                case "DEF":
                    result.def += value;
                    break;
                case "MAGIC_RESISTANCE":
                    result.magicResistance += value;
                    break;
                case "COST":
                    result.cost += value;
                    break;
                case "ATTACK_SPEED":
                    result.attackSpeed += value;
                    break;
                case "RESPAWN_TIME":
                    result.respawnTime += value;
                    break;
            }
        }
    }
    return result;
}

export function getOperatorAttributeStats(operator: IOperatorListItem, ctx: IStatsContext, level: number): IAttributeData | null {
    const phase: IOperatorPhase | undefined = operator.phases[ctx.phaseIndex];
    if (!phase) return null;
    const frames = phase.attributesKeyFrames;
    if (!frames || frames.length === 0) return null;
    if (level < 1 || level > phase.maxLevel) return null;

    const start = frames[0].data;
    const end = frames[frames.length - 1].data;

    const trust = trustStats(operator, ctx.favorPoint);
    const pot = potStats(operator, ctx.potentialRank);
    const activeModule = ctx.phaseIndex === 2 && ctx.moduleId ? operator.modules.find((m) => m.uniEquipId === ctx.moduleId) : undefined;
    const mod = activeModule ? moduleStats(activeModule, ctx.moduleLevel) : { ...ZERO_STATS };

    const maxHp = lerpByLevel(level, phase.maxLevel, start.maxHp, end.maxHp) + trust.maxHp + pot.maxHp + mod.maxHp;
    const atk = lerpByLevel(level, phase.maxLevel, start.atk, end.atk) + trust.atk + pot.atk + mod.atk;
    const def = lerpByLevel(level, phase.maxLevel, start.def, end.def) + trust.def + pot.def + mod.def;
    const res = lerpByLevel(level, phase.maxLevel, start.magicResistance, end.magicResistance) + trust.magicResistance + pot.magicResistance + mod.magicResistance;

    const aspd = 100 + pot.attackSpeed + mod.attackSpeed;
    return {
        ...end,
        maxHp,
        atk,
        def,
        magicResistance: res,
        cost: start.cost + pot.cost + mod.cost,
        blockCnt: start.blockCnt + mod.blockCnt,
        respawnTime: start.respawnTime + pot.respawnTime + mod.respawnTime,
        baseAttackTime: start.baseAttackTime,
        attackSpeed: Math.round((start.baseAttackTime * 30) / (aspd / 100)) / 30,
    };
}

export function clampDronePhase(drone: IDrone, parentPhaseIndex: number): number {
    const last = Math.max(0, drone.phases.length - 1);
    return Math.max(0, Math.min(parentPhaseIndex, last));
}

export function getDroneAttributeStats(drone: IDrone, parentPhaseIndex: number, parentLevel: number): IAttributeData | null {
    if (!drone.phases || drone.phases.length === 0) return null;
    const phaseIndex = clampDronePhase(drone, parentPhaseIndex);
    const phase = drone.phases[phaseIndex];
    const frames = phase?.attributesKeyFrames;
    if (!phase || !frames || frames.length === 0) return null;

    const level = Math.max(1, Math.min(parentLevel, phase.maxLevel));
    const start = frames[0].data;
    const end = frames[frames.length - 1].data;

    const aspd = 100;
    return {
        ...end,
        maxHp: lerpByLevel(level, phase.maxLevel, start.maxHp, end.maxHp),
        atk: lerpByLevel(level, phase.maxLevel, start.atk, end.atk),
        def: lerpByLevel(level, phase.maxLevel, start.def, end.def),
        magicResistance: lerpByLevel(level, phase.maxLevel, start.magicResistance, end.magicResistance),
        cost: start.cost,
        blockCnt: start.blockCnt,
        respawnTime: start.respawnTime,
        baseAttackTime: start.baseAttackTime,
        attackSpeed: Math.round((start.baseAttackTime * 30) / (aspd / 100)) / 30,
    };
}

export function droneTalentBlackboard(drone: IDrone): { key: string; value: number }[] {
    const out: IBlackboard[] = [];
    for (const t of drone.talents ?? []) {
        const c = t.candidates?.[t.candidates.length - 1];
        if (c?.blackboard) out.push(...c.blackboard);
    }
    return blackboardKeyMap(out);
}

const ATTRIBUTE_MESSAGE_KEY: Record<string, keyof typeof helperMessages & string> = {
    atk: "attr.atk",
    max_hp: "attr.maxHp",
    def: "attr.def",
    attack_speed: "attr.attackSpeed",
    magic_resistance: "attr.magicResistance",
    cost: "attr.cost",
    respawn_time: "attr.respawnTime",
    block_cnt: "attr.blockCnt",
    hp_recovery_per_sec: "attr.hpRecovery",
    sp_recovery_per_sec: "attr.spRecovery",
    base_attack_time: "attr.baseAttackTime",
};

export function formatAttributeKey(key: string, t: HelperT = sourceT): string {
    const messageKey = ATTRIBUTE_MESSAGE_KEY[key.toLowerCase()];
    // An unmapped key is a raw game token with nothing in the catalog to look
    // it up by, so it is title-cased rather than translated.
    return messageKey ? t(messageKey) : key.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

export function formatStatValue(value: number): string {
    if (Math.abs(value) < 2 && value !== 0 && !Number.isInteger(value)) {
        const pct = Math.round(value * 100);
        return `${pct >= 0 ? "+" : ""}${pct}%`;
    }
    return `${value >= 0 ? "+" : ""}${value}`;
}

export function phaseToIndex(phase: IUnlockCondition["phase"]): number {
    if (phase === "PHASE_1") return 1;
    if (phase === "PHASE_2") return 2;
    return 0;
}

export function isUnlocked(uc: { phase: string; level: number } | undefined, requiredPot: number, phaseIndex: number, level: number, potentialRank: number): boolean {
    if (!uc) return false;
    const reqPhase = phaseToIndex(uc.phase as IUnlockCondition["phase"]);
    if (phaseIndex < reqPhase) return false;
    if (phaseIndex === reqPhase && level < uc.level) return false;
    if (potentialRank < requiredPot) return false;
    return true;
}

export function getActiveTalentCandidate(talent: ITalent, phaseIndex: number, level: number, potentialRank: number): ITalentCandidate | null {
    const candidates = talent.candidates ?? [];
    let chosen: ITalentCandidate | null = null;
    for (const c of candidates) {
        if (isUnlocked(c.unlockCondition, c.requiredPotentialRank, phaseIndex, level, potentialRank)) chosen = c;
    }
    return chosen;
}

export function formatSkillLevel(idx: number, t: HelperT = sourceT): string {
    if (idx < 7) return t("skill.level.lv", { level: idx + 1 });
    return t("skill.level.mastery", { level: idx - 6 });
}

const SP_TYPE_MESSAGE_KEY: Record<string, keyof typeof helperMessages & string> = {
    INCREASE_WITH_TIME: "skill.spType.autoRecovery",
    "1": "skill.spType.autoRecovery",
    INCREASE_WHEN_ATTACK: "skill.spType.offensiveRecovery",
    "2": "skill.spType.offensiveRecovery",
    INCREASE_WHEN_TAKEN_DAMAGE: "skill.spType.defensiveRecovery",
    "4": "skill.spType.defensiveRecovery",
    ON_DEPLOYMENT: "skill.spType.onDeployment",
    "8": "skill.spType.onDeployment",
};

export function getSpTypeLabel(spType: string | number, t: HelperT = sourceT): string {
    const messageKey = SP_TYPE_MESSAGE_KEY[String(spType)];
    return messageKey ? t(messageKey) : String(spType);
}

const SKILL_TYPE_MESSAGE_KEY: Record<string, keyof typeof helperMessages & string> = {
    PASSIVE: "skill.type.passive",
    "0": "skill.type.passive",
    MANUAL: "skill.type.manual",
    "1": "skill.type.manual",
    AUTO: "skill.type.auto",
    "2": "skill.type.auto",
};

export function getSkillTypeLabel(skillType: string | number, t: HelperT = sourceT): string {
    const messageKey = SKILL_TYPE_MESSAGE_KEY[String(skillType)];
    return messageKey ? t(messageKey) : String(skillType);
}

interface ISkillDiff {
    spCostChanged: boolean;
    initSpChanged: boolean;
    durationChanged: boolean;
    blackboardChanges: Map<string, { prev: number; curr: number }>;
}

export function computeSkillDiff(prev: ISkillLevel | null, curr: ISkillLevel): ISkillDiff {
    const blackboardChanges = new Map<string, { prev: number; curr: number }>();
    if (prev) {
        const prevBb = new Map((prev.blackboard ?? []).map((b) => [b.key.toLowerCase(), b.value]));
        for (const c of curr.blackboard ?? []) {
            const p = prevBb.get(c.key.toLowerCase());
            if (p !== undefined && p !== c.value) blackboardChanges.set(c.key.toLowerCase(), { prev: p, curr: c.value });
        }
    }
    return {
        spCostChanged: prev !== null && prev.spData?.spCost !== curr.spData?.spCost,
        initSpChanged: prev !== null && prev.spData?.initSp !== curr.spData?.initSp,
        durationChanged: prev !== null && prev.duration !== curr.duration,
        blackboardChanges,
    };
}

export function formatBlackboardValue(key: string, value: number): string {
    const percentageKeys = ["atk", "attack_speed", "def", "max_hp", "hp_recovery_per_sec", "sp_recovery_per_sec", "damage_scale", "atk_scale", "def_scale"];
    const isPct = percentageKeys.some((k) => key.toLowerCase().includes(k));
    if (isPct && Math.abs(value) < 10) return `${Math.round(value * 100)}%`;
    if (Number.isInteger(value)) return String(value);
    return value.toFixed(1);
}
