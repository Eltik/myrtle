/**
 * Helpers for the optional-index records the generated bindings use.
 *
 * ts-rs models every Rust `HashMap<K, V>` as `{ [k in string]?: V }` rather than
 * `Record<string, V>`, which is the honest shape: indexing a map with a key it
 * does not hold yields `undefined` at runtime, and `Record` lies about that.
 *
 * The cost is that `Object.values(map)` widens to `(V | undefined)[]`. These
 * helpers narrow it back in one place instead of at every call site.
 */

/** Optional-index record, as emitted by ts-rs for a Rust `HashMap`. */
export type SparseRecord<V> = { [key in string]?: V };

/** `Object.values`, minus the holes. */
export function values<V>(record: SparseRecord<V> | null | undefined): V[] {
    if (!record) return [];
    return Object.values(record).filter((v): v is V => v !== undefined);
}

/** `Object.entries`, minus the holes. */
export function entries<V>(record: SparseRecord<V> | null | undefined): [string, V][] {
    if (!record) return [];
    return Object.entries(record).filter((e): e is [string, V] => e[1] !== undefined);
}

/**
 * Densify a sparse record into a total one.
 *
 * Use when a value is threaded through code that genuinely cannot handle holes.
 * Prefer {@link values}/{@link entries} where you only need to iterate.
 */
export function dense<V>(record: SparseRecord<V> | null | undefined): Record<string, V> {
    return Object.fromEntries(entries(record));
}
