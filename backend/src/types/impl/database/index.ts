export type ColumnMetadata = {
    primaryKey?: boolean;
    notNull?: boolean;
    unique?: boolean;
    defaultValue?: any;
    references?: { table: string; column: string };
};

export type JsonPath<T> = T extends object
    ? {
          [K in keyof T]: T[K] extends object ? [K, ...JsonPath<T[K]>] : [K];
      }[keyof T]
    : never;

export type FlattenedKeys<T> = {
    [K in keyof T]: T[K] extends object ? never : K;
}[keyof T];
