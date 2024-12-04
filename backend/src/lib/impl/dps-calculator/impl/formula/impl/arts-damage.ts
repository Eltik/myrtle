import Stats from "../../classes/stats";

export const artsDamage = (finalAttack: number, stats: Stats, flatResModifiers?: number[], scalingResModifiers?: number[], artsTakenModifiers?: number[]): number => {
    let artsDamage = 0;
    let resistTotal = 0;
    let flatResTotal = 0;

    if (flatResModifiers) {
        for (const flatRes of flatResModifiers) {
            flatResTotal += flatRes;
        }
    }

    resistTotal = stats.res + flatResTotal;

    if (scalingResModifiers) {
        for (const scaleRes of scalingResModifiers) {
            resistTotal = resistTotal * (1 + scaleRes);
        }
    }

    resistTotal = 1 - resistTotal / 100;

    artsDamage = finalAttack * resistTotal;

    if (artsTakenModifiers) {
        for (const taken of artsTakenModifiers) {
            artsDamage = artsDamage * (1 + taken);
        }
    }

    return artsDamage;
};
