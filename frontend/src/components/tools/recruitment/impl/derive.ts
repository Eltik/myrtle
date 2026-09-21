import { rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { HIDDEN_TAG_NAMES, PROFESSION_LABELS } from "./constants";
import type { IRecruitableOperator, IRecruitPotential } from "./types";

/** The recruitment view of one operator from the `/static/operators` map, computed once on the server. */
export function toRecruitableOperator(id: string, op: IOperatorListItem): IRecruitableOperator {
    const rarity = rarityToNumber(op.rarity);
    return {
        id,
        name: op.name,
        rarity,
        profession: op.profession,
        position: op.position,
        tagList: buildOperatorTagList(op, rarity),
        potentials: buildPotentials(op),
    };
}

/** Position, class and rarity qualification tags first, then the game's own affix tags, minus the ones the tool hides. */
export function buildOperatorTagList(op: IOperatorListItem, rarity: number): string[] {
    const tags: string[] = [];
    if (op.position === "MELEE") tags.push("Melee");
    if (op.position === "RANGED") tags.push("Ranged");

    const profTag = PROFESSION_LABELS[op.profession];
    if (profTag) tags.push(profTag);

    if (rarity === 6) tags.push("Top Operator");
    if (rarity === 5) tags.push("Senior Operator");
    if (rarity === 1) tags.push("Robot");

    if (op.tagList) tags.push(...op.tagList.filter((t) => !HIDDEN_TAG_NAMES.has(t)));

    return tags;
}

/**
 * One entry per potential rank, in rank order. A BUFF rank carries its first
 * attribute modifier; a CUSTOM rank names the talent whose candidate unlocks at
 * that rank. Measured on the EN table 2026-09-21: 408/408 CUSTOM ranks resolve
 * to exactly one talent this way, and every BUFF rank is one of COST,
 * RESPAWN_TIME, ATK, DEF, MAX_HP, MAGIC_RESISTANCE, ATTACK_SPEED. Anything
 * else keeps its description as text.
 */
export function buildPotentials(op: IOperatorListItem): IRecruitPotential[] {
    const ranks = op.potentialRanks ?? [];
    const talents = op.talents ?? [];
    return ranks.map((rank, i) => {
        const mod = rank.buff?.attributes?.attributeModifiers?.[0];
        if (mod && typeof mod.value === "number") return { kind: "stat", attribute: mod.attributeType, value: mod.value };

        const unlockedRank = i + 1;
        const index = talents.findIndex((talent) => talent.candidates?.some((c) => c.requiredPotentialRank === unlockedRank));
        if (index >= 0) return { kind: "talent", index, of: talents.length };

        return { kind: "text", text: rank.description ?? "" };
    });
}
