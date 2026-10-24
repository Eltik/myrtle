import { describe, expect, it } from "vitest";
import type { SupporterEntry } from "#/types/generated/SupporterEntry";
import { buildRequestZones, decodeZones, EMPTY_ZONES, encodeZones, type ISupporterSource, legacyBuffsToZones, legacyEnemyToZones, MAX_ZONE_SOURCES, normaliseZones, REQUEST_BOUNDS, resolveZone, supporterSources, ZONE_FIELDS, ZONE_KEYS } from "./zones";

const src = (id: string, zone: ISupporterSource["zone"], value: number, professions: string[] = []): ISupporterSource => ({ id, zone, value, professions });

describe("legacy buffs -> zones", () => {
    it("maps each legacy card buff onto the zone the backend joins it into", () => {
        expect(legacyBuffsToZones({ atk: 0.4, flatAtk: 120, aspd: 30, fragile: 0.25 })).toEqual({ ...EMPTY_ZONES, atkPercent: 0.4, inspire: 120, aspd: 30, fragile: 0.25 });
    });

    it("gives no zones for empty or zero buffs", () => {
        expect(legacyBuffsToZones({})).toBeUndefined();
        expect(legacyBuffsToZones({ atk: 0, aspd: 0 })).toBeUndefined();
        expect(legacyBuffsToZones(undefined)).toBeUndefined();
    });

    it("sends exactly the legacy values through the request, one source each", () => {
        const card = legacyBuffsToZones({ atk: 0.4, flatAtk: 120, aspd: 30, fragile: 0.25 });
        expect(buildRequestZones(EMPTY_ZONES, card, [])).toEqual({ atkPercent: 0.4, inspire: [120], aspd: 30, fragile: [0.25] });
    });

    it("moves the old enemy SP boost and whole-percent shred into team zones", () => {
        const z = legacyEnemyToZones({ spBoost: 0.2, shred: { def: 40, defFlat: 100, res: 15, resFlat: 10 } });
        expect(z).toEqual({ ...EMPTY_ZONES, spRecovery: 0.2, defPercent: 0.4, defFlat: 100, resPercent: 0.15, resFlat: 10 });
        expect(buildRequestZones(z, undefined, [])).toEqual({ spRecovery: 0.2, def: { flat: 100, percent: 0.4 }, res: { flat: 10, percent: 0.15 } });
        expect(legacyEnemyToZones({})).toEqual(EMPTY_ZONES);
    });
});

describe("buildRequestZones", () => {
    it("sends nothing when nothing is set (the kill switch: no zones key at all)", () => {
        expect(buildRequestZones(EMPTY_ZONES, undefined, [])).toBeUndefined();
        expect(buildRequestZones(EMPTY_ZONES, EMPTY_ZONES, [])).toBeUndefined();
    });

    it("sums additive zones and lists MAX and multiplying sources", () => {
        const team = { ...EMPTY_ZONES, atkPercent: 0.2, inspire: 150, fragile: 0.2, atkMultiplier: 1.1, damageReduction: 0.5 };
        const card = { ...EMPTY_ZONES, atkPercent: 0.1, inspire: 90, atkMultiplier: 1.2 };
        const z = buildRequestZones(team, card, [src("aak_s3", "atkPercent", 0.5), src("pramanix_t1", "fragile", 0.33)]);
        expect(z?.atkPercent).toBeCloseTo(0.8, 12);
        expect(z?.inspire).toEqual([150, 90]);
        expect(z?.fragile).toEqual([0.2, 0.33]);
        expect(z?.atkMultiplier).toEqual([1.1, 1.2]);
        expect(z?.damageReduction).toEqual([0.5]);
    });

    it("skips a class-limited supporter for an operator of another class, applies it to its class and to an unknown one", () => {
        const sources = [src("suzuran_t1", "spAura", 0.4, ["SUPPORT"])];
        expect(buildRequestZones(EMPTY_ZONES, undefined, sources, "SNIPER")).toBeUndefined();
        expect(buildRequestZones(EMPTY_ZONES, undefined, sources, "SUPPORT")).toEqual({ spAura: [0.4] });
        expect(buildRequestZones(EMPTY_ZONES, undefined, sources)).toEqual({ spAura: [0.4] });
    });

    it("routes DEF / RES reduction, ignore and steal into the stat blocks", () => {
        const team = { ...EMPTY_ZONES, defIgnorePercent: 0.4, defSteal: 70, resIgnoreFlat: 10 };
        expect(buildRequestZones(team, undefined, [src("meteorite_s2", "defFlat", 150)])).toEqual({ def: { flat: 150, ignorePercent: 0.4, steal: [70] }, res: { ignoreFlat: 10 } });
    });
});

describe("resolveZone", () => {
    it("applies only the largest source of a MAX group", () => {
        const r = resolveZone("fragile", [{ origin: "team", zones: { ...EMPTY_ZONES, fragile: 0.2 } }], [src("pramanix_t1", "fragile", 0.33), src("shamare_t1", "fragile", 0.33)]);
        expect(r.stacking).toBe("max");
        expect(r.sources.map((s) => [s.origin, s.applied])).toEqual([
            ["team", false],
            ["pramanix_t1", true],
            ["shamare_t1", false],
        ]);
        expect(r.total).toBe(0.33);
    });

    it("sums additive zones and multiplies the multiplying ones", () => {
        expect(resolveZone("atkPercent", [{ origin: "team", zones: { ...EMPTY_ZONES, atkPercent: 0.3 } }], [src("a", "atkPercent", 0.2)]).total).toBeCloseTo(0.5, 12);
        expect(resolveZone("atkMultiplier", [{ origin: "team", zones: { ...EMPTY_ZONES, atkMultiplier: 1.5 } }], []).total).toBe(1.5);
        expect(
            resolveZone(
                "damageReduction",
                [
                    { origin: "team", zones: { ...EMPTY_ZONES, damageReduction: 0.5 } },
                    { origin: "card", zones: { ...EMPTY_ZONES, damageReduction: 0.5 } },
                ],
                [],
            ).total,
        ).toBe(0.75);
    });
});

describe("supporterSources", () => {
    const entry = (id: string, zone: SupporterEntry["contributions"][number]["zone"], value: number, professions: string[] = []): SupporterEntry => ({
        id,
        charId: "char_x",
        name: id,
        sourceKind: "talent",
        sourceIndex: 1,
        sourceName: id,
        contributions: [{ zone, stacking: "max", key: "k", transform: "value", raw: value, value }],
        scope: { target: "allAllies", professions },
        investment: { promotion: 2, skillLevel: 10, potential: 1, moduleLevel: 0 },
    });

    it("resolves the selected ids in selection order and skips ids the data lacks", () => {
        const entries = [entry("a", "fragile", 0.3), entry("b", "defPercent", 0.2, ["CASTER"])];
        expect(supporterSources(entries, ["b", "gone", "a"])).toEqual([src("b", "defPercent", 0.2, ["CASTER"]), src("a", "fragile", 0.3)]);
        expect(supporterSources(undefined, ["a"])).toEqual([]);
    });
});

describe("zone vector", () => {
    it("round-trips every zone in the fixed order", () => {
        const z = { ...EMPTY_ZONES };
        ZONE_KEYS.forEach((k, i) => {
            z[k] = i + 0.5;
        });
        expect(decodeZones(encodeZones(z))).toEqual(z);
        expect(ZONE_KEYS).toHaveLength(ZONE_FIELDS.length);
        expect(new Set(ZONE_KEYS).size).toBe(Object.keys(EMPTY_ZONES).length);
    });

    it("fills a shorter vector with defaults, and rejects a longer or non-numeric one", () => {
        expect(decodeZones([0.4])).toEqual({ ...EMPTY_ZONES, atkPercent: 0.4 });
        expect(decodeZones([...encodeZones(EMPTY_ZONES), 1])).toBeNull();
        expect(decodeZones([0, "x"])).toBeNull();
        expect(decodeZones([0, Number.NaN])).toBeNull();
        expect(decodeZones("x")).toBeNull();
    });

    it("never yields a final ATK factor of 0 or below, which the backend rejects", () => {
        const at = ZONE_KEYS.indexOf("atkMultiplier");
        const v = encodeZones(EMPTY_ZONES);
        v[at] = 0;
        expect(decodeZones(v)?.atkMultiplier).toBe(1);
        expect(normaliseZones({ atkMultiplier: -2 })?.atkMultiplier).toBe(1);
    });
});

describe("request bounds", () => {
    it("clamps a summed DEF / RES percent to the backend's [-1, 1] instead of sending a 400", () => {
        const team = { ...EMPTY_ZONES, defPercent: 1, resPercent: 0.9, defIgnorePercent: 0.8 };
        const z = buildRequestZones(team, { ...EMPTY_ZONES, defIgnorePercent: 0.5 }, [src("pramanix_s2", "defPercent", 0.3), src("eyja_s2", "resPercent", 0.25)]);
        expect(z?.def).toEqual({ percent: 1, ignorePercent: 1 });
        expect(z?.res).toEqual({ percent: 1 });
    });

    it("keeps every additive sum and list value inside its REQUEST_BOUNDS entry", () => {
        const big = { ...EMPTY_ZONES };
        for (const k of ZONE_KEYS) big[k] = 1e9;
        const z = buildRequestZones(big, big, []);
        for (const k of ZONE_KEYS) expect(REQUEST_BOUNDS[k]).toBeDefined();
        expect(z?.atkPercent).toBe(100);
        expect(z?.aspd).toBe(1000);
        expect(z?.inspire).toEqual([100_000, 100_000]);
        expect(z?.damageReduction).toEqual([1, 1]);
        expect(z?.res?.steal).toEqual([10_000, 10_000]);
        expect(z?.def?.ignorePercent).toBe(1);
    });

    it("sends at most 16 sources to a MAX group, keeping the largest", () => {
        const sources = Array.from({ length: 20 }, (_, i) => src(`s${i}`, "fragile", (i + 1) / 100));
        const z = buildRequestZones(EMPTY_ZONES, undefined, sources);
        expect(z?.fragile).toHaveLength(MAX_ZONE_SOURCES);
        expect(Math.max(...(z?.fragile ?? []))).toBe(0.2);
        expect(Math.min(...(z?.fragile ?? []))).toBe(0.05);
    });

    it("gives each source a unique id when one supporter feeds a zone twice", () => {
        const r = resolveZone("defPercent", [], [src("x", "defPercent", 0.1), src("x", "defPercent", 0.2)]);
        expect(r.sources.map((s) => s.id)).toEqual(["x", "x#1"]);
    });
});
