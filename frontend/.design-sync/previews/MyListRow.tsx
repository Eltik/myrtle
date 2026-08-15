import { MyListRow } from "frontend";

/** `IOperator` — every id verified against https://api.myrtle.moe/api/operators/index. */
const op = (id: string, name: string, rarity: number, role: string, arch: string) => ({ id, name, rarity, role, arch });

const TIERS = [
    {
        name: "S",
        color: "#dc4d56",
        operators: [
            op("char_1035_wisdel", "Wiš'adel", 6, "SNIPER", "bombarder"),
            op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator"),
            op("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor"),
            op("char_4087_ines", "Ines", 6, "PIONEER", "agent"),
        ],
    },
    {
        name: "A",
        color: "#e0603c",
        operators: [
            op("char_350_surtr", "Surtr", 6, "WARRIOR", "artsfghter"),
            op("char_4116_blkkgt", "Degenbrecher", 6, "WARRIOR", "sword"),
            op("char_2012_typhon", "Typhon", 6, "SNIPER", "siegesniper"),
            op("char_377_gdglow", "Goldenglow", 6, "CASTER", "funnel"),
        ],
    },
    {
        name: "B",
        color: "#c9a227",
        operators: [op("char_103_angel", "Exusiai", 6, "SNIPER", "fastshot"), op("char_180_amgoat", "Eyjafjalla", 6, "CASTER", "corecaster"), op("char_293_thorns", "Thorns", 6, "WARRIOR", "lord")],
    },
];

/** `ITierListBrowseItem` */
const base = {
    id: "tl-1",
    slug: "endgame-dps-rankings",
    title: "Endgame DPS rankings",
    description: "Single-target and burst damage ranked for CC risk 18+ and IS#5 Ashring. Assumes E2 90 and module stage 3.",
    tag: "Meta",
    stage: "CC#12 Daybreak",
    author: { name: "Dr. Kal'tsit", avatarId: "char_003_kalts" },
    updated: "2 days ago",
    votes: 412,
    views: 48213,
    comments: 37,
    accent: "coral",
    listType: "community",
    createdAtMs: 1707000000000,
    updatedAtMs: 1715600000000,
    flairCode: "meta",
    flairLabel: "Meta",
    flairColor: "#dc4d56",
    favorites: 1264,
    shares: 88,
    views24h: 1042,
    views7d: 6210,
    trendingScore: 91.4,
    isTrending: true,
    tiers: TIERS,
};

const officialList = {
    ...base,
    id: "tl-2",
    slug: "official-operator-rankings",
    title: "Myrtle official operator rankings",
    description: "Maintained by the tier-list editors. Updated every major banner.",
    listType: "official",
    flairCode: null,
    flairLabel: null,
    flairColor: null,
    updated: "6 hours ago",
    views: 312884,
    favorites: 9317,
};

const emptyDraft = {
    ...base,
    id: "tl-3",
    slug: "is5-ashring-starter-picks",
    title: "IS#5 Ashring starter picks",
    description: "",
    flairCode: "roguelike",
    flairLabel: "Integrated Strategies",
    flairColor: "#8b6ad6",
    updated: "just now",
    views: 3,
    favorites: 0,
    tiers: [{ name: "S", color: "#dc4d56", operators: [] }],
};

const noop = () => {};

export const Default = () => <MyListRow tl={base} onEdit={noop} onDelete={noop} onCopyLink={noop} />;

export const OfficialList = () => <MyListRow tl={officialList} onEdit={noop} onDelete={noop} onCopyLink={noop} />;

export const EmptyDraft = () => <MyListRow tl={emptyDraft} onEdit={noop} onDelete={noop} onCopyLink={noop} />;

export const ListView = () => (
    <div className="flex flex-col gap-2">
        <MyListRow tl={base} onEdit={noop} onDelete={noop} onCopyLink={noop} />
        <MyListRow tl={officialList} onEdit={noop} onDelete={noop} onCopyLink={noop} />
        <MyListRow tl={emptyDraft} onEdit={noop} onDelete={noop} onCopyLink={noop} />
    </div>
);
