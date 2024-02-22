import { Drop } from "../../../../types/types";
import { stages } from "./before";

export const getPlan = async (requirements: Record<string, number>, ownedItems: Record<string, number>) => {
    // Initialize a plan object to store the results
    const plan: Record<string, { stageId: string, times: number }[]> = {};

    // Loop through each item in the requirements
    for (const itemId in requirements) {
        const requiredQuantity = requirements[itemId] - (ownedItems[itemId] ||  0);
        let remainingQuantity = requiredQuantity;

        // Filter stages that drop the current item and sort them by efficiency
        const relevantStages = stages
            .filter(stage => stage.drops.some(drop => drop.itemId === itemId))
            .sort((a, b) => {
                const efficiencyA = (a.drops.find(drop => drop.itemId === itemId)?.quantity ?? 0) / (a.drops.find(drop => drop.itemId === itemId)?.times ?? 0) || 0;
                const efficiencyB = (b.drops.find(drop => drop.itemId === itemId)?.quantity ?? 0) / (b.drops.find(drop => drop.itemId === itemId)?.times ?? 0) || 0;
                return efficiencyA - efficiencyB;
            });

        // Determine the number of times each stage needs to be played
        for (const stage of relevantStages) {
            const drop = stage.drops.find(d => d.itemId === itemId);
            if (!drop) continue;

            const efficiency = calculateEfficiency(drop);
            const timesToFarm = Math.ceil(remainingQuantity / (drop.quantity * efficiency));
            remainingQuantity -= timesToFarm * drop.quantity;

            if (!plan[itemId]) {
                plan[itemId] = [];
            }
            plan[itemId].push({ stageId: stage.stageId, times: timesToFarm });

            if (remainingQuantity <=  0) break;
        }
    }

    return plan;
};

const calculateEfficiency = (drop: Drop) => {
    return drop.quantity / drop.times;
};