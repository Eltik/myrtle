import type { RecruitmentOperator } from "#/types/generated/RecruitmentOperator";
import { HIDDEN_TAG_NAMES, PROFESSION_LABELS } from "./constants";
import type { IRecruitableOperator, IRecruitPotential } from "./types";

/** One operator as `/operators/recruitment` serves it. Generated from the Rust struct. */
export type IRecruitmentSourceOperator = RecruitmentOperator;

/** The recruitment view of one operator, computed once on the server. */
export function toRecruitableOperator(op: IRecruitmentSourceOperator): IRecruitableOperator {
    return {
        id: op.id,
        name: op.name,
        rarity: op.rarity,
        profession: op.profession,
        position: op.position,
        tagList: buildOperatorTagList(op, op.rarity),
        potentials: buildPotentials(op),
    };
}

/** Position, class and rarity qualification tags first, then the game's own affix tags, minus the ones the tool hides. */
export function buildOperatorTagList(op: Pick<IRecruitmentSourceOperator, "position" | "profession" | "tagList">, rarity: number): string[] {
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
export function buildPotentials(op: Partial<Pick<IRecruitmentSourceOperator, "potentialRanks" | "talentPotentialRanks">>): IRecruitPotential[] {
    const ranks = op.potentialRanks ?? [];
    const talents = op.talentPotentialRanks ?? [];
    return ranks.map((rank, i) => {
        const mod = rank.modifier;
        if (mod && typeof mod.value === "number") return { kind: "stat", attribute: mod.attributeType, value: mod.value };

        const unlockedRank = i + 1;
        const index = talents.findIndex((requiredRanks) => requiredRanks.includes(unlockedRank));
        if (index >= 0) return { kind: "talent", index, of: talents.length };

        return { kind: "text", text: rank.description ?? "" };
    });
}
