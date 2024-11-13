export type ColumnMetadata = {
    primaryKey?: boolean;
    notNull?: boolean;
    unique?: boolean;
    defaultValue?: any;
    references?: { table: string; column: string };
};
