import type { IAttributeData, IBlackboard, IOperatorListItem, IOperatorModule, IOperatorPhase } from "#/types/operators";

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

function lerpByLevel(level: number, maxLevel: number, base: number, max: number): number {
    if (maxLevel <= 1) return base;
    return Math.round(base + ((level - 1) * (max - base)) / (maxLevel - 1));
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
    const mod = ctx.phaseIndex === 2 && ctx.moduleId ? moduleStats(operator.modules.find((m) => m.uniEquipId === ctx.moduleId)!, ctx.moduleLevel) : { ...ZERO_STATS };

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
