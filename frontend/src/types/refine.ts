/**
 * Narrow a generated binding's field without losing the anchor to it.
 *
 * Some backend fields are `String` in Rust because the value set lives in the
 * frontend registry, not in game data (`room_type`, `category`, `group`). The
 * generated binding is honestly `string`; the component that consumes it wants
 * the union.
 *
 * A bare `Omit<T, K> & { ... }` does that, but silently: rename the field in
 * Rust and the `Omit` becomes a no-op while the intersection puts the old key
 * back, so the type still compiles and is now wrong in exactly the way the
 * generated types exist to prevent.
 *
 * `Refine` resolves to `never` when a refined key is not on the base type, so
 * an upstream rename breaks the build at the declaration.
 *
 *     export type ICatalogRoom = Refine<CatalogRoomDto, { room_type: FacilityType }>;
 *
 * Only for narrowing an existing field. To ADD a frontend-only field, intersect
 * normally - there is nothing upstream to anchor to.
 */
export type Refine<T, R> = keyof R extends keyof T ? Omit<T, keyof R> & R : never;

// ---------------------------------------------------------------------------
// Compile-time assertions.
// ---------------------------------------------------------------------------
type _Base = { kind: string; count: number };

// A refined key that exists narrows the field and leaves the rest alone.
type _Ok = Refine<_Base, { kind: "a" | "b" }>;
const _ok: _Ok = { kind: "a", count: 1 };
void _ok;

// A refined key that does NOT exist collapses to `never`.
type _Bad = Refine<_Base, { knid: "a" | "b" }>;
const _bad: [_Bad] extends [never] ? true : never = true;
void _bad;
