export type Ranges = Record<string, Range>;
export type Range = {
    id: string;
    direction: number;
    grids: {
        row: number;
        col: number;
    }[];
};
