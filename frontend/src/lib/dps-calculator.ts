/**
 * Client-side DPS Calculator API Functions
 *
 * This module provides functions for fetching DPS calculation data from the API.
 * Use these functions in React components to interact with the DPS calculator backend.
 */

import type { DpsCalculateRequest, DpsCalculateResponse, DpsEnemyStats, DpsListOperatorsResponse, DpsOperatorParams, DpsRangeParams } from "~/types/api/impl/dps-calculator";

// Re-export types for convenience
export type {
    DpsBuffs,
    DpsCalculateRequest,
    DpsCalculateResponse,
    DpsConditionals,
    DpsEnemyStats,
    DpsListOperatorsResponse,
    DpsOperatorInfo,
    DpsOperatorListEntry,
    DpsOperatorParams,
    DpsRangeDataPoint,
    DpsRangeParams,
    DpsRangeResult,
    DpsShred,
    DpsSingleResult,
} from "~/types/api/impl/dps-calculator";

export { isDpsRangeResult, isDpsSingleResult } from "~/types/api/impl/dps-calculator";

const API_BASE = "/api/dps-calculator";

/**
 * Error class for DPS calculator API errors
 */
export class DpsCalculatorError extends Error {
    constructor(
        message: string,
        public statusCode: number,
        public details?: string,
    ) {
        super(message);
        this.name = "DpsCalculatorError";
    }
}

/**
 * Fetch all operators that have DPS calculator implementations
 *
 * @returns List of operators with their metadata
 * @throws {DpsCalculatorError} If the request fails
 *
 * @example
 * ```ts
 * const { operators, count } = await fetchDpsOperators();
 * console.log(`Found ${count} operators with DPS calculators`);
 * ```
 */
export async function fetchDpsOperators(): Promise<DpsListOperatorsResponse> {
    const response = await fetch(API_BASE, {
        method: "GET",
        headers: {
            "Content-Type": "application/json",
        },
    });

    if (!response.ok) {
        const error = await response.json().catch(() => ({}));
        throw new DpsCalculatorError(error.error ?? "Failed to fetch operators", response.status, JSON.stringify(error));
    }

    return response.json() as Promise<DpsListOperatorsResponse>;
}

/**
 * Calculate DPS for a specific operator
 *
 * @param operatorId - The operator ID (e.g., "char_017_huang" for Blaze)
 * @param params - Optional operator configuration (potential, skill, module, etc.)
 * @param enemy - Optional enemy stats to calculate against
 * @param range - Optional range parameters for generating DPS curves
 * @returns DPS calculation results and operator metadata
 * @throws {DpsCalculatorError} If the request fails
 *
 * @example
 * ```ts
 * // Simple calculation with defaults
 * const result = await calculateDps("char_017_huang");
 *
 * // With custom parameters
 * const result = await calculateDps("char_017_huang", {
 *   skillIndex: 2,
 *   masteryLevel: 3,
 *   moduleIndex: 1,
 * }, {
 *   defense: 500,
 *   res: 0,
 * });
 * ```
 */
export async function calculateDps(operatorId: string, params?: DpsOperatorParams, enemy?: DpsEnemyStats, range?: DpsRangeParams): Promise<DpsCalculateResponse> {
    const request: DpsCalculateRequest = {
        operatorId,
        params,
        enemy,
        range,
    };

    const response = await fetch(API_BASE, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(request),
    });

    if (!response.ok) {
        const error = await response.json().catch(() => ({}));
        throw new DpsCalculatorError(error.error ?? "Failed to calculate DPS", response.status, JSON.stringify(error));
    }

    return response.json() as Promise<DpsCalculateResponse>;
}

/**
 * Calculate DPS range for generating charts/graphs
 *
 * Convenience wrapper around calculateDps that sets up range parameters.
 *
 * @param operatorId - The operator ID
 * @param params - Optional operator configuration
 * @param rangeConfig - Range configuration for DEF/RES iteration
 * @returns DPS calculation results with range data
 *
 * @example
 * ```ts
 * const result = await calculateDpsRange("char_017_huang", {
 *   skillIndex: 2,
 * }, {
 *   maxDef: 2000,
 *   defStep: 50,
 * });
 *
 * // Use result.dps.byDefense for chart data
 * ```
 */
export async function calculateDpsRange(operatorId: string, params?: DpsOperatorParams, rangeConfig?: DpsRangeParams): Promise<DpsCalculateResponse> {
    return calculateDps(
        operatorId,
        params,
        undefined, // No single enemy stats for range calc
        rangeConfig ?? {
            minDef: 0,
            maxDef: 3000,
            defStep: 100,
            minRes: 0,
            maxRes: 120,
            resStep: 10,
        },
    );
}

/**
 * Compare DPS between multiple operators
 *
 * @param operatorIds - Array of operator IDs to compare
 * @param params - Optional shared parameters to apply to all operators
 * @param enemy - Optional enemy stats to calculate against
 * @returns Array of DPS results for each operator
 *
 * @example
 * ```ts
 * const results = await compareDps(
 *   ["char_017_huang", "char_172_svrash", "char_293_thorns"],
 *   { skillIndex: 2, masteryLevel: 3 },
 *   { defense: 500, res: 0 }
 * );
 * ```
 */
export async function compareDps(operatorIds: string[], params?: DpsOperatorParams, enemy?: DpsEnemyStats): Promise<DpsCalculateResponse[]> {
    const results = await Promise.all(operatorIds.map((operatorId) => calculateDps(operatorId, params, enemy)));
    return results;
}

/**
 * Search operators by name (client-side filtering)
 *
 * @param query - Search query string
 * @param operators - List of operators to search (from fetchDpsOperators)
 * @returns Filtered list of operators matching the query
 *
 * @example
 * ```ts
 * const { operators } = await fetchDpsOperators();
 * const results = searchOperators("blaze", operators);
 * ```
 */
export function searchOperators(query: string, operators: DpsListOperatorsResponse["operators"]): DpsListOperatorsResponse["operators"] {
    const lowerQuery = query.toLowerCase();
    return operators.filter((op) => op.name.toLowerCase().includes(lowerQuery) || op.id.toLowerCase().includes(lowerQuery) || op.calculatorName.toLowerCase().includes(lowerQuery));
}

/**
 * Filter operators by rarity
 *
 * @param rarity - Rarity to filter by (1-6)
 * @param operators - List of operators to filter
 * @returns Filtered list of operators with the specified rarity
 */
export function filterOperatorsByRarity(rarity: number, operators: DpsListOperatorsResponse["operators"]): DpsListOperatorsResponse["operators"] {
    return operators.filter((op) => op.rarity === rarity);
}

/**
 * Filter operators by profession
 *
 * @param profession - Profession to filter by (e.g., "Warrior", "Sniper")
 * @param operators - List of operators to filter
 * @returns Filtered list of operators with the specified profession
 */
export function filterOperatorsByProfession(profession: string, operators: DpsListOperatorsResponse["operators"]): DpsListOperatorsResponse["operators"] {
    const lowerProfession = profession.toLowerCase();
    return operators.filter((op) => op.profession.toLowerCase() === lowerProfession);
}
