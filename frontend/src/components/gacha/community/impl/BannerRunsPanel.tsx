import { Fragment, useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import { Skeleton } from "#/components/ui/skeleton";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { type ClientGachaGroup, classifyBannerGroup, type IBanner, type IBannerPullStat } from "#/lib/api/gacha";
import { type IFormatters, useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IOperatorIndexEntry } from "#/types/operators";
import { BANNER_GROUP_COLOR, BANNER_GROUP_LABEL_KEYS, BANNER_STATUS_INK, BANNER_STATUS_LABEL_KEYS, type BannerStatus, bannerStatus } from "../../constants";
import type { messages as gachaConstantsMessages } from "../../constants.messages";
import type { messages } from "./BannerRunsPanel.messages";

/** This panel renders its own chrome plus the shared banner bucket and status labels. */
type RunsT = TypedT<typeof messages & typeof gachaConstantsMessages>;
type RunsMessageKey = (keyof typeof messages | keyof typeof gachaConstantsMessages) & string;

interface IBannerRunsPanelProps {
    banners: IBanner[];
    operatorsById: Map<string, IOperatorIndexEntry>;
    statsById: Map<string, IBannerPullStat>;
    isLoading: boolean;
}

type Status = BannerStatus;

interface IBannerWithStatus {
    banner: IBanner;
    group: ClientGachaGroup;
    status: Status;
}

function fmtDateRange(banner: IBanner, f: IFormatters, t: RunsT): string {
    // A returning pool's `endTime` is a placeholder years out, not a date to show.
    if (banner.returning) return t("banner.window.returning");
    const [openSec, endSec] = [banner.openTime, banner.endTime];
    const fmt = (s: number) => f.date(new Date(s * 1000), { month: "short", day: "numeric", year: "numeric" });
    return t("community.runs.dateRange", { open: fmt(openSec), close: fmt(endSec) });
}

function StatusPill({ status }: { status: Status }) {
    const t: RunsT = useT("gacha");
    return (
        <span className="kicker mb-0 inline-flex items-center gap-1.5 leading-none" style={{ color: BANNER_STATUS_INK[status] }}>
            <span className="block h-1.5 w-1.5 rounded-full bg-current" aria-hidden />
            {t(BANNER_STATUS_LABEL_KEYS[status])}
        </span>
    );
}

/** `named` prints each name beside its portrait (the detail popup); without it the name is a hover title plus sr-only text, as the card has no room. */
function FeaturedAvatars({ ids, operatorsById, ringColor, cap, named = false }: { ids: string[]; operatorsById: Map<string, IOperatorIndexEntry>; ringColor: string; cap: number; named?: boolean }) {
    const t: RunsT = useT("gacha");
    if (ids.length === 0) return null;
    const visible = ids.slice(0, cap);
    const extra = ids.length - visible.length;
    return (
        <div className={named ? "flex flex-wrap items-center gap-x-3 gap-y-1.5" : "flex items-center gap-1"}>
            {visible.map((charId) => {
                const op = operatorsById.get(charId);
                const opName = op?.name ?? charId;
                const portrait = (
                    <span className="relative inline-flex h-7 w-7 shrink-0 overflow-hidden rounded-md" title={named ? undefined : opName} style={{ background: "var(--muted)", boxShadow: `inset 0 0 0 1px color-mix(in oklch, ${ringColor} 60%, transparent)` }}>
                        <OperatorAvatar charId={charId} name={opName} className="block h-full w-full object-cover" />
                        {named ? null : <span className="sr-only">{opName}</span>}
                    </span>
                );
                if (!named) return <Fragment key={charId}>{portrait}</Fragment>;
                return (
                    <span key={charId} className="inline-flex min-w-0 items-center gap-1.5 font-sans text-[12px] text-foreground leading-none">
                        {portrait}
                        {opName}
                    </span>
                );
            })}
            {extra > 0 ? (
                <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-muted font-mono text-[10px] text-muted-foreground tabular-nums" title={t("community.runs.moreOperators", { count: extra })}>
                    <span aria-hidden>+{extra}</span>
                    <span className="sr-only">{t("community.runs.moreOperators", { count: extra })}</span>
                </span>
            ) : null}
        </div>
    );
}

function CommunityPullsLine({ stat }: { stat: IBannerPullStat | undefined }) {
    const t: RunsT = useT("gacha");
    const f = useFormatters();
    // No opted-in user has pulled here yet. Said outright so it doesn't read as a failed fetch.
    if (!stat || stat.pullCount === 0) {
        return <div className="font-mono text-[11px] text-subtle-foreground">{t("community.runs.noPulls")}</div>;
    }
    // One decimal is enough to tell banners apart; `f.number` would print up to three.
    const average = stat.userCount > 0 ? Math.round((stat.pullCount / stat.userCount) * 10) / 10 : null;
    return (
        <div className="flex items-baseline gap-2 leading-none">
            <span className="font-sans font-semibold text-[18px] text-foreground tabular-nums tracking-[-0.02em]">{f.number(stat.pullCount)}</span>
            <span className="text-[12px] text-muted-foreground">{t("community.runs.communityPulls")}</span>
            <div className="ml-auto flex flex-col items-end gap-1 font-mono text-[10px] text-muted-foreground tabular-nums">
                <span title={t("community.runs.contributors", { count: f.number(stat.userCount) })}>
                    <span aria-hidden>{t("community.runs.people", { count: f.compact(stat.userCount) })}</span>
                    <span className="sr-only">{t("community.runs.contributors", { count: f.number(stat.userCount) })}</span>
                </span>
                {average !== null ? (
                    <span className="text-subtle-foreground" title={t("community.runs.averagePullsTitle")}>
                        <span className="sr-only">{t("community.runs.averagePullsTitle")} </span>
                        {t("community.runs.averagePulls", { count: f.number(average) })}
                    </span>
                ) : null}
            </div>
        </div>
    );
}

function BannerCard({ entry, operatorsById, stat, rank, rankTitle }: { entry: IBannerWithStatus; operatorsById: Map<string, IOperatorIndexEntry>; stat: IBannerPullStat | undefined; rank: number | null; rankTitle: string }) {
    const t: RunsT = useT("gacha");
    const f = useFormatters();
    const { banner, group, status } = entry;
    const groupColor = BANNER_GROUP_COLOR[group];

    return (
        <HoverCard>
            <HoverCardTrigger
                render={
                    <div
                        className="flex cursor-default flex-col gap-2.5 rounded-2xl border bg-muted/30 p-3.5 transition-colors hover:bg-muted/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        style={{ borderColor: status === "active" ? `color-mix(in oklch, ${groupColor.fill} 50%, var(--border))` : "var(--border)" }}
                    >
                        <div className="flex items-start justify-between gap-2">
                            <div className="flex items-center gap-1.5">
                                {rank !== null ? <RankChip rank={rank} title={rankTitle} /> : null}
                                <span className={`kicker mb-0 ${groupColor.inkClass}`}>{t(BANNER_GROUP_LABEL_KEYS[group])}</span>
                            </div>
                            <StatusPill status={status} />
                        </div>
                        <div className="min-w-0">
                            <div className="line-clamp-2 font-sans font-semibold text-[14px] text-foreground leading-snug">{banner.gachaPoolName}</div>
                            <div className="mt-0.5 font-mono text-[10.5px] text-muted-foreground tabular-nums">{fmtDateRange(banner, f, t)}</div>
                        </div>
                        <CommunityPullsLine stat={stat} />
                        {banner.featured6.length > 0 ? <FeaturedAvatars ids={banner.featured6} operatorsById={operatorsById} ringColor="oklch(0.78 0.18 80)" cap={5} /> : null}
                    </div>
                }
            />
            <HoverCardContent className="w-80 p-4">
                <div className="flex flex-col gap-3">
                    <div className="flex flex-col gap-1">
                        <span className={`kicker mb-0 ${groupColor.inkClass}`}>
                            {t(BANNER_GROUP_LABEL_KEYS[group])} · {banner.gachaRuleType}
                        </span>
                        <span className="font-sans font-semibold text-[14px] text-foreground leading-snug">{banner.gachaPoolName}</span>
                        {/* A returning pool's summary is the runtime template "Ends at {0}", filled in per player by the client. */}
                        {banner.gachaPoolSummary && banner.gachaPoolSummary !== "-" && !banner.returning ? <span className="font-sans text-[12px] text-muted-foreground leading-snug">{banner.gachaPoolSummary}</span> : null}
                    </div>

                    <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
                        <span className="font-sans text-[12px] text-foreground tabular-nums leading-none">{fmtDateRange(banner, f, t)}</span>
                        <StatusPill status={status} />
                    </div>

                    {banner.guarantee5Avail ? <div className="font-mono text-[10.5px] text-muted-foreground">{t("community.runs.guarantee", { name: banner.guaranteeName ?? t("community.runs.guaranteeDefault"), count: banner.guarantee5Count })}</div> : null}

                    {banner.featured6.length > 0 ? (
                        <div className="flex flex-col gap-1.5">
                            <span className="kicker mb-0" style={{ color: "var(--featured-six-ink)" }}>
                                {t("community.runs.featuredSixStar")}
                            </span>
                            <FeaturedAvatars ids={banner.featured6} operatorsById={operatorsById} ringColor="oklch(0.78 0.18 80)" cap={8} named />
                        </div>
                    ) : null}

                    <div className="break-all font-mono text-[10px] text-subtle-foreground tabular-nums">{banner.gachaPoolId}</div>
                </div>
            </HoverCardContent>
        </HoverCard>
    );
}

const FILTERS: ReadonlyArray<{ key: "all" | "active" | "limited" | "linkage" | "special"; labelKey: RunsMessageKey }> = [
    { key: "all", labelKey: "community.runs.filter.all" },
    { key: "active", labelKey: "community.runs.filter.active" },
    { key: "limited", labelKey: "banner.group.limited" },
    { key: "linkage", labelKey: "banner.group.linkage" },
    { key: "special", labelKey: "banner.group.special" },
];
type Filter = (typeof FILTERS)[number]["key"];

const SORTS: ReadonlyArray<{ key: "timeline" | "pulls" | "popularity"; labelKey: RunsMessageKey }> = [
    // Default: status-grouped chronological order (active -> upcoming -> recently ended).
    { key: "timeline", labelKey: "community.runs.sort.timeline" },
    { key: "pulls", labelKey: "community.runs.sort.pulls" },
    { key: "popularity", labelKey: "community.runs.sort.popularity" },
];
type Sort = (typeof SORTS)[number]["key"];

const SORT_METRIC_LABEL_KEYS: Record<Exclude<Sort, "timeline">, RunsMessageKey> = {
    pulls: "community.runs.metric.pulls",
    popularity: "community.runs.metric.popularity",
};

function Segmented<T extends string>({ label, options, value, onChange }: { label: string; options: ReadonlyArray<{ key: T; label: string }>; value: T; onChange: (key: T) => void }) {
    // Kit single-select group: pressed state with its inset ring and the coarse 44px hit area.
    return (
        <div className="flex flex-col gap-1.5 sm:flex-row sm:items-center sm:gap-2">
            <span className="kicker mb-0" aria-hidden>
                {label}
            </span>
            <ToggleGroup
                aria-label={label}
                onValueChange={(next) => {
                    const picked = options.find((o) => o.key === next[0]);
                    if (picked) onChange(picked.key);
                }}
                size="sm"
                value={[value]}
                variant="outline"
                className="grid w-full auto-cols-fr grid-flow-col sm:flex sm:w-fit"
            >
                {options.map((o) => (
                    <ToggleGroupItem key={o.key} className="min-w-0 px-2.5 text-[12px]" value={o.key}>
                        {o.label}
                    </ToggleGroupItem>
                ))}
            </ToggleGroup>
        </div>
    );
}

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
// Banners that ended longer ago than this are dropped: they crowd the panel and
// the community data rarely has pulls that far back.
const PAST_WINDOW_SECS = 180 * 86_400;

export function BannerRunsPanel({ banners, operatorsById, statsById, isLoading }: IBannerRunsPanelProps) {
    const t: RunsT = useT("gacha");
    const f = useFormatters();
    const [filter, setFilter] = useState<Filter>("active");
    const [sort, setSort] = useState<Sort>("timeline");
    const [showAll, setShowAll] = useState(false);

    const nowSec = useMemo(() => Math.floor(Date.now() / 1000), []);

    const entries = useMemo<IBannerWithStatus[]>(() => {
        return banners
            .filter((b) => b.endTime >= nowSec - PAST_WINDOW_SECS)
            .map<IBannerWithStatus>((b) => ({ banner: b, group: classifyBannerGroup(b), status: bannerStatus(b, nowSec) }))
            .sort((a, b) => {
                // Active first, then upcoming, then returning-only, then most-recently-ended.
                const order: Record<Status, number> = { active: 0, upcoming: 1, returning: 2, ended: 3 };
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
            // Ties: most recently ended first.
            return b.banner.endTime - a.banner.endTime;
        });

        let nextRank = 0;
        return arr.map((entry) => ({ entry, rank: metricOf(entry) > 0 ? ++nextRank : null }));
    }, [filtered, sort, statsById]);

    const visible = showAll ? ranked : ranked.slice(0, PAGE_SIZE);
    const activeCount = entries.filter((e) => e.status === "active").length;

    // The segmented control takes resolved labels, so the option tables above
    // stay plain data and the keys are resolved in one place.
    const filterOptions = useMemo(() => FILTERS.map((o) => ({ key: o.key, label: t(o.labelKey) })), [t]);
    const sortOptions = useMemo(() => SORTS.map((o) => ({ key: o.key, label: t(o.labelKey) })), [t]);

    if (isLoading) {
        return (
            <section id="banner-runs" className="flex scroll-mt-24 flex-col gap-4 rounded-2xl border border-border bg-card p-[18px_18px] sm:p-[22px_24px]">
                <Skeleton className="h-4 w-32 rounded" />
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
                    {Array.from({ length: 6 }).map((_, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: skeleton
                        <Skeleton key={i} className="h-32 rounded-2xl" />
                    ))}
                </div>
            </section>
        );
    }

    if (entries.length === 0) {
        return null;
    }

    return (
        <section id="banner-runs" className="flex scroll-mt-24 flex-col gap-4 rounded-2xl border border-border bg-card p-[18px_18px] sm:p-[22px_24px]">
            <header className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between sm:gap-4">
                <div className="flex items-baseline gap-2">
                    <h2 className="m-0 font-sans font-semibold text-[15px] text-foreground leading-tight">{t("community.runs.kicker")}</h2>
                    {activeCount > 0 ? <span className="font-sans text-[12.5px] text-muted-foreground tabular-nums">{t("community.runs.active", { count: f.number(activeCount) })}</span> : null}
                </div>

                <div className="flex flex-col gap-3 sm:items-end sm:gap-2">
                    <Segmented
                        label={t("community.runs.sort")}
                        options={sortOptions}
                        value={sort}
                        onChange={(key) => {
                            setSort(key);
                            setShowAll(false);
                        }}
                    />
                    <Segmented
                        label={t("community.runs.filter")}
                        options={filterOptions}
                        value={filter}
                        onChange={(key) => {
                            setFilter(key);
                            setShowAll(false);
                        }}
                    />
                </div>
            </header>

            {filtered.length === 0 ? (
                <div className="py-8 text-center font-sans text-muted-foreground text-sm">{t("community.runs.empty")}</div>
            ) : (
                <>
                    <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
                        {visible.map(({ entry, rank }) => (
                            <BannerCard key={entry.banner.gachaPoolId} entry={entry} operatorsById={operatorsById} stat={statsById.get(entry.banner.gachaPoolId)} rank={rank} rankTitle={rank !== null && sort !== "timeline" ? t("community.runs.rankTitle", { rank, metric: t(SORT_METRIC_LABEL_KEYS[sort]) }) : ""} />
                        ))}
                    </div>

                    {filtered.length > PAGE_SIZE ? (
                        <div className="flex justify-center">
                            <Button onClick={() => setShowAll((s) => !s)} size="sm" variant="outline">
                                {showAll ? t("community.runs.showFewer") : t("community.runs.showAll", { count: filtered.length })}
                            </Button>
                        </div>
                    ) : null}
                </>
            )}
        </section>
    );
}
