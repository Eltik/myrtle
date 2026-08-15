import { ListView } from "frontend";

// Chronological list; months are ordered starting from `today`'s month, so with
// the harness clock at 2024-05-15 this opens on May and runs forward.
type Row = [string, string, number, string, string, number, number];

const toBirthday = ([id, name, rarity, profession, nationId, month, day]: Row) => ({
    operator: { id, name, rarity: `TIER_${rarity}`, profession, nationId },
    known: true,
    raw: `${["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul"][month - 1]}. ${day}`,
    month,
    day,
});

const ITEMS: Row[] = [
    ["char_291_aglina", "Angelina", 6, "SUPPORT", "siracusa", 5, 14],
    ["char_154_morgan", "Morgan", 5, "WARRIOR", "victoria", 5, 14],
    ["char_486_takila", "Tequila", 5, "WARRIOR", "bolivar", 5, 15],
    ["char_1031_slent2", "Silence the Paradigmatic", 6, "SUPPORT", "columbia", 5, 18],
    ["char_107_liskam", "Liskarm", 5, "TANK", "columbia", 5, 18],
    ["char_102_texas", "Texas", 5, "PIONEER", "lungmen", 6, 1],
    ["char_4116_blkkgt", "Degenbrecher", 6, "WARRIOR", "kjerag", 6, 6],
    ["char_010_chen", "Ch'en", 6, "WARRIOR", "lungmen", 7, 7],
];

const TODAY = new Date(2024, 4, 15);

export const ByMonth = () => (
    <div className="max-w-2xl">
        <ListView items={ITEMS.map(toBirthday)} today={TODAY} />
    </div>
);

export const SingleMonth = () => (
    <div className="max-w-2xl">
        <ListView items={ITEMS.filter((r) => r[5] === 5).map(toBirthday)} today={TODAY} />
    </div>
);

export const NoResults = () => (
    <div className="max-w-2xl">
        <ListView items={[]} today={TODAY} />
    </div>
);
