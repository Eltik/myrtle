import glpk, { Options } from "glpk.js";
import { stages } from "./before";

// Good for getting based on the drop percentage. Not for Sanity though.
export const getPlanOLD = async (requirements: Record<string, number>, ownedItems: Record<string, number>) => {
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

export const getPlan = async (requirements: Record<string, number>, ownedItems: Record<string, number>) => {
    // Define the objective function and constraints
    const objective = {
        direction: glpk().GLP_MIN,
        name: "obj",
        vars: [],
    } as {
        direction: number;
        name: string;
        vars: { name: string; coef: number }[];
    };

    const subjectTo = [];

    // Initialize the total required amount of each item
    const totalRequired = { ...requirements };
    for (const itemId in ownedItems) {
        if (totalRequired[itemId]) {
            totalRequired[itemId] -= ownedItems[itemId];
        } else {
            totalRequired[itemId] = -ownedItems[itemId];
        }
    }

    // Prepare variables for the objective function and constraints
    const stageVariables = stages.map((stage, index) => {
        const variableName = `stage_${index}`;
        objective.vars.push({ name: variableName, coef: 1 }); // Minimize the number of farmed items

        // Add constraints for each item requirement
        for (const itemId in totalRequired) {
            if (totalRequired[itemId] > 0) {
                const drop = stage.drops.find((drop) => drop.itemId === itemId);
                if (drop) {
                    // Calculate the expected quantity of this item per farm
                    const expectedQuantity = drop.quantity / drop.times;
                    subjectTo.push({
                        name: `${itemId}_${index}`,
                        vars: [{ name: variableName, coef: expectedQuantity }],
                        bnds: { type: glpk().GLP_LO, lb: totalRequired[itemId], ub: Infinity },
                    });
                }
            }
        }

        return variableName;
    });

    // Add a constraint to ensure we don't farm more than necessary
    subjectTo.push({
        name: "total_farm_times",
        vars: stageVariables.map((variableName) => ({ name: variableName, coef: 1 })),
        bnds: { type: glpk().GLP_LO, lb: 1, ub: Infinity }, // Minimize total farms
    });

    // Solve the linear programming problem
    const options = {
        msglev: glpk().GLP_MSG_ALL,
        presol: true,
        cb: {
            //call: progress => console.log(progress),
            each: 250,
        },
    } as Options;

    const res = glpk().solve(
        {
            name: "FarmingPlan",
            objective: objective,
            subjectTo: subjectTo,
        },
        options,
    );

    // Extract the solution
    const plan = Object.entries(res.result.vars)
        .map((item) => {
            return {
                stageId: stages[parseInt(item[0].split("_")[1])].stageId,
                times: Math.ceil(item[1]), // Round up to ensure we meet the requirement
            };
        })
        .filter((item) => item.times !== 0)
        .sort((a, b) => a.times - b.times);

    return plan;
};
