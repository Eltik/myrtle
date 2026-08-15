import { UpcomingView } from "frontend";

// "Upcoming" counts forward from `today` (2024-05-15 under the harness clock),
// groups by date and shows the next 24 dates soonest-first.
type Row = [string, string, number, string, string, number, number];

const toBirthday = ([id, name, rarity, profession, nationId, month, day]: Row) => ({
    operator: { id, name, rarity: `TIER_${rarity}`, profession, nationId },
    known: true,
    raw: `${["Jan", "Feb", "Mar", "Apr", "May", "Jun"][month - 1]}. ${day}`,
    month,
    day,
});

const ITEMS: Row[] = [
    ["char_486_takila", "Tequila", 5, "WARRIOR", "bolivar", 5, 15],
    ["char_1031_slent2", "Silence the Paradigmatic", 6, "SUPPORT", "columbia", 5, 18],
    ["char_451_robin", "Robin", 5, "SPECIAL", "columbia", 5, 18],
    ["char_107_liskam", "Liskarm", 5, "TANK", "columbia", 5, 18],
    ["char_199_yak", "Matterhorn", 4, "TANK", "kjerag", 5, 19],
    ["char_283_midn", "Midnight", 3, "WARRIOR", "rhodes", 5, 20],
    ["char_4193_lemuen", "Lemuen", 6, "SNIPER", "laterano", 5, 21],
    ["char_4122_grabds", "Grain Buds", 5, "SUPPORT", "yan", 5, 21],
    ["char_4138_narant", "Narantuya", 6, "SNIPER", "sargon", 5, 26],
    ["char_130_doberm", "Dobermann", 4, "WARRIOR", "rhodes", 5, 27],
    ["char_479_sleach", "Saileach", 6, "PIONEER", "victoria", 5, 31],
    ["char_102_texas", "Texas", 5, "PIONEER", "lungmen", 6, 1],
    ["char_4046_ebnhlz", "Ebenholz", 6, "CASTER", "leithanien", 6, 5],
];

const TODAY = new Date(2024, 4, 15);

export const NextDates = () => <UpcomingView items={ITEMS.map(toBirthday)} today={TODAY} />;

export const SixStarsOnly = () => <UpcomingView items={ITEMS.filter((r) => r[2] === 6).map(toBirthday)} today={TODAY} />;

export const NoResults = () => (
    <div className="max-w-2xl">
        <UpcomingView items={[]} today={TODAY} />
    </div>
);
