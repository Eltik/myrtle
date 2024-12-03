import type { Operator } from "~/types/impl/api/static/operator";

export function applyPotential(charData: Operator, rank: number, basic: Record<string, number>) {
    const PotentialAttributeTypeList = {
        0: "maxHp",
        1: "atk",
        2: "def",
        3: "magicResistance",
        4: "cost",
        5: "blockCnt",
        6: "moveSpeed",
        7: "attackSpeed",
        21: "respawnTime",
    };

    if (!charData.potentialRanks || charData.potentialRanks.length == 0) return;
    for (let i = 0; i < rank; i++) {
        const potentialData = charData.potentialRanks[i];
        if (!potentialData?.buff) continue;
        const y = potentialData.buff.attributes.attributeModifiers?.[0];
        const key = PotentialAttributeTypeList[y?.attributeType as unknown as keyof typeof PotentialAttributeTypeList];
        const value = y?.value ?? 0;

        Object.assign(basic, {
            [key]: (basic[key] ?? 0) + value,
        });
    }
}
