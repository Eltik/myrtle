import { isOperatorEntity } from "#/lib/api/tier-entities";
import type { ITierEntryFull, ITierOperator } from "#/lib/api/tier-lists";
import type { OperatorPosition, OperatorProfession, OperatorRarity } from "#/types/operators";
import { operatorPlacementNote } from "../shared";

export interface ITierStats {
    /** Every placement, of any kind. */
    total: number;
    /** The operators among them, which every breakdown below reads. */
    operatorCount: number;
    rarity: { rarity: OperatorRarity; count: number }[];
    profession: { profession: OperatorProfession; count: number }[];
    position: { melee: number; ranged: number; other: number };
    averageRarity: number | null;
    describedCount: number;
    lastUpdatedAt: string | null;
    topOperator: ITierOperator | null;
}

const PROFESSION_ORDER: OperatorProfession[] = ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "MEDIC", "SUPPORT", "SPECIAL", "TOKEN", "TRAP"];

/** Counts every placement; the rarity, class and position breakdowns read the operators among them. */
export function computeTierStats(tier: ITierEntryFull): ITierStats {
    const entities = tier.entities;
    const ops = entities.filter(isOperatorEntity);
    const total = entities.length;

    const rarityCounts: Partial<Record<OperatorRarity, number>> = {};
    const professionCounts: Partial<Record<OperatorProfession, number>> = {};
    const position = { melee: 0, ranged: 0, other: 0 };
    let raritySum = 0;
    let describedCount = 0;
    let lastUpdatedAt: string | null = null;

    for (const op of ops) {
        rarityCounts[op.rarity] = (rarityCounts[op.rarity] ?? 0) + 1;
        professionCounts[op.profession] = (professionCounts[op.profession] ?? 0) + 1;
        bumpPosition(position, op.position);
        raritySum += op.rarity;
    }
    for (const entity of entities) {
        if (operatorPlacementNote(entity)) describedCount += 1;
        if (entity.updatedAt) {
            if (lastUpdatedAt === null || entity.updatedAt > lastUpdatedAt) lastUpdatedAt = entity.updatedAt;
        }
    }

    const rarity = ([6, 5, 4, 3, 2, 1] as OperatorRarity[]).filter((r) => (rarityCounts[r] ?? 0) > 0).map((r) => ({ rarity: r, count: rarityCounts[r] ?? 0 }));

    const profession = PROFESSION_ORDER.filter((p) => (professionCounts[p] ?? 0) > 0).map((p) => ({ profession: p, count: professionCounts[p] ?? 0 }));

    const averageRarity = ops.length > 0 ? raritySum / ops.length : null;
    const topOperator = ops.length > 0 ? [...ops].sort((a, b) => b.rarity - a.rarity || a.subOrder - b.subOrder)[0] : null;

    return { total, operatorCount: ops.length, rarity, profession, position, averageRarity, describedCount, lastUpdatedAt, topOperator };
}

function bumpPosition(position: { melee: number; ranged: number; other: number }, p: OperatorPosition) {
    if (p === "MELEE") position.melee += 1;
    else if (p === "RANGED") position.ranged += 1;
    else position.other += 1;
}
