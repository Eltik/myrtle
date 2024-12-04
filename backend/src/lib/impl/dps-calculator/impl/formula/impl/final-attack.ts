import Stats from "../../classes/stats";

export const finalAttack = (stats: Stats, baseAttackModifiers?: number[], attackMultiplierModifiers?: number[], soraBuff?: number): number => {
    let finalAttack = 0;
    let baseATKTotal = 0;

    if (baseAttackModifiers) {
        for (const baseAtk of baseAttackModifiers) {
            baseATKTotal += baseAtk;
        }
    }

    finalAttack = (stats.atk ?? 1) * (1 + baseATKTotal) + (soraBuff ?? 0);

    if (attackMultiplierModifiers) {
        for (const mulitplier of attackMultiplierModifiers) {
            finalAttack *= mulitplier;
        }
    }

    return finalAttack;
};
