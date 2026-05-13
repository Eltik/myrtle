import type { ITierEntryFull, ITierOperator } from "#/lib/api/tier-lists";
import type { OperatorPosition, OperatorProfession, OperatorRarity } from "#/types/operators";

export { formatRelative } from "#/lib/utils";

export interface ITierStats {
    total: number;
    rarity: { rarity: OperatorRarity; count: number }[];
    profession: { profession: OperatorProfession; count: number }[];
    position: { melee: number; ranged: number; other: number };
    averageRarity: number | null;
    notesCount: number;
    lastUpdatedAt: string | null;
    topOperator: ITierOperator | null;
}

const PROFESSION_ORDER: OperatorProfession[] = ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "MEDIC", "SUPPORT", "SPECIAL", "TOKEN", "TRAP"];

export function computeTierStats(tier: ITierEntryFull): ITierStats {
    const ops = tier.operators;
    const total = ops.length;

    const rarityCounts: Partial<Record<OperatorRarity, number>> = {};
    const professionCounts: Partial<Record<OperatorProfession, number>> = {};
    const position = { melee: 0, ranged: 0, other: 0 };
    let raritySum = 0;
    let notesCount = 0;
    let lastUpdatedAt: string | null = null;

    for (const op of ops) {
        rarityCounts[op.rarity] = (rarityCounts[op.rarity] ?? 0) + 1;
        professionCounts[op.profession] = (professionCounts[op.profession] ?? 0) + 1;
        bumpPosition(position, op.position);
        raritySum += op.rarity;
        if (op.notes?.trim()) notesCount += 1;
        if (op.updatedAt) {
            if (lastUpdatedAt === null || op.updatedAt > lastUpdatedAt) lastUpdatedAt = op.updatedAt;
        }
    }

    const rarity = ([6, 5, 4, 3, 2, 1] as OperatorRarity[]).filter((r) => (rarityCounts[r] ?? 0) > 0).map((r) => ({ rarity: r, count: rarityCounts[r] ?? 0 }));

    const profession = PROFESSION_ORDER.filter((p) => (professionCounts[p] ?? 0) > 0).map((p) => ({ profession: p, count: professionCounts[p] ?? 0 }));

    const averageRarity = total > 0 ? raritySum / total : null;
    const topOperator = ops.length > 0 ? [...ops].sort((a, b) => b.rarity - a.rarity || a.subOrder - b.subOrder)[0] : null;

    return { total, rarity, profession, position, averageRarity, notesCount, lastUpdatedAt, topOperator };
}

function bumpPosition(position: { melee: number; ranged: number; other: number }, p: OperatorPosition) {
    if (p === "MELEE") position.melee += 1;
    else if (p === "RANGED") position.ranged += 1;
    else position.other += 1;
}
