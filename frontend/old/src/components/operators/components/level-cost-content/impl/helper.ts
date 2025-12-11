import type { Item } from "~/types/impl/api/static/material";
import type { MaterialCost, SkillLevelCost } from "~/types/impl/frontend/impl/operators";

/**
 * Fetches material data by item ID
 */
export function fetchMaterial(itemId: string, materials: Item[]): Item | null {
    if (materials.length > 0) {
        return materials.find((item) => item.itemId === itemId) ?? null;
    } else {
        return null;
    }
}

/**
 * Calculates total materials needed up to a specific level for a specific skill
 */
export function calculateTotalMaterials(costs: SkillLevelCost[], _skillId: string, upToIndex: number) {
    if (!costs || upToIndex < 0 || costs.length === 0) {
        return [];
    }

    const totalMaterials: Record<string, MaterialCost> = {};

    // Combine materials from level 1 up to the selected level
    for (let i = 0; i <= upToIndex && i < costs.length; i++) {
        const levelCost = costs[i];
        if (levelCost?.materials) {
            levelCost.materials.forEach((material: MaterialCost) => {
                const itemId = material.material.itemId;
                if (totalMaterials[itemId]) {
                    totalMaterials[itemId].quantity += material.quantity;
                } else {
                    totalMaterials[itemId] = { ...material };
                }
            });
        }
    }

    return Object.values(totalMaterials);
}

/**
 * Calculates total module materials needed
 */
export function calculateTotalModuleMaterials(costs: MaterialCost[][], _moduleId: string, upToLevel: number) {
    if (!costs || upToLevel < 0 || costs.length === 0) {
        return [];
    }

    const totalMaterials: Record<string, MaterialCost> = {};

    // Combine materials from level 1 up to the selected level
    for (let i = 0; i <= upToLevel && i < costs.length; i++) {
        const levelCost = costs[i];
        if (levelCost) {
            levelCost.forEach((material) => {
                const itemId = material.material.itemId;
                if (totalMaterials[itemId]) {
                    totalMaterials[itemId].quantity += material.quantity;
                } else {
                    totalMaterials[itemId] = { ...material };
                }
            });
        }
    }

    return Object.values(totalMaterials);
}

/**
 * Get the grid columns based on the number of materials
 */
export function getGridColumns(materialCount: number) {
    if (materialCount <= 3) return "grid-cols-3";
    if (materialCount <= 4) return "grid-cols-4";
    if (materialCount <= 6) return "grid-cols-6";
    return "grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6";
}
