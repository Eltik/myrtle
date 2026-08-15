import { CalendarView } from "frontend";

// The calendar surface: toolbar + month grid, or toolbar + agenda columns.
// `scale`/`anchor` are controlled by the page, so each scale is its own story.
type Row = [string, string, number, string, string, number];

const toBirthday = ([id, name, rarity, profession, nationId, day]: Row) => ({
    operator: { id, name, rarity: `TIER_${rarity}`, profession, nationId },
    known: true,
    raw: `May ${day}`,
    month: 5,
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
    ["char_1026_gvial2", "Gavial the Invincible", 6, "WARRIOR", "rhodes", 1],
    ["char_193_frostl", "Frostleaf", 4, "WARRIOR", "rhodes", 1],
    ["char_126_shotst", "Meteor", 4, "SNIPER", "kazimierz", 1],
    ["char_301_cutter", "Cutter", 4, "WARRIOR", "columbia", 2],
    ["char_343_tknogi", "Tsukinogi", 5, "SUPPORT", "higashi", 3],
    ["char_179_cgbird", "Nightingale", 6, "MEDIC", "rhodes", 4],
    ["char_458_rfrost", "Frost", 5, "SPECIAL", "lungmen", 4],
    ["char_188_helage", "Hellagur", 6, "WARRIOR", "ursus", 5],
    ["char_117_myrrh", "Myrrh", 4, "MEDIC", "rhodes", 5],
    ["char_493_firwhl", "Firewhistle", 5, "TANK", "rim", 6],
    ["char_4048_doroth", "Dorothy", 6, "SPECIAL", "columbia", 8],
    ["char_401_elysm", "Elysium", 5, "PIONEER", "rhodes", 8],
    ["char_346_aosta", "Aosta", 5, "SNIPER", "siracusa", 9],
    ["char_484_robrta", "Roberta", 4, "SUPPORT", "columbia", 10],
    ["char_297_hamoni", "Harmonie", 5, "CASTER", "victoria", 13],
    ["char_291_aglina", "Angelina", 6, "SUPPORT", "siracusa", 14],
    ["char_4082_qiubai", "Qiubai", 6, "WARRIOR", "yan", 14],
    ["char_1029_yato2", "Kirin R Yato", 6, "SPECIAL", "rhodes", 14],
    ["char_154_morgan", "Morgan", 5, "WARRIOR", "victoria", 14],
    ["char_502_nblade", "Yato", 2, "PIONEER", "rhodes", 14],
    ["char_486_takila", "Tequila", 5, "WARRIOR", "bolivar", 15],
    ["char_1031_slent2", "Silence the Paradigmatic", 6, "SUPPORT", "columbia", 18],
    ["char_107_liskam", "Liskarm", 5, "TANK", "columbia", 18],
    ["char_199_yak", "Matterhorn", 4, "TANK", "kjerag", 19],
    ["char_283_midn", "Midnight", 3, "WARRIOR", "rhodes", 20],
    ["char_4193_lemuen", "Lemuen", 6, "SNIPER", "laterano", 21],
    ["char_4138_narant", "Narantuya", 6, "SNIPER", "sargon", 26],
    ["char_130_doberm", "Dobermann", 4, "WARRIOR", "rhodes", 27],
    ["char_4148_philae", "Philae", 5, "TANK", "sargon", 29],
    ["char_211_adnach", "Adnachiel", 3, "SNIPER", "rhodes", 30],
    ["char_479_sleach", "Saileach", 6, "PIONEER", "victoria", 31],
];

const BY_DAY = groupByDay(MAY);
const TODAY = new Date(2024, 4, 15);
const noop = () => {};

export const MonthView = () => (
    <div className="max-w-2xl">
        <CalendarView scale="month" onScaleChange={noop} anchor={new Date(2024, 4, 1)} onAnchorChange={noop} byDay={BY_DAY} onSelect={noop} today={TODAY} />
    </div>
);

export const ThreeDayAgenda = () => <CalendarView scale="3day" onScaleChange={noop} anchor={new Date(2024, 4, 13)} onAnchorChange={noop} byDay={BY_DAY} onSelect={noop} today={TODAY} />;

export const DayAgenda = () => (
    <div className="max-w-2xl">
        <CalendarView scale="day" onScaleChange={noop} anchor={new Date(2024, 4, 14)} onAnchorChange={noop} byDay={BY_DAY} onSelect={noop} today={TODAY} />
    </div>
);
