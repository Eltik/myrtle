import { lerpByLevel } from "#/lib/utils";
import type { IAttributeData, IBlackboard, IDrone, IOperatorListItem, IOperatorModule, IOperatorPhase, ISkillLevel, ITalent, ITalentCandidate, IUnlockCondition } from "#/types/operators";

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

export function formatAttributeKey(key: string): string {
    const map: Record<string, string> = {
        atk: "ATK",
        max_hp: "HP",
        def: "DEF",
        attack_speed: "ASPD",
        magic_resistance: "RES",
        cost: "DP Cost",
        respawn_time: "Redeploy",
        block_cnt: "Block",
        hp_recovery_per_sec: "HP Regen",
        sp_recovery_per_sec: "SP Regen",
        base_attack_time: "Attack Interval",
    };
    return map[key.toLowerCase()] ?? key.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
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

export function formatSkillLevel(idx: number): string {
    if (idx < 7) return `Lv.${idx + 1}`;
    return `M${idx - 6}`;
}

export function getSpTypeLabel(spType: string | number): string {
    const map: Record<string, string> = {
        INCREASE_WITH_TIME: "Auto Recovery",
        "1": "Auto Recovery",
        INCREASE_WHEN_ATTACK: "Offensive Recovery",
        "2": "Offensive Recovery",
        INCREASE_WHEN_TAKEN_DAMAGE: "Defensive Recovery",
        "4": "Defensive Recovery",
        ON_DEPLOYMENT: "On Deployment",
        "8": "On Deployment",
    };
    return map[String(spType)] ?? String(spType);
}

export function getSkillTypeLabel(skillType: string | number): string {
    const map: Record<string, string> = {
        PASSIVE: "Passive",
        "0": "Passive",
        MANUAL: "Manual Trigger",
        "1": "Manual Trigger",
        AUTO: "Auto Trigger",
        "2": "Auto Trigger",
    };
    return map[String(skillType)] ?? String(skillType);
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
