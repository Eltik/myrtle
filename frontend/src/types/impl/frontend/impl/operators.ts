import type { OperatorPhase } from "../../api/static/operator";

export enum GridCell {
    Operator = "operator",
    empty = "empty",
    active = "active",
}

export type NormalizedRange = {
    rows: number;
    cols: number;
    grid: GridCell[][];
};

export type InterpolatedValue = {
    key: string;
    value: number;
};

export type MaterialCost = {
    quantity: number;
    material: {
        itemId: string;
        name: string;
    };
};

export type SkillLevelCost = {
    level: number;
    phase: OperatorPhase;
    materials: MaterialCost[];
};

// Voice line types
export type VoiceLine = {
    id: string;
    name: string;
    description: string;
    transcript: string;
    url: string;
    cvName: string[];
};

export type VoiceCategory = {
    id: string;
    name: string;
    lines: VoiceLine[];
};

export interface UISkin {
    id: string;
    name: string;
    description: string;
    image: string;
    obtainMethod: string;
    releaseDate: string;
    artists: string[];
    voiceLines: boolean;
    animations: boolean;
    available: boolean;
    isDefault: boolean;
}
