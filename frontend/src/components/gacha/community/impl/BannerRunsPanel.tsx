import { useMemo, useState } from "react";
import { Kicker } from "#/components/ui/kicker";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import { type ClientGachaGroup, classifyBannerGroup, type IBanner, type IBannerPullStat } from "#/lib/api/gacha";
import { formatNumber, formatNumberCompact } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";

interface IBannerRunsPanelProps {
    banners: IBanner[];
    operatorsById: Map<string, IOperatorIndexEntry>;
    statsById: Map<string, IBannerPullStat>;
    isLoading: boolean;
}

const TYPE_LABELS: Record<ClientGachaGroup, string> = {
    limited: "Limited",
    linkage: "Collab",
    regular: "Standard",
    special: "Kernel",
};

const TYPE_COLORS: Record<ClientGachaGroup, string> = {
    limited: "oklch(0.85 0.18 80)",
    linkage: "oklch(0.78 0.16 320)",
    regular: "#bcabdb",
    special: "#88c8e3",
};

type Status = "active" | "upcoming" | "ended";

interface IBannerWithStatus {
    banner: IBanner;
    group: ClientGachaGroup;
    status: Status;
}

function statusOf(banner: IBanner, nowSec: number): Status {
    if (nowSec < banner.openTime) return "upcoming";
    if (nowSec > banner.endTime) return "ended";
    return "active";
}

function fmtDateRange(openSec: number, endSec: number): string {
    const fmt = (s: number) => new Date(s * 1000).toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
    return `${fmt(openSec)} → ${fmt(endSec)}`;
}

const STATUS_PILL: Record<Status, { label: string; color: string }> = {
    active: { label: "Active", color: "oklch(0.78 0.18 145)" },
    upcoming: { label: "Upcoming", color: "oklch(0.78 0.16 220)" },
    ended: { label: "Ended", color: "var(--muted-foreground)" },
};

function StatusPill({ status }: { status: Status }) {
    const meta = STATUS_PILL[status];
    return (
        <span className="inline-flex items-center gap-1.5 font-mono text-[9.5px] uppercase leading-none tracking-[0.14em]" style={{ color: meta.color }}>
            <span className="block h-1.5 w-1.5 rounded-full bg-current" aria-hidden />
            {meta.label}
        </span>
    );
}

function FeaturedAvatars({ ids, operatorsById, ringColor, cap }: { ids: string[]; operatorsById: Map<string, IOperatorIndexEntry>; ringColor: string; cap: number }) {
    if (ids.length === 0) return null;
    const visible = ids.slice(0, cap);
    const extra = ids.length - visible.length;
    return (
        <div className="flex items-center gap-1">
            {visible.map((charId) => {
                const op = operatorsById.get(charId);
                const opName = op?.name ?? charId;
                return (
                    <span key={charId} className="relative inline-flex h-7 w-7 shrink-0 overflow-hidden rounded-md" title={opName} style={{ background: "var(--muted)", boxShadow: `inset 0 0 0 1px color-mix(in oklch, ${ringColor} 60%, transparent)` }}>
                        <OperatorAvatar charId={charId} name={opName} className="block h-full w-full object-cover" />
                    </span>
                );
            })}
            {extra > 0 ? (
                <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-muted font-mono text-[10px] text-muted-foreground tabular-nums" title={`${extra} more`}>
                    +{extra}
                </span>
            ) : null}
        </div>
    );
}

function CommunityPullsLine({ stat }: { stat: IBannerPullStat | undefined }) {
    // Banner had zero community contribution (no opted-in user pulled here yet).
    // Still useful to show explicitly so users know the panel is keyed off
    // shared data rather than something we couldn't fetch.
    if (!stat || stat.pullCount === 0) {
        return <div className="font-mono text-[10.5px] text-muted-foreground/70 uppercase tracking-[0.12em]">No community pulls yet</div>;
    }
    return (
        <div className="flex items-baseline gap-2 leading-none">
            <span className="font-sans font-semibold text-[18px] text-foreground tabular-nums tracking-[-0.02em]">{formatNumber(stat.pullCount)}</span>
            <span className="font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">community pulls</span>
            <span className="ml-auto font-mono text-[10px] text-muted-foreground tabular-nums" title={`${stat.userCount.toLocaleString()} contributing doctors`}>
                {formatNumberCompact(stat.userCount)}&nbsp;ppl
            </span>
        </div>
    );
}

function BannerCard({ entry, operatorsById, stat, rank, rankTitle }: { entry: IBannerWithStatus; operatorsById: Map<string, IOperatorIndexEntry>; stat: IBannerPullStat | undefined; rank: number | null; rankTitle: string }) {
    const { banner, group, status } = entry;
    const typeColor = TYPE_COLORS[group];

    return (
        <HoverCard>
            <HoverCardTrigger
                render={
                    <div
                        className="flex cursor-default flex-col gap-2.5 rounded-xl border bg-muted/30 p-3.5 transition-colors hover:bg-muted/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40"
                        style={{ borderColor: status === "active" ? `color-mix(in oklch, ${typeColor} 50%, var(--border))` : "var(--border)" }}
                    >
                        <div className="flex items-start justify-between gap-2">
                            <div className="flex items-center gap-1.5">
                                {rank !== null ? <RankChip rank={rank} title={rankTitle} /> : null}
                                <span className="font-mono text-[9.5px] uppercase tracking-[0.14em]" style={{ color: typeColor }}>
                                    {TYPE_LABELS[group]}
                                </span>
                            </div>
                            <StatusPill status={status} />
                        </div>
                        <div className="min-w-0">
                            <div className="line-clamp-2 font-sans font-semibold text-[14px] text-foreground leading-snug">{banner.gachaPoolName}</div>
                            <div className="mt-0.5 font-mono text-[10.5px] text-muted-foreground tabular-nums">{fmtDateRange(banner.openTime, banner.endTime)}</div>
                        </div>
                        <CommunityPullsLine stat={stat} />
                        {banner.featured6.length > 0 ? <FeaturedAvatars ids={banner.featured6} operatorsById={operatorsById} ringColor="oklch(0.78 0.18 80)" cap={5} /> : null}
                    </div>
                }
            />
            <HoverCardContent className="w-80 p-4">
                <div className="flex flex-col gap-3">
                    <div className="flex flex-col gap-1">
                        <span className="font-mono text-[9.5px] uppercase tracking-[0.14em]" style={{ color: typeColor }}>
                            {TYPE_LABELS[group]} · {banner.gachaRuleType}
                        </span>
                        <span className="font-sans font-semibold text-[14px] text-foreground leading-snug">{banner.gachaPoolName}</span>
                        {banner.gachaPoolSummary && banner.gachaPoolSummary !== "-" ? <span className="font-sans text-[12px] text-muted-foreground leading-snug">{banner.gachaPoolSummary}</span> : null}
                    </div>

                    <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
                        <span className="font-sans text-[12px] text-foreground tabular-nums leading-none">{fmtDateRange(banner.openTime, banner.endTime)}</span>
                        <StatusPill status={status} />
                    </div>

                    {banner.guarantee5Avail ? (
                        <div className="font-mono text-[10.5px] text-muted-foreground">
                            {banner.guaranteeName ?? "Guaranteed 5★"} within {banner.guarantee5Count} pulls
                        </div>
                    ) : null}

                    {banner.featured6.length > 0 ? (
                        <div className="flex flex-col gap-1.5">
                            <span className="font-mono text-[9.5px] uppercase tracking-[0.14em]" style={{ color: "oklch(0.78 0.18 80)" }}>
                                Featured 6★
                            </span>
                            <FeaturedAvatars ids={banner.featured6} operatorsById={operatorsById} ringColor="oklch(0.78 0.18 80)" cap={8} />
                        </div>
                    ) : null}

                    <div className="break-all font-mono text-[10px] text-muted-foreground/70 tabular-nums">{banner.gachaPoolId}</div>
                </div>
            </HoverCardContent>
        </HoverCard>
    );
}

const FILTERS = [
    { key: "all" as const, label: "All" },
    { key: "active" as const, label: "Active" },
    { key: "limited" as const, label: "Limited" },
    { key: "linkage" as const, label: "Collab" },
    { key: "special" as const, label: "Kernel" },
];
type Filter = (typeof FILTERS)[number]["key"];

const SORTS = [
    // Default: status-grouped chronological order (active → upcoming → recently ended).
    { key: "timeline" as const, label: "Timeline" },
    // Metric sorts rank by shared community data, highest first.
    { key: "pulls" as const, label: "Pulls" },
    { key: "popularity" as const, label: "Popularity" },
];
type Sort = (typeof SORTS)[number]["key"];

const SORT_METRIC_LABEL: Record<Exclude<Sort, "timeline">, string> = {
    pulls: "community pulls",
    popularity: "contributing doctors",
};

/** Small segmented pill group, shared by the filter and sort controls. */
function Segmented<T extends string>({ label, options, value, onChange }: { label: string; options: ReadonlyArray<{ key: T; label: string }>; value: T; onChange: (key: T) => void }) {
    return (
        // Mobile: label stacked above a full-width, equal-column control. ≥sm:
        // compact inline row, as before.
        <div className="flex flex-col gap-1.5 sm:flex-row sm:items-center sm:gap-2">
            <span className="font-mono text-[9.5px] text-muted-foreground/70 uppercase tracking-[0.14em]">{label}</span>
            <div className="grid auto-cols-fr grid-flow-col gap-1 rounded-lg border border-border bg-muted p-0.75 sm:inline-flex sm:gap-1.5 sm:rounded-[10px]">
                {options.map((o) => {
                    const isActive = value === o.key;
                    return (
                        <button
                            key={o.key}
                            type="button"
                            aria-pressed={isActive}
                            onClick={() => onChange(o.key)}
                            className={`flex h-8 cursor-pointer touch-manipulation items-center justify-center whitespace-nowrap rounded-md px-2 font-medium font-sans text-[11.5px] outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring sm:h-6.5 sm:px-2.5 ${isActive ? "bg-card text-foreground shadow-[0_1px_2px_oklch(0_0_0/0.4)]" : "bg-transparent text-muted-foreground hover:text-foreground"}`}
                        >
                            {o.label}
                        </button>
                    );
                })}
            </div>
        </div>
    );
}

/** Rank badge shown on cards while a metric sort is active. Top 3 get an accent. */
function RankChip({ rank, title }: { rank: number; title: string }) {
    const top = rank <= 3;
    return (
        <span
            className="inline-flex h-4.5 min-w-5.5 items-center justify-center rounded-md px-1 font-mono font-semibold text-[10px] tabular-nums leading-none"
            title={title}
            style={{
                color: top ? "var(--primary)" : "var(--muted-foreground)",
                background: top ? "color-mix(in oklch, var(--primary) 14%, transparent)" : "var(--muted)",
                boxShadow: top ? "inset 0 0 0 1px color-mix(in oklch, var(--primary) 35%, transparent)" : "inset 0 0 0 1px var(--border)",
            }}
        >
            #{rank}
        </span>
    );
}

const PAGE_SIZE = 12;
// Banner runs panel only displays banners from the past N seconds and into the
// future. Older entries are dropped - they crowd the panel and the community
// dataset rarely has pulls that far back anyway.
const PAST_WINDOW_SECS = 180 * 86_400; // 180 days

export function BannerRunsPanel({ banners, operatorsById, statsById, isLoading }: IBannerRunsPanelProps) {
    const [filter, setFilter] = useState<Filter>("active");
    const [sort, setSort] = useState<Sort>("timeline");
    const [showAll, setShowAll] = useState(false);

    const nowSec = useMemo(() => Math.floor(Date.now() / 1000), []);

    const entries = useMemo<IBannerWithStatus[]>(() => {
        // Pre-window: ignore very old retired banners and very speculative
        // far-future entries. Newest-first.
        return banners
            .filter((b) => b.endTime >= nowSec - PAST_WINDOW_SECS)
            .map<IBannerWithStatus>((b) => ({ banner: b, group: classifyBannerGroup(b), status: statusOf(b, nowSec) }))
            .sort((a, b) => {
                // Active first, then upcoming, then most-recently-ended.
                const order: Record<Status, number> = { active: 0, upcoming: 1, ended: 2 };
                if (order[a.status] !== order[b.status]) return order[a.status] - order[b.status];
                if (a.status === "ended") return b.banner.endTime - a.banner.endTime;
                return a.banner.openTime - b.banner.openTime;
            });
    }, [banners, nowSec]);

    const filtered = useMemo(() => {
        if (filter === "all") return entries;
        if (filter === "active") return entries.filter((e) => e.status === "active");
        return entries.filter((e) => e.group === filter);
    }, [entries, filter]);

    // Apply the active sort and attach a 1-based rank for metric sorts. `entries`
    // is already timeline-ordered, so "timeline" needs no re-sort and carries no
    // rank. Metric sorts rank by shared community data, highest first; banners
    // with no community pulls fall to the bottom and stay unranked.
    const ranked = useMemo<Array<{ entry: IBannerWithStatus; rank: number | null }>>(() => {
        if (sort === "timeline") return filtered.map((entry) => ({ entry, rank: null }));

        const metricOf = (e: IBannerWithStatus): number => {
            const s = statsById.get(e.banner.gachaPoolId);
            if (!s) return 0;
            return sort === "popularity" ? s.userCount : s.pullCount;
        };

        const arr = [...filtered].sort((a, b) => {
            const diff = metricOf(b) - metricOf(a);
            if (diff !== 0) return diff;
            // Tie-break by recency so equal-metric banners stay in a stable, sensible order.
            return b.banner.endTime - a.banner.endTime;
        });

        let nextRank = 0;
        return arr.map((entry) => ({ entry, rank: metricOf(entry) > 0 ? ++nextRank : null }));
    }, [filtered, sort, statsById]);

    const visible = showAll ? ranked : ranked.slice(0, PAGE_SIZE);
    const activeCount = entries.filter((e) => e.status === "active").length;

    if (isLoading) {
        return (
            <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-[18px_18px] sm:p-[22px_24px]">
                <div className="h-4 w-32 animate-pulse rounded bg-muted" />
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
                    {Array.from({ length: 6 }).map((_, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: skeleton
                        <div key={i} className="h-32 animate-pulse rounded-xl bg-muted" />
                    ))}
                </div>
            </section>
        );
    }

    if (entries.length === 0) {
        return null;
    }

    return (
        <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-[18px_18px] sm:p-[22px_24px]">
            <header className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between sm:gap-4">
                <div>
                    <Kicker className="mb-1.5">Banner runs</Kicker>
                    <h2 className="m-0 text-balance font-sans font-semibold text-[20px] text-foreground leading-[1.15] tracking-[-0.02em] sm:text-[22px]">
                        {activeCount > 0 ? (
                            <>
                                <em className="text-primary not-italic">{activeCount}</em> active right now.
                            </>
                        ) : (
                            "Recent and upcoming banners."
                        )}
                    </h2>
                </div>

                <div className="flex flex-col gap-3 sm:items-end sm:gap-2">
                    <Segmented
                        label="Sort"
                        options={SORTS}
                        value={sort}
                        onChange={(key) => {
                            setSort(key);
                            setShowAll(false);
                        }}
                    />
                    <Segmented
                        label="Filter"
                        options={FILTERS}
                        value={filter}
                        onChange={(key) => {
                            setFilter(key);
                            setShowAll(false);
                        }}
                    />
                </div>
            </header>

            {filtered.length === 0 ? (
                <div className="py-8 text-center font-sans text-muted-foreground text-sm">No banners match this filter.</div>
            ) : (
                <>
                    <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
                        {visible.map(({ entry, rank }) => (
                            <BannerCard key={entry.banner.gachaPoolId} entry={entry} operatorsById={operatorsById} stat={statsById.get(entry.banner.gachaPoolId)} rank={rank} rankTitle={rank !== null && sort !== "timeline" ? `Ranked #${rank} by ${SORT_METRIC_LABEL[sort]}` : ""} />
                        ))}
                    </div>

                    {filtered.length > PAGE_SIZE ? (
                        <div className="flex justify-center">
                            <button type="button" onClick={() => setShowAll((s) => !s)} className="inline-flex h-8 cursor-pointer items-center rounded-md border border-border bg-card px-4 font-mono text-[11px] text-muted-foreground uppercase tracking-[0.12em] transition-colors hover:text-foreground">
                                {showAll ? "Show fewer" : `Show all ${filtered.length}`}
                            </button>
                        </div>
                    ) : null}
                </>
            )}
        </section>
    );
}
