import Stats from "../../classes/stats";

export const attacksPerSecond = (stats: Stats, attackSpeedModifiers?: number[], attackIntervalModifiers?: number[]): number => {
    let speedModifierTotal = 0;
    let intervalModifierTotal = 0;

    if (attackSpeedModifiers) {
        for (const speed of attackSpeedModifiers) {
            speedModifierTotal += speed;
        }
    }

    if (attackIntervalModifiers) {
        for (const interval of attackIntervalModifiers) {
            intervalModifierTotal += interval;
        }
    }

    const aps = (1 + speedModifierTotal / 100) / ((stats.attackSpeed ?? 0) + intervalModifierTotal);
    return aps;
};
