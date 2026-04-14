/**
 * Raw game data shape used by the randomizer for stage completion
 * and operator E2 checks. This data comes from the old game sync
 * and may not be available in v3 (AuthUser is a flat UserProfile).
 */
export interface GameUserData {
    troop: {
        chars: Record<
            string,
            {
                charId: string;
                evolvePhase: number;
            }
        >;
    };
    dungeon: {
        stages: Record<string, { completeTimes: number }>;
    };
}

export interface RandomizerSettings {
    allowedClasses: string[];
    allowedRarities: number[];
    allowedZoneTypes: string[];
    squadSize: number;
    allowDuplicates: boolean;
    allowUnplayableOperators: boolean;
    onlyCompletedStages: boolean;
    onlyAvailableStages: boolean; // Only show stages from currently open or permanent events
    onlyE2Operators: boolean;
    selectedStages: string[]; // Stage IDs that are manually selected
}

export interface Challenge {
    type: "restriction" | "modifier" | "objective";
    title: string;
    description: string;
}
