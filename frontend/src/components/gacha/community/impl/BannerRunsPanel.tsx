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
        <span className="inline-flex items-center gap-1.5 font-mono text-[9.5px] leading-none uppercase tracking-[0.14em]" style={{ color: meta.color }}>
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
                <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-muted font-mono text-[10px] tabular-nums text-muted-foreground" title={`${extra} more`}>
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
        return <div className="font-mono text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground/70">No community pulls yet</div>;
    }
    return (
        <div className="flex items-baseline gap-2 leading-none">
            <span className="font-sans text-[18px] font-semibold tabular-nums tracking-[-0.02em] text-foreground">{formatNumber(stat.pullCount)}</span>
            <span className="font-mono text-[10px] uppercase tracking-[0.14em] text-muted-foreground">community pulls</span>
            <span className="ml-auto font-mono text-[10px] tabular-nums text-muted-foreground" title={`${stat.userCount.toLocaleString()} contributing doctors`}>
                {formatNumberCompact(stat.userCount)}&nbsp;ppl
            </span>
        </div>
    );
}

function BannerCard({ entry, operatorsById, stat }: { entry: IBannerWithStatus; operatorsById: Map<string, IOperatorIndexEntry>; stat: IBannerPullStat | undefined }) {
    const { banner, group, status } = entry;
    const typeColor = TYPE_COLORS[group];

    return (
        <HoverCard>
            <HoverCardTrigger
                render={
                    <div
                        className="flex flex-col gap-2.5 rounded-xl border bg-muted/30 p-3.5 transition-colors hover:bg-muted/60 cursor-default focus-visible:ring-2 focus-visible:ring-primary/40 focus-visible:outline-none"
                        style={{ borderColor: status === "active" ? `color-mix(in oklch, ${typeColor} 50%, var(--border))` : "var(--border)" }}
                    >
                        <div className="flex items-start justify-between gap-2">
                            <span className="font-mono text-[9.5px] uppercase tracking-[0.14em]" style={{ color: typeColor }}>
                                {TYPE_LABELS[group]}
                            </span>
                            <StatusPill status={status} />
                        </div>
                        <div className="min-w-0">
                            <div className="font-sans text-[14px] font-semibold leading-snug text-foreground line-clamp-2">{banner.gachaPoolName}</div>
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
                        <span className="font-sans text-[14px] font-semibold text-foreground leading-snug">{banner.gachaPoolName}</span>
                        {banner.gachaPoolSummary && banner.gachaPoolSummary !== "-" ? <span className="font-sans text-[12px] text-muted-foreground leading-snug">{banner.gachaPoolSummary}</span> : null}
                    </div>

                    <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
                        <span className="font-sans text-[12px] leading-none text-foreground tabular-nums">{fmtDateRange(banner.openTime, banner.endTime)}</span>
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

                    <div className="font-mono text-[10px] text-muted-foreground/70 tabular-nums break-all">{banner.gachaPoolId}</div>
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

const PAGE_SIZE = 12;
// Banner runs panel only displays banners from the past N seconds and into the
// future. Older entries are dropped — they crowd the panel and the community
// dataset rarely has pulls that far back anyway.
const PAST_WINDOW_SECS = 180 * 86_400; // 180 days

export function BannerRunsPanel({ banners, operatorsById, statsById, isLoading }: IBannerRunsPanelProps) {
    const [filter, setFilter] = useState<Filter>("active");
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

    const visible = showAll ? filtered : filtered.slice(0, PAGE_SIZE);
    const activeCount = entries.filter((e) => e.status === "active").length;

    if (isLoading) {
        return (
            <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-[18px_18px] sm:p-[22px_24px]">
                <div className="h-4 w-32 rounded bg-muted animate-pulse" />
                <div className="grid gap-3 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3">
                    {Array.from({ length: 6 }).map((_, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: skeleton
                        <div key={i} className="h-32 rounded-xl bg-muted animate-pulse" />
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
                    <h2 className="m-0 font-sans text-[20px] font-semibold leading-[1.15] tracking-[-0.02em] text-foreground text-balance sm:text-[22px]">
                        {activeCount > 0 ? (
                            <>
                                <em className="not-italic text-primary">{activeCount}</em> active right now.
                            </>
                        ) : (
                            "Recent and upcoming banners."
                        )}
                    </h2>
                </div>

                <div className="inline-flex flex-wrap items-center gap-1.5 rounded-[9px] border border-border bg-muted p-0.75">
                    {FILTERS.map((f) => {
                        const isActive = filter === f.key;
                        return (
                            <button
                                key={f.key}
                                type="button"
                                onClick={() => {
                                    setFilter(f.key);
                                    setShowAll(false);
                                }}
                                className={`h-6.5 cursor-pointer rounded-md px-2.5 font-sans text-[11.5px] font-medium transition-colors ${isActive ? "bg-card text-foreground shadow-[0_1px_2px_oklch(0_0_0/0.4)]" : "bg-transparent text-muted-foreground hover:text-foreground"}`}
                            >
                                {f.label}
                            </button>
                        );
                    })}
                </div>
            </header>

            {filtered.length === 0 ? (
                <div className="py-8 text-center font-sans text-sm text-muted-foreground">No banners match this filter.</div>
            ) : (
                <>
                    <div className="grid gap-3 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3">
                        {visible.map((entry) => (
                            <BannerCard key={entry.banner.gachaPoolId} entry={entry} operatorsById={operatorsById} stat={statsById.get(entry.banner.gachaPoolId)} />
                        ))}
                    </div>

                    {filtered.length > PAGE_SIZE ? (
                        <div className="flex justify-center">
                            <button type="button" onClick={() => setShowAll((s) => !s)} className="inline-flex h-8 cursor-pointer items-center rounded-md border border-border bg-card px-4 font-mono text-[11px] uppercase tracking-[0.12em] text-muted-foreground transition-colors hover:text-foreground">
                                {showAll ? "Show fewer" : `Show all ${filtered.length}`}
                            </button>
                        </div>
                    ) : null}
                </>
            )}
        </section>
    );
}
