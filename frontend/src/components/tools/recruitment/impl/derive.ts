import type { RecruitmentOperator } from "#/types/generated/RecruitmentOperator";
import { HIDDEN_TAG_NAMES, MELEE_TAG_ID, PROFESSION_LABELS, PROFESSION_TAG_ID, RANGED_TAG_ID, ROBOT_TAG_ID, SENIOR_OPERATOR_TAG_ID, TOP_OPERATOR_TAG_ID } from "./constants";
import type { IRecruitableOperator, IRecruitPotential } from "./types";

/** One operator as `/operators/recruitment` serves it. Generated from the Rust struct. */
export type IRecruitmentSourceOperator = RecruitmentOperator;

/** The recruitment view of one operator, computed once on the server. */
export function toRecruitableOperator(op: IRecruitmentSourceOperator, tagNames?: ReadonlyMap<number, string>): IRecruitableOperator {
    return {
        id: op.id,
        name: op.name,
        rarity: op.rarity,
        profession: op.profession,
        position: op.position,
        professionName: professionTagName(op.profession, tagNames),
        tagList: buildOperatorTagList(op, op.rarity, tagNames),
        potentials: buildPotentials(op),
    };
}

/**
 * A class's name in the server's own wording, read from its recruitment tag list.
 *
 * Every client ships the 8 class names as recruitment tags under fixed ids (`PROFESSION_TAG_ID`), so this names a class
 * in Korean on KR and Japanese on JP without a translation table, and a server added later needs nothing. The English
 * label is the fallback for a list without that id; an unknown profession code comes back as itself.
 */
export function professionTagName(profession: string, tagNames?: ReadonlyMap<number, string>): string {
    const id = PROFESSION_TAG_ID[profession];
    return (id !== undefined ? tagNames?.get(id) : undefined) ?? PROFESSION_LABELS[profession] ?? profession;
}

/**
 * Position, class and rarity qualification tags first, then the game's own affix tags, minus the ones the tool hides.
 * The synthetic tags are named by the server's gacha tag list (`tagNames`, by id) so a KR or JP client shows its own
 * wording; the English literal is only the fallback when the list is missing an id.
 */
export function buildOperatorTagList(op: Pick<IRecruitmentSourceOperator, "position" | "profession" | "tagList">, rarity: number, tagNames?: ReadonlyMap<number, string>): string[] {
    const name = (id: number, fallback: string): string => tagNames?.get(id) ?? fallback;
    const tags: string[] = [];
    if (op.position === "MELEE") tags.push(name(MELEE_TAG_ID, "Melee"));
    if (op.position === "RANGED") tags.push(name(RANGED_TAG_ID, "Ranged"));

    if (PROFESSION_LABELS[op.profession]) tags.push(professionTagName(op.profession, tagNames));

    if (rarity === 6) tags.push(name(TOP_OPERATOR_TAG_ID, "Top Operator"));
    if (rarity === 5) tags.push(name(SENIOR_OPERATOR_TAG_ID, "Senior Operator"));
    if (rarity === 1) tags.push(name(ROBOT_TAG_ID, "Robot"));

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
