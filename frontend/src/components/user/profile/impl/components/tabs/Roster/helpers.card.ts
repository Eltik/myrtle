import { asset } from "#/components/operators/detail/impl/assets";
import type { IRosterEntry } from "#/lib/api/user";
import { getAvatarById } from "#/lib/utils";
import type { IAttributeKeyFrame, IEnrichedSkill, IModule, IOperatorListItem, IPotentialRank } from "#/types/operators";
import type { IOwnedEntry } from "./types";

export { parseOperatorName } from "#/lib/utils";

const MAX_FAVOR_POINT = 25570;

export function ownedHeroURL(entry: IOwnedEntry): string {
    if (entry.skin_id?.includes("@")) {
        const file = entry.skin_id.replaceAll("@", "_").replaceAll("#", "%23");
        return asset(`/textures/skinpack/${entry.operator_id}/${file}.png`);
    }
    const suffix = entry.elite >= 2 ? "_2" : "_1";
    return asset(`/textures/chararts/${entry.operator_id}/${entry.operator_id}${suffix}.png`);
}

export function rarityIcon(starCount: number): string {
    return asset(`/textures/arts/rarity_hub/rarity_yellow_${starCount - 1}.png`);
}

export function specializedIcon(level: number): string {
    return asset(`/textures/arts/specialized_hub/specialized_${level}.png`);
}

export function skillIconURL(skill: IEnrichedSkill): string {
    if (skill.static?.image) return asset(skill.static.image);
    const id = skill.static?.iconId ?? skill.static?.skillId ?? skill.skillId;
    return asset(`/textures/skill-icons/${id}.png`);
}

export function moduleIconURL(mod: IModule): string {
    if (mod.image) return asset(mod.image);
    return asset(`/textures/spritepack/ui_equip_big_img_hub_0/${mod.uniEquipIcon}.png`);
}

/** Skin-aware avatar. `getAvatarById` already normalizes `@`-skins and `#1` base IDs. */
export function ownedAvatar(opId: string, skinId: string | null): string {
    return getAvatarById(skinId ?? opId);
}

export const MAX_LEVEL_BY_RARITY: Record<number, number[]> = {
    6: [50, 80, 90],
    5: [50, 70, 80],
    4: [45, 60, 70],
    3: [40, 55, 55],
    2: [30, 30, 30],
    1: [30, 30, 30],
};

export const MAX_ELITE_BY_RARITY: Record<number, number> = {
    6: 2,
    5: 2,
    4: 2,
    3: 1,
    2: 0,
    1: 0,
};

export function isMaxed(entry: IOwnedEntry): boolean {
    const maxElite = MAX_ELITE_BY_RARITY[entry.rarity] ?? 2;
    const maxLevel = MAX_LEVEL_BY_RARITY[entry.rarity]?.[maxElite] ?? 90;

    if (entry.elite !== maxElite) return false;
    if (entry.level !== maxLevel) return false;
    if (entry.rarity > 2 && entry.skill_level !== 7) return false;

    if (maxElite === 2) {
        if (!entry.masteries.every((m) => m.mastery === 3)) return false;

        const unlockedModules = entry.modules.filter((m) => m.level > 0);
        if (unlockedModules.length > 0 && !unlockedModules.every((m) => m.level === 3)) return false;
    }

    if (entry.potential !== 5) return false;

    return true;
}

export function getTrustPercent(rawFavorPoint: number): number {
    return Math.min(200, Math.round((rawFavorPoint / MAX_FAVOR_POINT) * 200));
}

export interface IDerivedStats {
    maxHp: number;
    atk: number;
    def: number;
    magicResistance: number;
    cost: number;
    blockCnt: number;
}

function lerpByLevel(level: number, maxLevel: number, base: number, max: number): number {
    if (maxLevel === 1) return base;
    return Math.round(base + ((level - 1) * (max - base)) / (maxLevel - 1));
}

function statsAtTrust(frames: IAttributeKeyFrame[] | undefined, raw: number): { maxHp: number; atk: number; def: number } {
    const empty = { maxHp: 0, atk: 0, def: 0 };
    if (!frames || frames.length === 0) return empty;
    const max = frames[frames.length - 1]?.data;
    if (!max) return empty;
    const ratio = Math.max(0, Math.min(raw, MAX_FAVOR_POINT)) / MAX_FAVOR_POINT;
    return {
        maxHp: Math.round((max.maxHp ?? 0) * ratio),
        atk: Math.round((max.atk ?? 0) * ratio),
        def: Math.round((max.def ?? 0) * ratio),
    };
}

interface IPotentialBonus {
    health: number;
    attackPower: number;
    defense: number;
    artsResistance: number;
    dpCost: number;
    attackSpeed: number;
    blockCnt: number;
}

function statsAtPotential(ranks: IPotentialRank[] | undefined, potential: number): IPotentialBonus {
    const out: IPotentialBonus = { health: 0, attackPower: 0, defense: 0, artsResistance: 0, dpCost: 0, attackSpeed: 0, blockCnt: 0 };
    if (!ranks || potential === 0) return out;
    for (let p = 1; p <= potential; p++) {
        const mod = ranks[p - 1]?.buff?.attributes?.attributeModifiers?.[0];
        if (!mod) continue;
        switch (mod.attributeType) {
            case "MAX_HP":
                out.health += mod.value;
                break;
            case "ATK":
                out.attackPower += mod.value;
                break;
            case "DEF":
                out.defense += mod.value;
                break;
            case "MAGIC_RESISTANCE":
                out.artsResistance += mod.value;
                break;
            case "COST":
                out.dpCost += mod.value;
                break;
            case "ATTACK_SPEED":
                out.attackSpeed += mod.value;
                break;
            case "BLOCK_CNT":
                out.blockCnt += mod.value;
                break;
        }
    }
    return out;
}

interface IModuleBonus {
    maxHp: number;
    atk: number;
    def: number;
    magicResistance: number;
    cost: number;
    attackSpeed: number;
    blockCnt: number;
}

function statsFromModule(op: IOperatorListItem, currentEquip: string | null, rosterModules: IRosterEntry["modules"]): IModuleBonus {
    const empty: IModuleBonus = { maxHp: 0, atk: 0, def: 0, magicResistance: 0, cost: 0, attackSpeed: 0, blockCnt: 0 };
    if (!currentEquip) return empty;
    const equipped = op.modules.find((m) => m.uniEquipId === currentEquip);
    if (!equipped?.data?.phases) return empty;
    const rosterMod = rosterModules.find((m) => m.id === currentEquip);
    const lvl = rosterMod?.level ?? 0;
    if (lvl <= 0) return empty;
    const phase = equipped.data.phases[lvl - 1];
    if (!phase?.attributeBlackboard) return empty;

    const out: IModuleBonus = { ...empty };
    for (const attr of phase.attributeBlackboard) {
        switch (attr.key) {
            case "max_hp":
                out.maxHp += attr.value;
                break;
            case "atk":
                out.atk += attr.value;
                break;
            case "def":
                out.def += attr.value;
                break;
            case "magic_resistance":
                out.magicResistance += attr.value;
                break;
            case "cost":
                out.cost += attr.value;
                break;
            case "attack_speed":
                out.attackSpeed += attr.value;
                break;
            case "block_cnt":
                out.blockCnt += attr.value;
                break;
        }
    }
    return out;
}

export function getAttributeStats(entry: IOwnedEntry, op: IOperatorListItem): IDerivedStats | null {
    const phase = op.phases?.[entry.elite];
    const frames = phase?.attributesKeyFrames;
    if (!frames || frames.length === 0) return null;

    const first = frames[0]?.data;
    const last = frames[frames.length - 1]?.data;
    if (!first || !last) return null;

    const maxLevel = phase.maxLevel;
    const trust = statsAtTrust(op.favorKeyFrames, entry.favor_point);
    const pot = statsAtPotential(op.potentialRanks, entry.potential);
    const mod = entry.elite === 2 ? statsFromModule(op, entry.current_equip, entry.modules) : { maxHp: 0, atk: 0, def: 0, magicResistance: 0, cost: 0, attackSpeed: 0, blockCnt: 0 };

    return {
        maxHp: lerpByLevel(entry.level, maxLevel, first.maxHp, last.maxHp) + trust.maxHp + pot.health + mod.maxHp,
        atk: lerpByLevel(entry.level, maxLevel, first.atk, last.atk) + trust.atk + pot.attackPower + mod.atk,
        def: lerpByLevel(entry.level, maxLevel, first.def, last.def) + trust.def + pot.defense + mod.def,
        magicResistance: lerpByLevel(entry.level, maxLevel, first.magicResistance, last.magicResistance) + pot.artsResistance + mod.magicResistance,
        cost: first.cost + pot.dpCost + mod.cost,
        blockCnt: first.blockCnt + pot.blockCnt + mod.blockCnt,
    };
}
