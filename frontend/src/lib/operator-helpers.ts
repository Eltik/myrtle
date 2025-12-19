import { descriptionToHtml } from "~/lib/description-parser";
import type { Blackboard, OperatorPhase, Talent, TalentCandidate } from "~/types/api";
import type { InterpolatedValue } from "~/types/frontend/operators";

/**
 * Formats operator description with blackboard interpolation.
 * Combines trait blackboard values for proper value substitution.
 */
export function formatOperatorDescription(description: string, blackboard: Blackboard[]): string {
    if (!description) return "";

    const validBlackboard = blackboard.filter((b) => b.key != null);

    const interpolatedValues: InterpolatedValue[] = validBlackboard.map((b) => ({
        key: b.key,
        value: b.value,
    }));

    return descriptionToHtml(description, interpolatedValues);
}

/**
 * Format attribute key to human-readable label.
 */
export function formatAttributeKey(key: string): string {
    const keyMap: Record<string, string> = {
        atk: "ATK",
        max_hp: "HP",
        def: "DEF",
        attack_speed: "ASPD",
        magic_resistance: "RES",
        cost: "DP Cost",
        respawn_time: "Redeploy",
        block_cnt: "Block",
    };
    return keyMap[key] ?? key.replace(/_/g, " ").toUpperCase();
}

/**
 * Convert phase string to numeric index for comparison.
 */
export function phaseToIndex(phase: OperatorPhase): number {
    switch (phase) {
        case "PHASE_0":
            return 0;
        case "PHASE_1":
            return 1;
        case "PHASE_2":
            return 2;
        default:
            return 0;
    }
}

/**
 * Get the appropriate talent candidate based on current phase, level, and potential rank.
 * Returns the highest-tier candidate that the operator has unlocked.
 */
export function getActiveTalentCandidate(talent: Talent, currentPhaseIndex: number, currentLevel: number, currentPotentialRank: number): TalentCandidate | null {
    if (!talent.Candidates || talent.Candidates.length === 0) {
        return null;
    }

    const unlockedCandidates = talent.Candidates.filter((candidate) => {
        const requiredPhaseIndex = phaseToIndex(candidate.UnlockCondition.Phase);
        const requiredLevel = candidate.UnlockCondition.Level;
        const requiredPotential = candidate.RequiredPotentialRank;

        if (currentPhaseIndex < requiredPhaseIndex) {
            return false;
        }

        if (currentPhaseIndex === requiredPhaseIndex && currentLevel < requiredLevel) {
            return false;
        }

        if (currentPotentialRank < requiredPotential) {
            return false;
        }

        return true;
    });

    if (unlockedCandidates.length === 0) {
        return null;
    }

    return unlockedCandidates[unlockedCandidates.length - 1] ?? null;
}

/**
 * Convert Blackboard array to InterpolatedValue array for description parsing.
 */
export function blackboardToInterpolatedValues(blackboard: Blackboard[]): InterpolatedValue[] {
    return blackboard.map((b) => ({ key: b.key, value: b.value }));
}
