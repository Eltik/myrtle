// Range types

export interface Grid {
    row: number;
    col: number;
}

export interface Range {
    id: string;
    direction: number;
    grids: Grid[];
}

export type Ranges = Record<string, Range>;
