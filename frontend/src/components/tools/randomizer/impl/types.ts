export interface RandomizerSettings {
    allowedClasses: string[];
    allowedRarities: number[];
    allowedZoneTypes: string[];
    squadSize: number;
    allowDuplicates: boolean;
    allowReserveOperators: boolean;
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
