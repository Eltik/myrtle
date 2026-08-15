import { TierListActions } from "frontend";

const detail = {
    id: "tl-global-meta",
    slug: "global-6-star-meta",
    title: "Global 6★ meta — May 2024",
    description: "The team's standing ranking of every 6★ operator on Global.",
    listType: "official" as const,
    createdBy: "u-myrtle",
    isListed: true,
    flair: { id: 1, code: "meta", label: "Meta", color: "#5aa9d9", displayOrder: 1, isActive: true },
    author: { id: "u-myrtle", uid: "myrtle", nickname: "myrtle.moe", avatarId: "char_151_myrtle" },
    stats: {
        viewCount: 214800,
        uniqueViewCount: 98400,
        favoriteCount: 6120,
        shareCount: 1240,
        isTrending: true,
        trendingScore: 88.6,
        viewsLast24h: 3180,
        viewsLast7d: 22400,
        lastViewedAt: "2024-05-15T11:40:00.000Z",
        statsUpdatedAt: "2024-05-15T12:00:00.000Z",
    },
    tiers: [],
    createdAt: "2024-01-18T10:00:00.000Z",
    updatedAt: "2024-05-14T08:30:00.000Z",
};

const draft = {
    ...detail,
    id: "tl-draft",
    slug: "chapter-14-draft",
    title: "Chapter 14 — Roaring Flare draft",
    listType: "community" as const,
    isListed: false,
    flair: null,
    stats: null,
};

export const ActionRow = () => <TierListActions detail={detail} />;

export const InHeroHeader = () => (
    <div className="rounded-2xl border border-border bg-card/60 p-5">
        <div className="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
            <div className="min-w-0 flex-1">
                <p className="m-0 font-bold font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.18em]">Tier List</p>
                <h2 className="m-0 mt-2 font-bold font-sans text-2xl text-foreground leading-tight tracking-tight">Global 6★ meta — May 2024</h2>
                <p className="mt-2 font-sans text-[12.5px] text-muted-foreground">Updated yesterday · 214.8k views · 6.1k favorites</p>
            </div>
            <TierListActions detail={detail} />
        </div>
    </div>
);

export const UnlistedDraft = () => (
    <div className="flex flex-col gap-3">
        <p className="m-0 font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Unlisted draft — share still copies the private link</p>
        <TierListActions detail={draft} />
    </div>
);
