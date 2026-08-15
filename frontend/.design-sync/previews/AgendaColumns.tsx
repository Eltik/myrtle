import { AgendaColumns, Card } from "frontend";

// Agenda scales for the week of 2024-05-15 (the harness clock's "today").
// Day renders a full inline list; 3 Day / Week render clickable day columns.
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

const WEEK: Row[] = [
    ["char_297_hamoni", "Harmonie", 5, "CASTER", "victoria", 13],
    ["char_291_aglina", "Angelina", 6, "SUPPORT", "siracusa", 14],
    ["char_4082_qiubai", "Qiubai", 6, "WARRIOR", "yan", 14],
    ["char_1029_yato2", "Kirin R Yato", 6, "SPECIAL", "rhodes", 14],
    ["char_154_morgan", "Morgan", 5, "WARRIOR", "victoria", 14],
    ["char_494_vendla", "Vendela", 5, "MEDIC", "victoria", 14],
    ["char_502_nblade", "Yato", 2, "PIONEER", "rhodes", 14],
    ["char_486_takila", "Tequila", 5, "WARRIOR", "bolivar", 15],
    ["char_1031_slent2", "Silence the Paradigmatic", 6, "SUPPORT", "columbia", 18],
    ["char_108_silent", "Silence", 5, "MEDIC", "columbia", 18],
    ["char_107_liskam", "Liskarm", 5, "TANK", "columbia", 18],
];

// The week card is the tall one, so its story uses the 6★-filtered calendar -
// what you get with the rarity chip on.
const SIX_STARS = WEEK.filter((r) => r[2] === 6);

const TODAY = new Date(2024, 4, 15);
const day = (d: number) => new Date(2024, 4, d);
const noop = () => {};

export const WeekColumns = () => (
    <Card className="overflow-hidden">
        <AgendaColumns scale="week" days={[12, 13, 14, 15, 16, 17, 18].map(day)} byDay={groupByDay(SIX_STARS)} today={TODAY} onSelect={noop} />
    </Card>
);

export const ThreeDayColumns = () => (
    <Card className="overflow-hidden">
        <AgendaColumns scale="3day" days={[13, 14, 15].map(day)} byDay={groupByDay(WEEK)} today={TODAY} onSelect={noop} />
    </Card>
);

export const SingleDayList = () => (
    <Card className="max-w-2xl overflow-hidden">
        <AgendaColumns scale="day" days={[day(14)]} byDay={groupByDay(WEEK)} today={TODAY} onSelect={noop} />
    </Card>
);

export const EmptyDay = () => (
    <Card className="max-w-2xl overflow-hidden">
        <AgendaColumns scale="day" days={[day(16)]} byDay={groupByDay(WEEK)} today={TODAY} onSelect={noop} />
    </Card>
);
