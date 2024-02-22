import { stages } from "./before";

export const getPlan = async (requirements: Record<string, number>, ownedItems: Record<string, number>) => {
    // Calculate expected drops per farm for each stage and item
    const efficiencyMap: Record<string, { stageId: string; efficiency: number }[]> = {};

    stages.forEach((stage) => {
        stage.drops.forEach((drop) => {
            const efficiency = drop.quantity / drop.times;
            if (!efficiencyMap[drop.itemId]) {
                efficiencyMap[drop.itemId] = [];
            }
            efficiencyMap[drop.itemId].push({ stageId: stage.stageId, efficiency });
        });
    });

    // Sort stages by efficiency for each item
    Object.keys(efficiencyMap).forEach((itemId) => {
        efficiencyMap[itemId].sort((a, b) => b.efficiency - a.efficiency);
    });

    // Calculate the plan
    const plan: Record<string, { stageId: string; times: number }[]> = {};

    Object.keys(requirements).forEach((itemId) => {
        if (!plan[itemId]) {
            plan[itemId] = [];
        }

        let remaining = requirements[itemId];
        if (ownedItems[itemId]) {
            remaining -= ownedItems[itemId];
        }

        while (remaining > 0) {
            const stageInfo = efficiencyMap[itemId][0]; // Start with the most efficient stage
            const times = Math.ceil(remaining / stageInfo.efficiency);
            plan[itemId].push({ stageId: stageInfo.stageId, times });
            remaining -= times * stageInfo.efficiency;
        }
    });

    return plan;
};
