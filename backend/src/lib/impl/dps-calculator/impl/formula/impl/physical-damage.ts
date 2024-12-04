import Stats from "../../classes/stats";

export const physicalDamage = (finalAttack: number, stats: Stats, flatDefModifiers?: number[], scalingDefModifiers?: number[], physTakenModifiers?: number[]): number => {
    const minimumPhysicalDamage = (finalAttack * 5) / 100;

    let physicalDamage = minimumPhysicalDamage;
    let defTotal = 0;
    let flatDefTotal = 0;

    if (flatDefModifiers) {
        for (const flatDef of flatDefModifiers) {
            flatDefTotal += flatDef;
        }
    }

    defTotal = stats.def + flatDefTotal;

    if (scalingDefModifiers) {
        for (const scaleDef of scalingDefModifiers) {
            defTotal = defTotal * (1 + scaleDef);
        }
    }

    physicalDamage = finalAttack - defTotal;

    if (physTakenModifiers != null) {
        for (const taken of physTakenModifiers) {
            physicalDamage = physicalDamage * (1 + taken);
        }
    }

    if (physicalDamage < minimumPhysicalDamage) {
        physicalDamage = minimumPhysicalDamage;
    }

    return physicalDamage;
};
