import { Card, MonthGrid } from "frontend";

// Real May/June birthdays out of the game's handbook data. `byDay` is keyed
// "month-day" exactly as `groupByDay` builds it, each bucket rarity-desc.
type Row = [string, string, number, string, string, number, number];

const toBirthday = ([id, name, rarity, profession, nationId, month, day]: Row) => ({
    operator: { id, name, rarity: `TIER_${rarity}`, profession, nationId },
    known: true,
    raw: `${["Jan", "Feb", "Mar", "Apr", "May", "Jun"][month - 1]}. ${day}`,
    month,
    day,
});

const groupByDay = (rows: Row[]) => {
    const map = new Map<string, ReturnType<typeof toBirthday>[]>();
    for (const row of rows) {
        const b = toBirthday(row);
        const key = `${b.month}-${b.day}`;
        const bucket = map.get(key);
        if (bucket) bucket.push(b);
        else map.set(key, [b]);
    }
    return map;
};

const MAY: Row[] = [
    ["char_1026_gvial2", "Gavial the Invincible", 6, "WARRIOR", "rhodes", 5, 1],
    ["char_193_frostl", "Frostleaf", 4, "WARRIOR", "rhodes", 5, 1],
    ["char_126_shotst", "Meteor", 4, "SNIPER", "kazimierz", 5, 1],
    ["char_301_cutter", "Cutter", 4, "WARRIOR", "columbia", 5, 2],
    ["char_343_tknogi", "Tsukinogi", 5, "SUPPORT", "higashi", 5, 3],
    ["char_4052_surfer", "Surfer", 5, "PIONEER", "columbia", 5, 3],
    ["char_179_cgbird", "Nightingale", 6, "MEDIC", "rhodes", 5, 4],
    ["char_458_rfrost", "Frost", 5, "SPECIAL", "lungmen", 5, 4],
    ["char_188_helage", "Hellagur", 6, "WARRIOR", "ursus", 5, 5],
    ["char_117_myrrh", "Myrrh", 4, "MEDIC", "rhodes", 5, 5],
    ["char_493_firwhl", "Firewhistle", 5, "TANK", "rim", 5, 6],
    ["char_4119_wanqin", "Wanqing", 5, "PIONEER", "yan", 5, 6],
    ["char_4048_doroth", "Dorothy", 6, "SPECIAL", "columbia", 5, 8],
    ["char_401_elysm", "Elysium", 5, "PIONEER", "rhodes", 5, 8],
    ["char_346_aosta", "Aosta", 5, "SNIPER", "siracusa", 5, 9],
    ["char_328_cammou", "Click", 4, "CASTER", "rhodes", 5, 9],
    ["char_484_robrta", "Roberta", 4, "SUPPORT", "columbia", 5, 10],
    ["char_297_hamoni", "Harmonie", 5, "CASTER", "victoria", 5, 13],
    ["char_291_aglina", "Angelina", 6, "SUPPORT", "siracusa", 5, 14],
    ["char_4082_qiubai", "Qiubai", 6, "WARRIOR", "yan", 5, 14],
    ["char_1029_yato2", "Kirin R Yato", 6, "SPECIAL", "rhodes", 5, 14],
    ["char_154_morgan", "Morgan", 5, "WARRIOR", "victoria", 5, 14],
    ["char_494_vendla", "Vendela", 5, "MEDIC", "victoria", 5, 14],
    ["char_502_nblade", "Yato", 2, "PIONEER", "rhodes", 5, 14],
    ["char_486_takila", "Tequila", 5, "WARRIOR", "bolivar", 5, 15],
    ["char_1031_slent2", "Silence the Paradigmatic", 6, "SUPPORT", "columbia", 5, 18],
    ["char_108_silent", "Silence", 5, "MEDIC", "columbia", 5, 18],
    ["char_107_liskam", "Liskarm", 5, "TANK", "columbia", 5, 18],
    ["char_199_yak", "Matterhorn", 4, "TANK", "kjerag", 5, 19],
    ["char_283_midn", "Midnight", 3, "WARRIOR", "rhodes", 5, 20],
    ["char_4193_lemuen", "Lemuen", 6, "SNIPER", "laterano", 5, 21],
    ["char_4122_grabds", "Grain Buds", 5, "SUPPORT", "yan", 5, 21],
    ["char_4138_narant", "Narantuya", 6, "SNIPER", "sargon", 5, 26],
    ["char_130_doberm", "Dobermann", 4, "WARRIOR", "rhodes", 5, 27],
    ["char_4148_philae", "Philae", 5, "TANK", "sargon", 5, 29],
    ["char_211_adnach", "Adnachiel", 3, "SNIPER", "rhodes", 5, 30],
    ["char_479_sleach", "Saileach", 6, "PIONEER", "victoria", 5, 31],
];

const SIX_STARS_ONLY: Row[] = MAY.filter((r) => r[2] === 6);

const TODAY = new Date(2024, 4, 15);
const noop = () => {};

export const May2024 = () => (
    <Card className="max-w-2xl overflow-hidden">
        <MonthGrid anchor={new Date(2024, 4, 1)} byDay={groupByDay(MAY)} onSelect={noop} today={TODAY} />
    </Card>
);

export const FilteredToSixStars = () => (
    <Card className="max-w-2xl overflow-hidden">
        <MonthGrid anchor={new Date(2024, 4, 1)} byDay={groupByDay(SIX_STARS_ONLY)} onSelect={noop} today={TODAY} />
    </Card>
);

export const NoMatches = () => (
    <Card className="max-w-2xl overflow-hidden">
        <MonthGrid anchor={new Date(2024, 5, 1)} byDay={new Map()} onSelect={noop} today={TODAY} />
    </Card>
);
