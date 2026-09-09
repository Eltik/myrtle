/**
 * Type-level mirror of `deepCamelize` in `#/lib/api/operators`.
 *
 * The operator endpoints serve the `character_table`-mirroring structs with
 * PascalCase and trailing-underscore keys (`AttributesKeyFrames`, `MaxHp`,
 * `Type_`), and `deepCamelize` normalizes them at the fetch boundary. So the
 * ts-rs bindings - which describe the WIRE shape - can't be used directly by
 * components, which see the normalized shape.
 *
 * `Camelize<T>` applies the same transform in the type system, letting the
 * frontend types stay derived from the Rust structs without changing the wire
 * format or the runtime normalizer.
 */

/** `"MaxHp"` -> `"maxHp"`, `"Type_"` -> `"type"`, `"id"` -> `"id"`. */
type StripTrailing<K extends string> = K extends `${infer B}_` ? B : K;
type LowerFirst<K extends string> = K extends `${infer F}${infer R}` ? `${Lowercase<F>}${R}` : K;
export type CamelKey<K extends string> = LowerFirst<StripTrailing<K>>;

/**
 * Recursively rename keys. Arrays map elementwise; `null` and primitives pass
 * through untouched, so nullable fields keep their nullability.
 */
export type Camelize<T> = T extends readonly (infer U)[] ? Camelize<U>[] : T extends object ? { [K in keyof T as CamelKey<K & string>]: Camelize<T[K]> } : T;

// ---------------------------------------------------------------------------
// Compile-time assertions. These are the test: if the transform regresses, the
// build fails here rather than somewhere downstream.
// ---------------------------------------------------------------------------
type Assert<A, B> = [A] extends [B] ? ([B] extends [A] ? true : never) : never;

type _Sample = {
    Id: string;
    MaxHp: number;
    Type_: string;
    AttributesKeyFrames: { Level: number; Data: { Atk: number; BlockCnt: number } }[];
    EvolveCost: { Id: string; Count: number } | null;
    alreadyCamel: boolean;
};

type _Expected = {
    id: string;
    maxHp: number;
    type: string;
    attributesKeyFrames: { level: number; data: { atk: number; blockCnt: number } }[];
    evolveCost: { id: string; count: number } | null;
    alreadyCamel: boolean;
};

// eslint-disable-next-line @typescript-eslint/no-unused-vars
type _Check = Assert<Camelize<_Sample>, _Expected>;
const _proof: _Check = true;
void _proof;
