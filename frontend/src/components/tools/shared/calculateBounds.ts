/**
 * The bounds `CalculateRequest::validate` (`backend/src/app/services/dps.rs`) enforces on
 * the request fields a share link can carry, `[min, max]` inclusive. A decoded link is
 * clamped to these, so a hand-edited or old link loads cards the backend computes instead
 * of cards whose every request is a 400. `calculateBounds.test.ts` reads the backend source
 * and fails when a bound here and the backend's drift apart.
 */
export const CALCULATE_BOUNDS = {
    promotion: [0, 2],
    level: [1, 90],
    potential: [1, 6],
    trust: [0, 200],
    skillIndex: [0, 3],
    moduleIndex: [-1, 5],
    moduleLevel: [0, 3],
    targets: [1, 50],
    defense: [0, 100_000],
    res: [0, 1_000],
    spBoost: [-10, 100],
    "buffs.aspd": [-500, 1_000],
    "buffs.flatAtk": [-100_000, 100_000],
    "buffs.atk": [-10, 100],
    "buffs.fragile": [-10, 100],
    "secondary.def": [0, 100_000],
    "secondary.res": [0, 1_000],
    "secondary.count": [1, 49],
} as const satisfies Record<string, readonly [number, number]>;

export type RequestField = keyof typeof CALCULATE_BOUNDS;

/** `v` inside the backend's bounds for `field`, and inside `[lo, hi]` when the page's control is narrower. */
export function clampField(field: RequestField, v: number, lo: number = CALCULATE_BOUNDS[field][0], hi: number = CALCULATE_BOUNDS[field][1]): number {
    const [min, max] = CALCULATE_BOUNDS[field];
    return Math.min(Math.min(hi, max), Math.max(Math.max(lo, min), v));
}

/** The fields of a calculate request body that fall outside `CALCULATE_BOUNDS`, as `field=value`. Empty when the backend's range checks would all pass. */
export function outOfBounds(request: object): string[] {
    const out: string[] = [];
    for (const [field, [min, max]] of Object.entries(CALCULATE_BOUNDS)) {
        const v = field.split(".").reduce<unknown>((o, k) => (o && typeof o === "object" ? (o as Record<string, unknown>)[k] : undefined), request);
        if (typeof v === "number" && !(v >= min && v <= max)) out.push(`${field}=${v}`);
    }
    return out;
}
