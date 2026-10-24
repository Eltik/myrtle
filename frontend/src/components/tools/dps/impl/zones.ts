/**
 * The buff-zone model of `POST /dps/calculate` (`zones`, `backend/src/dps/zones.rs`) as the
 * page holds it: one number per zone the user types, plus supporter presets whose values the
 * backend reads from game data. Pure, so the merge, the stacking display and the legacy
 * migrations are tested (`zones.test.ts`).
 *
 * Units in state are the request's: percentages as fractions (0.4 = +40%), the final ATK
 * multiplier as a factor (1 = none), everything else flat. 0 means unset everywhere except
 * `atkMultiplier`, where 1 does.
 */

import type { ICalcBuffs, IOperatorListEntry } from "#/components/tools/shared/types";
import type { BuffZones } from "#/types/generated/BuffZones";
import type { StatReduction } from "#/types/generated/StatReduction";
import type { SupporterEntry } from "#/types/generated/SupporterEntry";
import type { SupporterZone } from "#/types/generated/SupporterZone";
import type { messages as zoneMessages } from "./components/zones.messages";

export interface IZoneConfig {
    atkFlat: number;
    atkPercent: number;
    inspire: number;
    atkMultiplier: number;
    aspd: number;
    spRecovery: number;
    spAura: number;
    fragile: number;
    physicalFragile: number;
    artsFragile: number;
    damageReduction: number;
    defFlat: number;
    defPercent: number;
    defIgnoreFlat: number;
    defIgnorePercent: number;
    defSteal: number;
    resFlat: number;
    resPercent: number;
    resIgnoreFlat: number;
    resIgnorePercent: number;
    resSteal: number;
}

export type ZoneKey = keyof IZoneConfig;

export const EMPTY_ZONES: IZoneConfig = {
    atkFlat: 0,
    atkPercent: 0,
    inspire: 0,
    atkMultiplier: 1,
    aspd: 0,
    spRecovery: 0,
    spAura: 0,
    fragile: 0,
    physicalFragile: 0,
    artsFragile: 0,
    damageReduction: 0,
    defFlat: 0,
    defPercent: 0,
    defIgnoreFlat: 0,
    defIgnorePercent: 0,
    defSteal: 0,
    resFlat: 0,
    resPercent: 0,
    resIgnoreFlat: 0,
    resIgnorePercent: 0,
    resSteal: 0,
};

/** How the sources of one zone combine, as `dps::zones` resolves them. */
export type ZoneStacking = "additive" | "max" | "multiply";

/** `percent`: shown as a whole percent, stored as a fraction. `factor`: shown and stored as `x1.15`. */
export type ZoneUnit = "flat" | "percent" | "factor";

export type ZoneMessageKey = keyof typeof zoneMessages & string;

export interface IZoneField {
    key: ZoneKey;
    labelKey: ZoneMessageKey;
    stacking: ZoneStacking;
    unit: ZoneUnit;
    /** Shown without "show more". */
    common: boolean;
    /** Input bounds and step in DISPLAY units, inside the backend's `validate_zones` bounds. */
    min: number;
    max: number;
    step: number;
}

/** Every zone, in panel order. The order is also the share-link vector order: append, never reorder. */
export const ZONE_FIELDS: readonly IZoneField[] = [
    { key: "atkPercent", labelKey: "dps.zone.atkPercent", stacking: "additive", unit: "percent", common: true, min: -100, max: 1000, step: 5 },
    { key: "inspire", labelKey: "dps.zone.inspire", stacking: "max", unit: "flat", common: true, min: 0, max: 5000, step: 10 },
    { key: "aspd", labelKey: "dps.zone.aspd", stacking: "additive", unit: "flat", common: true, min: -100, max: 500, step: 5 },
    { key: "spRecovery", labelKey: "dps.zone.spRecovery", stacking: "additive", unit: "flat", common: true, min: 0, max: 5, step: 0.1 },
    { key: "fragile", labelKey: "dps.zone.fragile", stacking: "max", unit: "percent", common: true, min: 0, max: 300, step: 5 },
    { key: "defPercent", labelKey: "dps.zone.defPercent", stacking: "additive", unit: "percent", common: true, min: 0, max: 100, step: 5 },
    { key: "defFlat", labelKey: "dps.zone.defFlat", stacking: "additive", unit: "flat", common: true, min: 0, max: 3000, step: 50 },
    { key: "resPercent", labelKey: "dps.zone.resPercent", stacking: "additive", unit: "percent", common: true, min: 0, max: 100, step: 5 },
    { key: "resFlat", labelKey: "dps.zone.resFlat", stacking: "additive", unit: "flat", common: true, min: 0, max: 100, step: 5 },
    { key: "atkFlat", labelKey: "dps.zone.atkFlat", stacking: "additive", unit: "flat", common: false, min: 0, max: 5000, step: 10 },
    { key: "atkMultiplier", labelKey: "dps.zone.atkMultiplier", stacking: "multiply", unit: "factor", common: false, min: 0.05, max: 10, step: 0.05 },
    { key: "spAura", labelKey: "dps.zone.spAura", stacking: "max", unit: "flat", common: false, min: 0, max: 5, step: 0.1 },
    { key: "physicalFragile", labelKey: "dps.zone.physicalFragile", stacking: "max", unit: "percent", common: false, min: 0, max: 300, step: 5 },
    { key: "artsFragile", labelKey: "dps.zone.artsFragile", stacking: "max", unit: "percent", common: false, min: 0, max: 300, step: 5 },
    { key: "damageReduction", labelKey: "dps.zone.damageReduction", stacking: "multiply", unit: "percent", common: false, min: 0, max: 100, step: 5 },
    { key: "defIgnoreFlat", labelKey: "dps.zone.defIgnoreFlat", stacking: "additive", unit: "flat", common: false, min: 0, max: 3000, step: 50 },
    { key: "defIgnorePercent", labelKey: "dps.zone.defIgnorePercent", stacking: "additive", unit: "percent", common: false, min: 0, max: 100, step: 5 },
    { key: "defSteal", labelKey: "dps.zone.defSteal", stacking: "max", unit: "flat", common: false, min: 0, max: 3000, step: 10 },
    { key: "resIgnoreFlat", labelKey: "dps.zone.resIgnoreFlat", stacking: "additive", unit: "flat", common: false, min: 0, max: 100, step: 5 },
    { key: "resIgnorePercent", labelKey: "dps.zone.resIgnorePercent", stacking: "additive", unit: "percent", common: false, min: 0, max: 100, step: 5 },
    { key: "resSteal", labelKey: "dps.zone.resSteal", stacking: "max", unit: "flat", common: false, min: 0, max: 100, step: 1 },
];

export const ZONE_KEYS: readonly ZoneKey[] = ZONE_FIELDS.map((f) => f.key);

const FIELD_BY_KEY = new Map(ZONE_FIELDS.map((f) => [f.key, f]));

export function zoneField(key: ZoneKey): IZoneField {
    const f = FIELD_BY_KEY.get(key);
    if (!f) throw new Error(`unknown zone ${key}`);
    return f;
}

/** The value a zone has when nothing is set. */
export function zoneDefault(key: ZoneKey): number {
    return EMPTY_ZONES[key];
}

export function isZoneSet(zones: IZoneConfig, key: ZoneKey): boolean {
    return zones[key] !== zoneDefault(key);
}

/** Stored value -> the number the input shows. */
export function toDisplay(unit: ZoneUnit, value: number): number {
    return unit === "percent" ? Math.round(value * 100 * 1e6) / 1e6 : value;
}

/** The number the input shows -> stored value. */
export function fromDisplay(unit: ZoneUnit, value: number): number {
    return unit === "percent" ? value / 100 : value;
}

/** `GET /dps/supporters` zone -> the panel's zone. */
export const SUPPORTER_ZONE_KEYS: Readonly<Record<SupporterZone, ZoneKey>> = {
    atkPercent: "atkPercent",
    inspire: "inspire",
    aspd: "aspd",
    spRecovery: "spRecovery",
    spAura: "spAura",
    fragile: "fragile",
    artsFragile: "artsFragile",
    defPercent: "defPercent",
    defFlat: "defFlat",
    resPercent: "resPercent",
    resFlat: "resFlat",
};

/** One supporter value headed for one zone. */
export interface ISupporterSource {
    /** The supporter entry id (`warfarin_s2`). */
    id: string;
    zone: ZoneKey;
    value: number;
    /** Game profession codes the effect reaches; empty = every class. */
    professions: readonly string[];
}

/** The contributions of the selected entries, in selection order; ids the data no longer has are skipped. */
export function supporterSources(entries: readonly SupporterEntry[] | undefined, selected: readonly string[]): ISupporterSource[] {
    if (!entries || selected.length === 0) return [];
    const byId = new Map(entries.map((e) => [e.id, e]));
    const out: ISupporterSource[] = [];
    for (const id of selected) {
        const entry = byId.get(id);
        if (!entry) continue;
        for (const c of entry.contributions) out.push({ id, zone: SUPPORTER_ZONE_KEYS[c.zone], value: c.value, professions: entry.scope.professions });
    }
    return out;
}

/** Whether a source reaches an operator of `profession`. An unknown profession is reached (the ranking applies every source to every row). */
export function sourceReaches(source: Pick<ISupporterSource, "professions">, profession: string | undefined): boolean {
    return source.professions.length === 0 || profession === undefined || source.professions.includes(profession);
}

/** One contributor to a zone, for the stacking display. `origin` is "team", "card" or a supporter id. */
export interface IZoneSource {
    /** Unique within the zone: the origin, suffixed when one origin feeds the zone more than once. */
    id: string;
    origin: string;
    value: number;
    /** False for a MAX-group source a larger one beats. */
    applied: boolean;
}

export interface IZoneResolution {
    stacking: ZoneStacking;
    sources: IZoneSource[];
    /** What reaches the engine: the sum, the largest, or the product. */
    total: number;
}

/**
 * The sources of one zone and how they combine. Unset sources are left out. For a MAX
 * group exactly one source is applied, the first largest, as `max_source` picks it.
 */
export function resolveZone(key: ZoneKey, layers: readonly { origin: string; zones: IZoneConfig | undefined }[], supporters: readonly ISupporterSource[]): IZoneResolution {
    const { stacking } = zoneField(key);
    const unset = zoneDefault(key);
    const sources: IZoneSource[] = [];
    for (const layer of layers) {
        const v = layer.zones?.[key];
        if (v !== undefined && v !== unset) sources.push({ id: sourceId(sources, layer.origin), origin: layer.origin, value: v, applied: true });
    }
    for (const s of supporters) if (s.zone === key && s.value !== 0) sources.push({ id: sourceId(sources, s.id), origin: s.id, value: s.value, applied: true });
    if (stacking === "max") {
        let best = -1;
        sources.forEach((s, i) => {
            if (best < 0 || s.value > sources[best].value) best = i;
        });
        sources.forEach((s, i) => {
            s.applied = i === best;
        });
        return { stacking, sources, total: best >= 0 ? sources[best].value : 0 };
    }
    if (stacking === "multiply") {
        // `damageReduction` multiplies (1 - x); the ATK factor multiplies x.
        const total = key === "damageReduction" ? 1 - sources.reduce((p, s) => p * (1 - s.value), 1) : sources.reduce((p, s) => p * s.value, 1);
        return { stacking, sources, total };
    }
    return { stacking, sources, total: sources.reduce((sum, s) => sum + s.value, 0) };
}

function sourceId(existing: readonly IZoneSource[], origin: string): string {
    const n = existing.filter((s) => s.origin === origin).length;
    return n === 0 ? origin : `${origin}#${n}`;
}

/**
 * The request's `zones` for one card: the team panel, the card's own zones and the
 * supporter sources that reach this operator. Additive zones sum, MAX and multiplying
 * zones travel as source lists (the backend applies the rule). Undefined when nothing is
 * set, so a page with no buffs sends no `zones` at all.
 */
/**
 * The backend's request bounds per zone (`validate_zones` / `validate_reduction` in
 * `backend/src/app/services/dps.rs`), mirrored so a sum of in-range inputs never 400s a
 * whole batch: team DEF down 100% plus a DEF% supporter would send 1.3 > 1. Additive zones
 * clamp their SUM, list zones each source. Clamping a DEF / RES percent at 1 is exact (the
 * resolver floors the stat at 0 anyway); clamping anything else is a cost ceiling no real
 * setup reaches.
 */
export const REQUEST_BOUNDS: Readonly<Record<ZoneKey, readonly [number, number]>> = {
    atkFlat: [-100_000, 100_000],
    atkPercent: [-10, 100],
    inspire: [-100_000, 100_000],
    atkMultiplier: [0, 100],
    aspd: [-500, 1_000],
    spRecovery: [-10, 100],
    spAura: [-10, 100],
    fragile: [-10, 100],
    physicalFragile: [-10, 100],
    artsFragile: [-10, 100],
    damageReduction: [0, 1],
    defFlat: [-100_000, 100_000],
    defPercent: [-1, 1],
    defIgnoreFlat: [0, 100_000],
    defIgnorePercent: [0, 1],
    defSteal: [0, 100_000],
    resFlat: [-10_000, 10_000],
    resPercent: [-1, 1],
    resIgnoreFlat: [0, 10_000],
    resIgnorePercent: [0, 1],
    resSteal: [0, 10_000],
};

/** `MAX_SOURCES` in `backend/src/dps/zones.rs`: a list zone takes at most 16 sources. */
export const MAX_ZONE_SOURCES = 16;

function clampZone(key: ZoneKey, v: number): number {
    const [lo, hi] = REQUEST_BOUNDS[key];
    return Math.min(hi, Math.max(lo, v));
}

export function buildRequestZones(team: IZoneConfig, card: IZoneConfig | undefined, supporters: readonly ISupporterSource[], profession?: string): BuffZones | undefined {
    const reaching = supporters.filter((s) => sourceReaches(s, profession));
    const layers = [
        { origin: "team", zones: team },
        { origin: "card", zones: card },
    ];
    const add = (key: ZoneKey): number | undefined => {
        const r = resolveZone(key, layers, reaching);
        return r.sources.length > 0 ? clampZone(key, r.total) : undefined;
    };
    const list = (key: ZoneKey): number[] => {
        const r = resolveZone(key, layers, reaching);
        const values = r.sources.map((s) => clampZone(key, s.value));
        // A MAX group keeps its largest sources: the backend applies only the largest, so dropping the rest is exact.
        return r.stacking === "max" && values.length > MAX_ZONE_SOURCES ? values.sort((a, b) => b - a).slice(0, MAX_ZONE_SOURCES) : values.slice(0, MAX_ZONE_SOURCES);
    };
    const reduction = (prefix: "def" | "res"): StatReduction => ({
        ...defined("flat", add(`${prefix}Flat`)),
        ...defined("percent", add(`${prefix}Percent`)),
        ...defined("ignoreFlat", add(`${prefix}IgnoreFlat`)),
        ...defined("ignorePercent", add(`${prefix}IgnorePercent`)),
        steal: list(`${prefix}Steal`),
    });
    const zones: BuffZones = {
        ...defined("atkFlat", add("atkFlat")),
        ...defined("atkPercent", add("atkPercent")),
        inspire: list("inspire"),
        // The backend rejects a factor of 0 or below (Thorns module 3's fallout term turns 0 * inf into NaN).
        atkMultiplier: list("atkMultiplier").filter((v) => v > 0),
        ...defined("aspd", add("aspd")),
        ...defined("spRecovery", add("spRecovery")),
        spAura: list("spAura"),
        fragile: list("fragile"),
        physicalFragile: list("physicalFragile"),
        artsFragile: list("artsFragile"),
        damageReduction: list("damageReduction"),
        def: reduction("def"),
        res: reduction("res"),
    };
    return hasAnyZone(zones) ? compactZones(zones) : undefined;
}

function defined<K extends string>(key: K, v: number | undefined): Partial<Record<K, number>> {
    return v === undefined ? {} : ({ [key]: v } as Record<K, number>);
}

function hasAnyZone(z: object): boolean {
    return Object.values(z).some((v) => (Array.isArray(v) ? v.length > 0 : typeof v === "number" ? true : v && typeof v === "object" ? hasAnyZone(v) : false));
}

/** Drop empty lists and empty reductions: the backend defaults them, and the request key stays short. */
function compactZones<T extends object>(z: T): T {
    const out: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(z)) {
        if (Array.isArray(v)) {
            if (v.length > 0) out[k] = v;
        } else if (v && typeof v === "object") {
            if (hasAnyZone(v)) out[k] = compactZones(v);
        } else if (v !== undefined) out[k] = v;
    }
    return out as T;
}

/**
 * A card's old External buffs as zones. Each maps onto the zone the backend joins the
 * legacy field into, which `dps_phase1_test` holds bit-identical for one source: `atk` ->
 * ATK %, `flatAtk` -> Inspire, `aspd` -> ASPD, `fragile` -> generic Fragile.
 */
export function legacyBuffsToZones(buffs: ICalcBuffs | undefined): IZoneConfig | undefined {
    if (!buffs) return undefined;
    const z = { ...EMPTY_ZONES };
    let any = false;
    const set = (key: ZoneKey, v: number | undefined) => {
        if (typeof v === "number" && Number.isFinite(v) && v !== 0) {
            z[key] = v;
            any = true;
        }
    };
    set("atkPercent", buffs.atk);
    set("inspire", buffs.flatAtk);
    set("aspd", buffs.aspd);
    set("fragile", buffs.fragile);
    return any ? z : undefined;
}

/** The old enemy panel's SP boost and shred (whole percent) as team zones. `shred` -> DEF / RES flat and %, `spBoost` -> SP recovery. */
export function legacyEnemyToZones(enemy: { spBoost?: number; shred?: { def?: number; defFlat?: number; res?: number; resFlat?: number } }): IZoneConfig {
    const z = { ...EMPTY_ZONES };
    const num = (v: unknown) => (typeof v === "number" && Number.isFinite(v) ? v : 0);
    z.spRecovery = num(enemy.spBoost);
    z.defPercent = num(enemy.shred?.def) / 100;
    z.defFlat = num(enemy.shred?.defFlat);
    z.resPercent = num(enemy.shred?.res) / 100;
    z.resFlat = num(enemy.shred?.resFlat);
    return z;
}

/** Fill a stored or decoded zone object to the current shape; null when it is not an object. */
export function normaliseZones(raw: unknown): IZoneConfig | null {
    if (!raw || typeof raw !== "object" || Array.isArray(raw)) return null;
    const r = raw as Record<string, unknown>;
    const z = { ...EMPTY_ZONES };
    for (const key of ZONE_KEYS) {
        const v = r[key];
        if (typeof v === "number" && Number.isFinite(v)) z[key] = v;
    }
    if (!(z.atkMultiplier > 0)) z.atkMultiplier = 1;
    return z;
}

/** Positional vector in {@link ZONE_KEYS} order, for the share link. */
export function encodeZones(z: IZoneConfig): number[] {
    return ZONE_KEYS.map((k) => z[k]);
}

/** Null unless every entry is a finite number; a shorter vector (an older link) fills the rest with defaults. */
export function decodeZones(raw: unknown): IZoneConfig | null {
    if (!Array.isArray(raw) || raw.length > ZONE_KEYS.length || !raw.every((v) => typeof v === "number" && Number.isFinite(v))) return null;
    const z = { ...EMPTY_ZONES };
    raw.forEach((v: number, i) => {
        z[ZONE_KEYS[i]] = v;
    });
    if (!(z.atkMultiplier > 0)) z.atkMultiplier = 1;
    return z;
}

/** Number of zones set away from their default, for the collapsed-section badge. */
export function countSetZones(z: IZoneConfig | undefined): number {
    if (!z) return 0;
    return ZONE_KEYS.filter((k) => isZoneSet(z, k)).length;
}

/** The profession for supporter scoping, from the operator index; undefined when it is not loaded. */
export type ProfessionLookup = (op: Pick<IOperatorListEntry, "id">) => string | undefined;

/** One row of the "which multiplier is this?" helper: a phrasing from the game and the zone it belongs in. */
export interface IZoneHelperRow {
    wordingKey: ZoneMessageKey;
    noteKey: ZoneMessageKey;
    zones: readonly ZoneKey[];
}

export const ZONE_HELPER_ROWS: readonly IZoneHelperRow[] = [
    { wordingKey: "dps.helper.atkPercent.wording", noteKey: "dps.helper.atkPercent.note", zones: ["atkPercent"] },
    { wordingKey: "dps.helper.inspire.wording", noteKey: "dps.helper.inspire.note", zones: ["inspire"] },
    { wordingKey: "dps.helper.atkFlat.wording", noteKey: "dps.helper.atkFlat.note", zones: ["atkFlat"] },
    { wordingKey: "dps.helper.atkMultiplier.wording", noteKey: "dps.helper.atkMultiplier.note", zones: ["atkMultiplier"] },
    { wordingKey: "dps.helper.aspd.wording", noteKey: "dps.helper.aspd.note", zones: ["aspd"] },
    { wordingKey: "dps.helper.sp.wording", noteKey: "dps.helper.sp.note", zones: ["spRecovery", "spAura"] },
    { wordingKey: "dps.helper.fragile.wording", noteKey: "dps.helper.fragile.note", zones: ["fragile", "physicalFragile", "artsFragile"] },
    { wordingKey: "dps.helper.def.wording", noteKey: "dps.helper.def.note", zones: ["defPercent", "defFlat"] },
    { wordingKey: "dps.helper.ignore.wording", noteKey: "dps.helper.ignore.note", zones: ["defIgnorePercent", "defIgnoreFlat"] },
    { wordingKey: "dps.helper.steal.wording", noteKey: "dps.helper.steal.note", zones: ["defSteal"] },
    { wordingKey: "dps.helper.reduction.wording", noteKey: "dps.helper.reduction.note", zones: ["damageReduction"] },
];

export const STACKING_LABEL_KEYS: Readonly<Record<ZoneStacking, ZoneMessageKey>> = {
    additive: "dps.zone.stacking.additive",
    max: "dps.zone.stacking.max",
    multiply: "dps.zone.stacking.multiply",
};

/** A zone value as text in display units: `40%`, `×1.2`, `0.3`. */
export function formatZoneValue(unit: ZoneUnit, value: number, locale?: string): string {
    const fmt = (n: number) => n.toLocaleString(locale ?? "en-US", { maximumFractionDigits: 2 });
    if (unit === "percent") return `${fmt(value * 100)}%`;
    if (unit === "factor") return `×${fmt(value)}`;
    return fmt(value);
}
