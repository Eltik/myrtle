import { TopOperators } from "frontend";

// ---------------------------------------------------------------------------
// Fixture: one doctor's stored pull history, as /gacha/records serves it.
// 512 pulls across four banner buckets; timestamps land just before the
// capture clock's 2024-05-15T12:00Z so pity and "last pull" read correctly.
// ---------------------------------------------------------------------------
const secs = (y: number, m: number, d: number) => Math.floor(Date.UTC(y, m - 1, d) / 1000);
const tsOf = (y: number, m: number, d: number, h: number) => Date.UTC(y, m - 1, d, h);

type Op = [string, string, string];

const SIX: Op[] = [
    ["char_245_cello", "Virtuosa", "SUPPORT"],
    ["char_4116_blkkgt", "Degenbrecher", "WARRIOR"],
    ["char_4087_ines", "Ines", "PIONEER"],
    ["char_4064_mlynar", "Młynar", "WARRIOR"],
    ["char_2012_typhon", "Typhon", "SNIPER"],
    ["char_4117_ray", "Ray", "SNIPER"],
    ["char_2013_cerber", "Ceobe", "CASTER"],
    ["char_358_lisa", "Suzuran", "SUPPORT"],
    ["char_180_amgoat", "Eyjafjalla", "CASTER"],
];
const FIVE: Op[] = [
    ["char_143_ghost", "Specter", "WARRIOR"],
    ["char_140_whitew", "Lappland", "WARRIOR"],
    ["char_102_texas", "Texas", "PIONEER"],
    ["char_128_plosis", "Ptilopsis", "MEDIC"],
    ["char_219_meteo", "Meteorite", "SNIPER"],
    ["char_144_red", "Projekt Red", "SPECIAL"],
    ["char_436_whispr", "Whisperain", "MEDIC"],
    ["char_145_prove", "Provence", "SNIPER"],
    ["char_107_liskam", "Liskarm", "TANK"],
    ["char_106_franka", "Franka", "WARRIOR"],
    ["char_101_sora", "Sora", "SUPPORT"],
    ["char_129_bluep", "Blue Poison", "SNIPER"],
];
const FOUR: Op[] = [
    ["char_151_myrtle", "Myrtle", "PIONEER"],
    ["char_196_sunbr", "Gummy", "TANK"],
    ["char_181_flower", "Perfumer", "MEDIC"],
    ["char_328_cammou", "Click", "CASTER"],
    ["char_385_finlpp", "Purestream", "MEDIC"],
    ["char_118_yuki", "Shirayuki", "SNIPER"],
    ["char_289_gyuki", "Matoimaru", "WARRIOR"],
    ["char_258_podego", "Podenco", "SUPPORT"],
    ["char_337_utage", "Utage", "WARRIOR"],
    ["char_149_scave", "Scavenger", "PIONEER"],
    ["char_109_fmout", "Gitano", "CASTER"],
    ["char_183_skgoat", "Earthspirit", "SUPPORT"],
    ["char_110_deepcl", "Deepcolor", "SUPPORT"],
    ["char_491_humus", "Humus", "WARRIOR"],
];
const THREE: Op[] = [
    ["char_124_kroos", "Kroos", "SNIPER"],
    ["char_209_ardign", "Cardigan", "TANK"],
    ["char_123_fang", "Fang", "PIONEER"],
    ["char_208_melan", "Melantha", "WARRIOR"],
    ["char_240_wyvern", "Vanilla", "PIONEER"],
    ["char_210_stward", "Steward", "CASTER"],
    ["char_121_lava", "Lava", "CASTER"],
    ["char_120_hibisc", "Hibiscus", "MEDIC"],
    ["char_212_ansel", "Ansel", "MEDIC"],
    ["char_192_falco", "Plume", "PIONEER"],
    ["char_282_catap", "Catapult", "SNIPER"],
    ["char_122_beagle", "Beagle", "TANK"],
    ["char_211_adnach", "Adnachiel", "SNIPER"],
    ["char_278_orchid", "Orchid", "SUPPORT"],
    ["char_283_midn", "Midnight", "WARRIOR"],
    ["char_284_spot", "Spot", "TANK"],
];

const OPERATORS_BY_ID = new Map([...SIX, ...FIVE, ...FOUR, ...THREE].map(([id, name, profession]) => [id, { id, name, profession }]));

const byId = (id: string) => (SIX.find((o) => o[0] === id) ?? SIX[0]) as Op;

let seed = 20240515;
const rnd = () => {
    seed = (seed * 1103515245 + 12345) % 2147483648;
    return seed / 2147483648;
};

type Pool = [string, string, string];

const pull = (star: number, op: Op, pool: Pool, t: number) => ({
    charId: op[0],
    charName: op[1],
    star: String(star),
    color: "",
    poolId: pool[0],
    poolName: pool[1],
    typeName: pool[2],
    at: t,
    atStr: new Date(t).toISOString().replace("T", " ").slice(0, 19),
});

/** `sixes` maps a 0-based position in the run to the 6★ that dropped there. */
function run(pool: Pool, count: number, sixes: Record<number, string>, start: number, stepMs: number) {
    const out = [];
    for (let i = 0; i < count; i++) {
        const t = start + i * stepMs;
        const six = sixes[i];
        if (six) {
            out.push(pull(6, byId(six), pool, t));
            continue;
        }
        const r = rnd();
        if (r < 0.093) out.push(pull(5, FIVE[Math.floor(rnd() * FIVE.length)], pool, t));
        else if (r < 0.56) out.push(pull(4, FOUR[Math.floor(rnd() * FOUR.length)], pool, t));
        else out.push(pull(3, THREE[Math.floor(rnd() * THREE.length)], pool, t));
    }
    return out;
}

const P_LIMITED: Pool = ["LIMITED_EN_27_0_3", "By My Will", "limited"];
const P_LINKAGE: Pool = ["LINKAGE_EN_23_1_1", "Sharpened By Flame", "linkage"];
const P_SINGLE: Pool = ["SINGLE_EN_27_0_1", "From Gleams And Smoke I Emerge", "single"];
const P_NORM: Pool = ["NORM_EN_27_0_4", "Rare Operators useful in all kinds of stages", "normal"];
const P_CLASSIC: Pool = ["CLASSIC_EN_27_0_2", "Rare Operators useful in all kinds of stages", "classic"];

const R_LIMITED = run(P_LIMITED, 148, { 22: "char_245_cello", 61: "char_245_cello", 99: "char_4116_blkkgt", 132: "char_245_cello" }, tsOf(2024, 4, 30, 13), 7_500_000);
const R_LINKAGE = run(P_LINKAGE, 24, {}, tsOf(2023, 9, 8, 20), 1_800_000);
const R_SINGLE = run(P_SINGLE, 84, { 30: "char_4087_ines", 71: "char_4064_mlynar" }, tsOf(2024, 4, 16, 11), 13_400_000);
const R_NORM = run(P_NORM, 126, { 12: "char_2012_typhon", 40: "char_4117_ray", 63: "char_4064_mlynar" }, tsOf(2024, 5, 10, 9), 3_430_000);
const R_CLASSIC = run(P_CLASSIC, 130, { 11: "char_2013_cerber", 34: "char_358_lisa", 53: "char_180_amgoat" }, tsOf(2024, 5, 7, 10), 4_650_000);

const RECORDS = {
    limited: { gacha_type: "limited", records: R_LIMITED, total: R_LIMITED.length },
    linkage: { gacha_type: "linkage", records: R_LINKAGE, total: R_LINKAGE.length },
    regular: { gacha_type: "regular", records: [...R_SINGLE, ...R_NORM], total: R_SINGLE.length + R_NORM.length },
    special: { gacha_type: "special", records: R_CLASSIC, total: R_CLASSIC.length },
};

const EMPTY_RECORDS = {
    limited: { gacha_type: "limited", records: [], total: 0 },
    linkage: { gacha_type: "linkage", records: [], total: 0 },
    regular: { gacha_type: "regular", records: [], total: 0 },
    special: { gacha_type: "special", records: [], total: 0 },
};

const banner = (gachaPoolId: string, gachaPoolName: string, gachaRuleType: string, o: number[], e: number[], featured6: string[] = [], featured5: string[] = []) => ({
    gachaPoolId,
    gachaPoolName,
    gachaRuleType,
    gachaIndex: 0,
    openTime: secs(o[0], o[1], o[2]),
    endTime: secs(e[0], e[1], e[2]),
    gachaPoolSummary: "-",
    gachaPoolDetail: null,
    guarantee5Avail: 1,
    guarantee5Count: 10,
    guaranteeName: null,
    featured6,
    featured5,
});

const BANNERS_BY_ID = new Map(
    [
        banner("LIMITED_EN_27_0_3", "By My Will", "LIMITED", [2024, 4, 30], [2024, 5, 14], ["char_245_cello"]),
        banner("LINKAGE_EN_23_1_1", "Sharpened By Flame", "LINKAGE", [2023, 9, 7], [2023, 9, 21]),
        banner("SINGLE_EN_27_0_1", "From Gleams And Smoke I Emerge", "SINGLE", [2024, 4, 16], [2024, 4, 30]),
        banner("NORM_EN_27_0_4", "Rare Operators useful in all kinds of stages", "NORMAL", [2024, 5, 10], [2024, 5, 24]),
        banner("CLASSIC_EN_27_0_2", "Rare Operators useful in all kinds of stages", "CLASSIC", [2024, 5, 7], [2024, 5, 21], ["char_2013_cerber", "char_358_lisa"], ["char_143_ghost", "char_140_whitew", "char_349_chiave"]),
    ].map((b) => [b.gachaPoolId, b]),
);

export const Default = () => <TopOperators records={RECORDS} operatorsById={OPERATORS_BY_ID} isLoading={false} />;

export const Loading = () => <TopOperators records={null} operatorsById={OPERATORS_BY_ID} isLoading={true} />;

/** The 6★ tab is selected by default, so an empty history lands on the per-rarity empty branch. */
export const NoPullsYet = () => <TopOperators records={EMPTY_RECORDS} operatorsById={OPERATORS_BY_ID} isLoading={false} />;
