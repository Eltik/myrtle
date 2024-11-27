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
