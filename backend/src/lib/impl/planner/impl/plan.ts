import { Stage } from "../../../../types/types";
import { stages } from "./before";

export const getPlan = async (requirements: Record<string, number>, ownedItems: Record<string, number>) => {
    // Create a list of all possible stages
    let possibleStages = stages.filter(stage => {
        // Check if the stage drops an item that is in the requirements
        return stage.drops.some(drop => requirements[drop.itemId] >  0);
    });

    // Calculate the expected value of each stage
    possibleStages = possibleStages.map(stage => {
        let totalExpectedValue =  0;
        let totalQuantity =  0;

        stage.drops.forEach(drop => {
            if (requirements[drop.itemId] >  0) {
                totalExpectedValue += drop.quantity * (1 - drop.probability);
                totalQuantity += drop.quantity;
            }
        });

        return {
            ...stage,
            expectedValue: totalExpectedValue / totalQuantity,
        };
    });

    // Sort the stages by expected value in descending order
    possibleStages.sort((a, b) => (b as Stage & { expectedValue: number }).expectedValue - (a as Stage & { expectedValue: number }).expectedValue);

    // Create a plan
    const plan: Record<string, {
        stage: Stage,
        amount: number
    }> = {};

    // Iterate through the stages
    for (const stage of possibleStages) {
        // Check if the stage drops an item that is in the requirements
        const drop = stage.drops.find(drop => requirements[drop.itemId] >  0);
        if (!drop) continue;

        // Check if the player already has the item
        if (ownedItems[drop.itemId] >= requirements[drop.itemId]) continue;

        // Calculate how much of the item the player needs
        const needed = requirements[drop.itemId] - (ownedItems[drop.itemId] ||  0);

        // Add the stage to the plan
        plan[stage.stageId] = {
            stage,
            amount: Math.ceil(needed / drop.quantity)
        };

        // Update the requirements
        requirements[drop.itemId] -= needed;
    }

    return plan;
};