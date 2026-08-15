import { MyListCard } from "frontend";
import type { ReactNode } from "react";

/** `IOperator` — every id verified against https://api.myrtle.moe/api/operators/index. */
const op = (id: string, name: string, rarity: number, role: string, arch: string) => ({ id, name, rarity, role, arch });

const S = [
    op("char_1035_wisdel", "Wiš'adel", 6, "SNIPER", "bombarder"),
    op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator"),
    op("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor"),
    op("char_4087_ines", "Ines", 6, "PIONEER", "agent"),
    op("char_2012_typhon", "Typhon", 6, "SNIPER", "siegesniper"),
];
const A = [
    op("char_350_surtr", "Surtr", 6, "WARRIOR", "artsfghter"),
    op("char_4116_blkkgt", "Degenbrecher", 6, "WARRIOR", "sword"),
    op("char_377_gdglow", "Goldenglow", 6, "CASTER", "funnel"),
    op("char_003_kalts", "Kal'tsit", 6, "MEDIC", "physician"),
    op("char_4133_logos", "Logos", 6, "CASTER", "corecaster"),
    op("char_2023_ling", "Ling", 6, "SUPPORT", "summoner"),
];
const B = [
    op("char_103_angel", "Exusiai", 6, "SNIPER", "fastshot"),
    op("char_180_amgoat", "Eyjafjalla", 6, "CASTER", "corecaster"),
    op("char_293_thorns", "Thorns", 6, "WARRIOR", "lord"),
    op("char_179_cgbird", "Nightingale", 6, "MEDIC", "ringhealer"),
    op("char_311_mudrok", "Mudrock", 6, "TANK", "unyield"),
];
const C = [
    op("char_263_skadi", "Skadi", 6, "WARRIOR", "fearless"),
    op("char_017_huang", "Blaze", 6, "WARRIOR", "centurion"),
    op("char_140_whitew", "Lappland", 5, "WARRIOR", "lord"),
    op("char_143_ghost", "Specter", 5, "WARRIOR", "centurion"),
];
const D = [
    op("char_102_texas", "Texas", 5, "PIONEER", "pioneer"),
    op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer"),
    op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer"),
    op("char_199_yak", "Matterhorn", 4, "TANK", "protector"),
];

const E = [
    op("char_358_lisa", "Suzuran", 6, "SUPPORT", "slower"),
    op("char_222_bpipe", "Bagpipe", 6, "PIONEER", "charger"),
    op("char_202_demkni", "Saria", 6, "TANK", "guardian"),
];

const TIERS = [
    { name: "S", color: "#dc4d56", operators: S },
    { name: "A", color: "#e0603c", operators: A },
    { name: "B", color: "#c9a227", operators: B },
    { name: "C", color: "#4f9d69", operators: C },
    { name: "D", color: "#5a7fb8", operators: D },
    { name: "E", color: "#7a6f8c", operators: E },
];

/** `ITierListBrowseItem` */
const base = {
    id: "tl-1",
    slug: "endgame-dps-rankings",
    title: "Endgame DPS rankings",
    description: "Single-target and burst damage ranked for CC risk 18+ and IS#5 Ashring.",
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

const noop = () => {};

/** The `my` grid is a 3-column layout at desktop; one column is 320px wide. */
const GridCell = ({ children }: { children: ReactNode }) => <div className="w-80">{children}</div>;

export const Default = () => (
    <GridCell>
        <MyListCard tl={base} onEdit={noop} onDelete={noop} onCopyLink={noop} />
    </GridCell>
);

export const OfficialList = () => (
    <GridCell>
        <MyListCard
            tl={{
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
                views24h: 8420,
                tiers: TIERS.slice(0, 3),
            }}
            onEdit={noop}
            onDelete={noop}
            onCopyLink={noop}
        />
    </GridCell>
);

export const EmptyDraft = () => (
    <GridCell>
        <MyListCard
            tl={{
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
                views24h: 0,
                tiers: [
                    { name: "S", color: "#dc4d56", operators: [] },
                    { name: "A", color: "#e0603c", operators: [] },
                    { name: "B", color: "#c9a227", operators: [] },
                ],
            }}
            onEdit={noop}
            onDelete={noop}
            onCopyLink={noop}
        />
    </GridCell>
);
