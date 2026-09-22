import { asset } from "#/components/operators/detail/impl/assets";
import type { IRosterEntry } from "#/lib/api/user";
import { DEFAULT_LOCALE, formatMessage, sourceMessage } from "#/lib/i18n";
import { fullMessageKey, type TypedT } from "#/lib/i18n/messages";
import { getAvatarById, lerpByLevel } from "#/lib/utils";
import type { IAttributeKeyFrame, IModule, IOperatorListItem, IPotentialRank } from "#/types/operators";
import type { messages as cardMessages } from "./helpers.card.messages";
import type { IOwnedEntry } from "./types";

/** The `t` these labels need, narrowed to the keys they can render. */
export type CardT = TypedT<typeof cardMessages>;

/**
 * Default `t` for a caller outside an `I18nProvider`. It resolves against the
 * bundled source catalog, so the English is the same one the components render
 * and this file carries no second copy of the text.
 */
const sourceT: CardT = (key, values) => formatMessage(sourceMessage(fullMessageKey("user", key)) ?? key, DEFAULT_LOCALE, values);

export { moduleIconURL, skillIconURL, specializedIcon } from "#/components/operators/detail/impl/assets";
export { parseOperatorName } from "#/lib/utils";

const MAX_FAVOR_POINT = 25570;

export function ownedHeroURL(entry: IOwnedEntry): string {
    if (entry.skin_id?.includes("@")) {
        const file = entry.skin_id.replaceAll("@", "_").replaceAll("#", "%23");
        return asset(`/textures/skinpack/${entry.operator_id}/${file}.png`);
    }
    const tmplMatch = entry.skin_id?.match(/#(\d+)$/);
    const suffix = tmplMatch ? `_${tmplMatch[1]}` : entry.elite >= 2 ? "_2" : "_1";
    return asset(`/textures/chararts/${entry.operator_id}/${entry.operator_id}${suffix}.png`);
}

export function rarityIcon(starCount: number): string {
    return asset(`/textures/arts/rarity_hub/rarity_yellow_${starCount - 1}.png`);
}

/**
 * Short label for the module-stage badge on operator cards.
 * Special cases: ISW -> IS, RA -> RA (Reclamation Algorithm),
 * typeName2 "D" -> Δ (Delta module). Otherwise uses typeName2
 * (the X/Y/Z designator) and falls back to typeName1.
 */
export function moduleBadgeLetter(mod: Pick<IModule, "typeName1" | "typeName2">): string {
    const t1 = mod.typeName1?.toUpperCase();
    if (t1 === "ISW") return "IS";
    if (t1 === "RA") return "RA";
    if (mod.typeName2 === "D") return "Δ";
    return mod.typeName2 ?? mod.typeName1 ?? "X";
}

/**
 * Full module-type label for dialogs/lists where horizontal space allows,
 * e.g. "CCR-X" / "CCR-Δ". Combines typeName1 with the X/Y/Δ designator so two
 * modules of the same group stay distinguishable (unlike typeName1 alone).
 * Falls back to the single badge letter when there is no second designator
 * (ISW/RA modules, which carry no X/Y suffix).
 */
export function moduleTypeLabel(mod: Pick<IModule, "typeName1" | "typeName2">): string {
    const letter = moduleBadgeLetter(mod);
    if (!mod.typeName1 || !mod.typeName2) return letter;
    return `${mod.typeName1}-${letter}`;
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
        // 4★+ operators at E2 should have at least one mastery row recorded
        // (even at mastery=0). An empty array means the data is missing/never
        // synced - treat that as not-maxed instead of vacuously true.
        if (entry.rarity >= 4 && entry.masteries.length === 0) return false;
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

// Operator completeness, client-side port of the backend grader
// (`backend/src/core/grade/grade_operators.rs`).

const WEIGHT_ELITE = 25;
const WEIGHT_MASTERY = 30;
const WEIGHT_MODULE = 25;
const WEIGHT_POTENTIAL = 10;
const WEIGHT_SKILL_LEVEL = 20;
const WEIGHT_TRUST = 5;

/** The level at which a skill (M3) or module (Mod3) counts as a milestone. */
const MILESTONE = 3;
const PARTIAL_CAP = 0.3;
const PARTIAL_BONUS = 0.1;
const TRUST_MILESTONE_PCT = 100;

function clamp01(n: number): number {
    return Math.max(0, Math.min(1, n));
}

/**
 * Level-dimension weight, inverted by rarity (`level_weight`): low-rarity ops
 * have fewer investment axes, so leveling dominates their score.
 */
function levelWeight(rarity: number): number {
    switch (rarity) {
        case 6:
            return 15;
        case 5:
            return 18;
        case 4:
            return 20;
        default:
            return 40;
    }
}

/** ln(1+t)/ln(2) - log compression of a 0..1 ratio (`log_curve_ratio`). */
function logCurveRatio(t: number): number {
    return Math.log(1 + t) / Math.LN2;
}

/** Cumulative level progress across all elite phases, log-compressed. */
function cumulativeLevelProgress(entry: IRosterEntry, op: IOperatorListItem): number {
    let progress = 0;
    let total = 0;
    (op.phases ?? []).forEach((phase, i) => {
        const maxLvl = phase.maxLevel;
        total += maxLvl;
        if (i < entry.elite) progress += maxLvl;
        else if (i === entry.elite) progress += entry.level;
    });
    if (total === 0) return 1;
    return logCurveRatio(progress / total);
}

/**
 * Fraction of the ladder climbed by the entries still below `MILESTONE`, over
 * `slots` slots of `MILESTONE` steps each; 0 with no slots.
 */
function subMilestoneProgress(levels: number[], slots: number): number {
    const max = slots * MILESTONE;
    if (max <= 0) return 0;
    const climbed = levels.filter((l) => l < MILESTONE).reduce((s, l) => s + l, 0);
    return climbed / max;
}

/**
 * The milestone curve shared by masteries and modules (`milestone_score`):
 * partial credit capped at `PARTIAL_CAP` until the first milestone, then the
 * dimension's ladder position plus up to `PARTIAL_BONUS` from the rest.
 */
function milestoneScore(levels: number[], slots: number, ladder: (reached: number, slots: number) => number): number {
    const reached = levels.filter((l) => l >= MILESTONE).length;
    if (reached === 0) return subMilestoneProgress(levels, slots) * PARTIAL_CAP;
    const remaining = Math.max(0, slots - reached);
    return Math.min(ladder(reached, slots) + subMilestoneProgress(levels, remaining) * PARTIAL_BONUS, 1);
}

/** One M3 -> 0.5, two -> 0.75, every skill the operator has -> 1 (`mastery_ladder`). */
function masteryLadder(reached: number, slots: number): number {
    if (reached >= slots) return 1;
    return reached === 1 ? 0.5 : reached === 2 ? 0.75 : 1;
}

/** The share of advanced modules at Mod3, never below 0.5 for the first (`module_ladder`). */
function moduleLadder(reached: number, slots: number): number {
    return Math.max(reached / slots, 0.5);
}

/**
 * Highest potential value (0-indexed) the operator can reach - P6 == 5.
 *
 * Unlike the backend grader, which only scores potential for operators that
 * can't use the general potential token (`potential_matters`), we count it for
 * *every* operator: a P1 operator genuinely isn't "complete", and gating it off
 * made a fully-built P6 alter read below 100% while a P5 standard op read 100%.
 */
function maxPotential(op: IOperatorListItem): number {
    return op.potentialRanks?.length ?? 0;
}

/** Advanced (non-default) modules - the only ones that count (`advanced_modules`). */
function advancedModules(op: IOperatorListItem) {
    return (op.modules ?? []).filter((m) => m.type === "ADVANCED");
}

/** The roster's levels for the operator's advanced modules (`advanced_module_levels`). */
function advancedModuleLevels(entry: IRosterEntry, advanced: IModule[]): number[] {
    const advancedIds = new Set(advanced.map((m) => m.uniEquipId));
    return entry.modules.filter((m) => advancedIds.has(m.id)).map((m) => m.level);
}

type Dimension = [weight: number, score: number];

/**
 * Build the applicable (weight, score) dimensions for an operator - the same
 * shape as the backend `build_dimensions`.
 *
 * Differences vs. the server: the elite and level weights are this card's own
 * (the server's are 35 and 25/40); potential is counted for every operator
 * (see {@link maxPotential}); `is_support` isn't known client-side, so trust
 * always uses the ordinary 100% target; and trust % comes from the linear
 * {@link getTrustPercent} approximation rather than the favor table.
 */
function operatorDimensions(entry: IRosterEntry, op: IOperatorListItem, rarity: number): Dimension[] {
    const phases = op.phases ?? [];
    const maxElite = phases.length - 1;
    const numSkills = op.skills?.length ?? 0;
    const canMaster = numSkills > 0 && phases.length >= 3;
    const advanced = advancedModules(op);

    const dims: Dimension[] = [];

    if (maxElite > 0) dims.push([WEIGHT_ELITE, entry.elite / maxElite]);

    dims.push([levelWeight(rarity), cumulativeLevelProgress(entry, op)]);

    if (!canMaster && numSkills > 0) {
        dims.push([WEIGHT_SKILL_LEVEL, (entry.skill_level - 1) / 6]); // SL1=0 … SL7=1
    }

    if (canMaster) {
        const masteryLevels = entry.masteries.map((m) => m.mastery);
        dims.push([WEIGHT_MASTERY, milestoneScore(masteryLevels, numSkills, masteryLadder)]);
    }

    if (advanced.length > 0) {
        // One weight per advanced module slot (`module_weight`): every module
        // of a rarity costs the same, so a two-slot 6★ earns as much per Mod3
        // as a one-slot one.
        dims.push([WEIGHT_MODULE * advanced.length, milestoneScore(advancedModuleLevels(entry, advanced), advanced.length, moduleLadder)]);
    }

    const maxPot = maxPotential(op);
    if (maxPot > 0) {
        dims.push([WEIGHT_POTENTIAL, clamp01(entry.potential / maxPot)]); // P1=0 … P6=1
    }

    dims.push([WEIGHT_TRUST, clamp01(getTrustPercent(entry.favor_point) / TRUST_MILESTONE_PCT)]);

    return dims;
}

/**
 * How "complete" an owned operator is, as a 0..1 fraction - the weight-normalized
 * average of its applicable investment dimensions. Follows the backend's
 * `grade_operator` (same milestone curves and log-compressed leveling; see
 * {@link operatorDimensions} for where the weights differ). Dimensions that
 * don't apply are skipped (a 3★ has no E2/mastery/module), so a fully-built
 * low-rarity op still reaches 1.0.
 */
export function operatorCompleteness(entry: IRosterEntry, op: IOperatorListItem, rarity: number): number {
    const dims = operatorDimensions(entry, op, rarity);
    const totalWeight = dims.reduce((s, [w]) => s + w, 0);
    if (totalWeight <= 0) return 0;
    return dims.reduce((s, [w, v]) => s + w * v, 0) / totalWeight;
}

export interface IOperatorGap {
    /** Stable tag (`ELITE`, `MAX_LEVEL`, `M3`, `SL7`, `MOD3`, `POT6`, `TRUST`). */
    tag: string;
    /** Brief human-readable label for the popover. */
    label: string;
}

/**
 * Everything still standing between this operator and 100% completeness - the
 * exact complement of {@link operatorCompleteness}, so an empty list ⟺ a full
 * score. Unlike the backend's milestone "improvements" list (which only surfaces
 * the *next* step), this reports every remaining sub-goal: e.g. how many skills
 * still need M3, not just "M3" once one skill is mastered.
 */
export function operatorMissing(entry: IRosterEntry, op: IOperatorListItem, t: CardT = sourceT): IOperatorGap[] {
    const phases = op.phases ?? [];
    const maxElite = Math.max(0, phases.length - 1);
    const maxLevelAtCurrentElite = phases[entry.elite]?.maxLevel ?? 0;
    const numSkills = op.skills?.length ?? 0;
    const canMaster = numSkills > 0 && phases.length >= 3;
    const advanced = advancedModules(op);

    const masteryLevels = entry.masteries.map((m) => m.mastery);
    const skillsToM3 = canMaster ? numSkills - masteryLevels.filter((m) => m >= MILESTONE).length : 0;

    const moduleLevels = advancedModuleLevels(entry, advanced);
    const modulesToL3 = advanced.length - moduleLevels.filter((l) => l >= MILESTONE).length;

    const maxPot = maxPotential(op);

    const missing: IOperatorGap[] = [];
    if (entry.elite < maxElite) missing.push({ tag: "ELITE", label: t("profile.roster.gap.elite", { elite: maxElite }) });
    if (maxLevelAtCurrentElite > 0 && entry.level < maxLevelAtCurrentElite) missing.push({ tag: "MAX_LEVEL", label: t("profile.roster.gap.level", { level: maxLevelAtCurrentElite }) });
    if (canMaster) {
        if (skillsToM3 > 0) missing.push({ tag: "M3", label: t("profile.roster.gap.mastery", { count: skillsToM3 }) });
    } else if (numSkills > 0 && entry.skill_level < 7) {
        missing.push({ tag: "SL7", label: t("profile.roster.gap.skillLevel") });
    }
    if (modulesToL3 > 0) missing.push({ tag: "MOD3", label: t("profile.roster.gap.module", { count: modulesToL3 }) });
    if (maxPot > 0 && entry.potential < maxPot) missing.push({ tag: "POT6", label: t("profile.roster.gap.potential") });
    if (getTrustPercent(entry.favor_point) < TRUST_MILESTONE_PCT) missing.push({ tag: "TRUST", label: t("profile.roster.gap.trust") });

    return missing;
}

/**
 * How expensive an operator is to fully build, by rarity. Mirrors the backend's
 * `rarity_to_weight_in` (grade_operators.rs): the cost of a full build relative
 * to a 6★ from the 2026-09-22 game-table census (5★ 0.44 to 0.54, 4★ 0.26 to
 * 0.31, 3★ 0.03 to 0.06 depending on how the non-farmable tokens are valued).
 * Used to rank "most invested", since completeness alone treats a finished 3★
 * the same as a finished 6★.
 */
export const RARITY_WEIGHT: Record<number, number> = {
    6: 1.0,
    5: 0.5,
    4: 0.3,
    3: 0.05,
    2: 0.02,
    1: 0.01,
};

export interface IDerivedStats {
    maxHp: number;
    atk: number;
    def: number;
    magicResistance: number;
    cost: number;
    blockCnt: number;
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

const NO_MODULE_BONUS: IModuleBonus = { maxHp: 0, atk: 0, def: 0, magicResistance: 0, cost: 0, attackSpeed: 0, blockCnt: 0 };

function statsFromModule(op: IOperatorListItem, currentEquip: string | null, rosterModules: IRosterEntry["modules"]): IModuleBonus {
    if (!currentEquip) return NO_MODULE_BONUS;
    const equipped = op.modules.find((m) => m.uniEquipId === currentEquip);
    if (!equipped?.data?.phases) return NO_MODULE_BONUS;
    const rosterMod = rosterModules.find((m) => m.id === currentEquip);
    const lvl = rosterMod?.level ?? 0;
    if (lvl <= 0) return NO_MODULE_BONUS;
    const phase = equipped.data.phases[lvl - 1];
    if (!phase?.attributeBlackboard) return NO_MODULE_BONUS;

    const out: IModuleBonus = { ...NO_MODULE_BONUS };
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
    const mod = entry.elite === 2 ? statsFromModule(op, entry.current_equip, entry.modules) : NO_MODULE_BONUS;

    return {
        maxHp: lerpByLevel(entry.level, maxLevel, first.maxHp, last.maxHp) + trust.maxHp + pot.health + mod.maxHp,
        atk: lerpByLevel(entry.level, maxLevel, first.atk, last.atk) + trust.atk + pot.attackPower + mod.atk,
        def: lerpByLevel(entry.level, maxLevel, first.def, last.def) + trust.def + pot.defense + mod.def,
        magicResistance: lerpByLevel(entry.level, maxLevel, first.magicResistance, last.magicResistance) + pot.artsResistance + mod.magicResistance,
        cost: first.cost + pot.dpCost + mod.cost,
        blockCnt: first.blockCnt + pot.blockCnt + mod.blockCnt,
    };
}
