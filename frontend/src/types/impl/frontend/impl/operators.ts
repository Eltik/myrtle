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
