"use client";

import { Calendar, Hash, Layers, Target } from "lucide-react";
import Image from "next/image";
import { useMemo, useState } from "react";
import { RARITY_COLORS } from "~/components/collection/operators/list/constants";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/shadcn/accordion";
import { Badge } from "~/components/ui/shadcn/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Progress } from "~/components/ui/shadcn/progress";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Separator } from "~/components/ui/shadcn/separator";
import { Skeleton } from "~/components/ui/shadcn/skeleton";
import { type BannerBreakdownEntry, type BannerSortMode, buildBannerBreakdown, formatRate, getAllRecords, HARD_PITY, parseRarity, type SixStarPityEntry, SOFT_PITY_START, sortBannerBreakdown } from "~/lib/gacha-utils";
import type { GachaItem, GachaRecords } from "~/types/api";

const INITIAL_DISPLAY_COUNT = 15;

const GACHA_TYPE_STYLES: Record<string, { label: string; className: string }> = {
    limited: { label: "Limited", className: "border-orange-500/30 bg-orange-500/10 text-orange-500" },
    regular: { label: "Regular", className: "border-blue-500/30 bg-blue-500/10 text-blue-500" },
    special: { label: "Special", className: "border-purple-500/30 bg-purple-500/10 text-purple-500" },
};

const RARITY_TEXT_COLORS: Record<number, string> = {
    6: "text-orange-500",
    5: "text-yellow-500",
    4: "text-purple-500",
    3: "text-blue-500",
};

const RARITY_BG_COLORS: Record<number, string> = {
    6: "bg-orange-500/10",
    5: "bg-yellow-500/10",
    4: "bg-purple-500/10",
    3: "bg-blue-500/10",
};

function formatShortDate(timestamp: number): string {
    return new Date(timestamp).toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
}

function formatDateRange(first: number, last: number): string {
    const firstStr = formatShortDate(first);
    const lastStr = formatShortDate(last);
    if (firstStr === lastStr) return firstStr;
    return `${firstStr} — ${lastStr}`;
}

function formatDateTime(timestamp: number): string {
    return new Date(timestamp).toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
    });
}

interface BannerBreakdownProps {
    records: GachaRecords | null;
    loading?: boolean;
}

export function BannerBreakdown({ records, loading }: BannerBreakdownProps) {
    const [sortMode, setSortMode] = useState<BannerSortMode>("recent");
    const [showAll, setShowAll] = useState(false);

    const allRecords = useMemo(() => {
        if (!records) return [];
        return getAllRecords(records);
    }, [records]);

    const bannerData = useMemo(() => {
        return buildBannerBreakdown(allRecords);
    }, [allRecords]);

    const sortedBanners = useMemo(() => {
        return sortBannerBreakdown(bannerData, sortMode);
    }, [bannerData, sortMode]);

    const totalPulls = useMemo(() => {
        return bannerData.reduce((sum, b) => sum + b.totalPulls, 0);
    }, [bannerData]);

    const displayedBanners = showAll ? sortedBanners : sortedBanners.slice(0, INITIAL_DISPLAY_COUNT);
    const hasMore = sortedBanners.length > INITIAL_DISPLAY_COUNT;

    if (loading) {
        return <Skeleton className="h-48 w-full" />;
    }

    if (!records || bannerData.length === 0) {
        return null;
    }

    return (
        <Card>
            <CardHeader>
                <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                    <div className="flex items-center gap-2">
                        <Layers className="h-5 w-5 text-muted-foreground" />
                        <div>
                            <CardTitle>Banner Breakdown</CardTitle>
                            <CardDescription>
                                {bannerData.length} unique banners · {totalPulls.toLocaleString()} total pulls
                            </CardDescription>
                        </div>
                    </div>
                    <Select onValueChange={(v) => setSortMode(v as BannerSortMode)} value={sortMode}>
                        <SelectTrigger size="sm">
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="recent">Most Recent</SelectItem>
                            <SelectItem value="pullCount">Most Pulls</SelectItem>
                            <SelectItem value="sixStarRate">Best 6★ Rate</SelectItem>
                        </SelectContent>
                    </Select>
                </div>
            </CardHeader>

            <CardContent>
                {/* Summary allocation bar */}
                <AllocationBar banners={sortedBanners} totalPulls={totalPulls} />

                <Separator className="my-4" />

                {/* Banner accordion list */}
                <Accordion className="w-full" collapsible type="single">
                    {displayedBanners.map((banner) => (
                        <BannerAccordionItem banner={banner} key={banner.poolId} totalPulls={totalPulls} />
                    ))}
                </Accordion>

                {/* Show all toggle */}
                {hasMore && !showAll && (
                    <button className="mt-3 w-full rounded-md py-2 text-center text-muted-foreground text-sm transition-colors hover:bg-accent hover:text-accent-foreground" onClick={() => setShowAll(true)} type="button">
                        Show all {sortedBanners.length} banners
                    </button>
                )}
                {hasMore && showAll && (
                    <button className="mt-3 w-full rounded-md py-2 text-center text-muted-foreground text-sm transition-colors hover:bg-accent hover:text-accent-foreground" onClick={() => setShowAll(false)} type="button">
                        Show fewer
                    </button>
                )}
            </CardContent>
        </Card>
    );
}

function AllocationBar({ banners, totalPulls }: { banners: BannerBreakdownEntry[]; totalPulls: number }) {
    if (totalPulls === 0) return null;

    const topBanners = banners.slice(0, 5);
    const otherPulls = banners.slice(5).reduce((sum, b) => sum + b.totalPulls, 0);

    const typeColors: Record<string, string> = {
        limited: "bg-orange-500",
        regular: "bg-blue-500",
        special: "bg-purple-500",
    };

    return (
        <div className="space-y-2">
            <p className="text-muted-foreground text-xs">Pull allocation across top banners</p>
            <div className="flex h-3 w-full overflow-hidden rounded-full bg-muted">
                {topBanners.map((banner) => {
                    const pct = (banner.totalPulls / totalPulls) * 100;
                    if (pct < 0.5) return null;
                    return <div className={`${typeColors[banner.gachaType] ?? "bg-muted-foreground"} opacity-80 transition-all first:rounded-l-full last:rounded-r-full`} key={banner.poolId} style={{ width: `${pct}%` }} title={`${banner.poolName}: ${banner.totalPulls} pulls (${pct.toFixed(1)}%)`} />;
                })}
                {otherPulls > 0 && <div className="bg-muted-foreground/30 last:rounded-r-full" style={{ width: `${(otherPulls / totalPulls) * 100}%` }} title={`Other: ${otherPulls} pulls`} />}
            </div>
            <div className="flex flex-wrap gap-x-3 gap-y-1">
                {topBanners.map((banner) => {
                    const pct = (banner.totalPulls / totalPulls) * 100;
                    if (pct < 0.5) return null;
                    return (
                        <div className="flex items-center gap-1.5 text-xs" key={banner.poolId}>
                            <div className={`h-2 w-2 rounded-full ${typeColors[banner.gachaType] ?? "bg-muted-foreground"} opacity-80`} />
                            <span className="max-w-32 truncate text-muted-foreground">{banner.poolName}</span>
                            <span className="text-foreground">{pct.toFixed(1)}%</span>
                        </div>
                    );
                })}
                {otherPulls > 0 && (
                    <div className="flex items-center gap-1.5 text-xs">
                        <div className="h-2 w-2 rounded-full bg-muted-foreground/30" />
                        <span className="text-muted-foreground">Other</span>
                        <span className="text-foreground">{((otherPulls / totalPulls) * 100).toFixed(1)}%</span>
                    </div>
                )}
            </div>
        </div>
    );
}

function BannerAccordionItem({ banner, totalPulls }: { banner: BannerBreakdownEntry; totalPulls: number }) {
    const typeStyle = GACHA_TYPE_STYLES[banner.gachaType] ?? { label: "Regular", className: "border-blue-500/30 bg-blue-500/10 text-blue-500" };
    const [showAllPulls, setShowAllPulls] = useState(false);

    return (
        <AccordionItem value={banner.poolId}>
            <AccordionTrigger className="gap-3 hover:no-underline">
                <div className="flex min-w-0 flex-1 flex-wrap items-center gap-x-3 gap-y-1">
                    <span className="min-w-0 truncate font-semibold text-sm">{banner.poolName}</span>
                    <Badge className={typeStyle.className} variant="outline">
                        {typeStyle.label}
                    </Badge>
                    <span className="shrink-0 font-medium text-muted-foreground text-xs">{banner.totalPulls} pulls</span>
                    <span className="hidden shrink-0 items-center gap-1 text-muted-foreground text-xs sm:flex">
                        <Calendar className="h-3 w-3" />
                        {formatDateRange(banner.firstPullTimestamp, banner.lastPullTimestamp)}
                    </span>
                </div>
                {/* Rarity indicators */}
                <div className="flex shrink-0 items-center gap-1.5">
                    {banner.sixStarCount > 0 && (
                        <span className="font-bold text-xs" style={{ color: RARITY_COLORS[6] }}>
                            {banner.sixStarCount}★6
                        </span>
                    )}
                    {banner.fiveStarCount > 0 && (
                        <span className="font-medium text-xs" style={{ color: RARITY_COLORS[5] }}>
                            {banner.fiveStarCount}★5
                        </span>
                    )}
                </div>
            </AccordionTrigger>
            <AccordionContent>
                <div className="space-y-5">
                    {/* Banner Info */}
                    <BannerInfoSection banner={banner} totalPulls={totalPulls} typeStyle={typeStyle} />

                    <Separator />

                    {/* 6-Star Pity Timeline */}
                    {(banner.pityHistory.length > 0 || banner.currentPity > 0) && (
                        <>
                            <PityTimeline banner={banner} />
                            <Separator />
                        </>
                    )}

                    {/* Full Pull History */}
                    <PullHistorySection banner={banner} onToggle={() => setShowAllPulls((v) => !v)} showAll={showAllPulls} />
                </div>
            </AccordionContent>
        </AccordionItem>
    );
}

function BannerInfoSection({ banner, totalPulls, typeStyle }: { banner: BannerBreakdownEntry; totalPulls: number; typeStyle: { label: string; className: string } }) {
    const allocationPct = totalPulls > 0 ? (banner.totalPulls / totalPulls) * 100 : 0;

    return (
        <div className="space-y-4">
            {/* Banner details */}
            <div className="grid gap-3 text-xs sm:grid-cols-2 lg:grid-cols-4">
                <div className="space-y-1">
                    <p className="font-medium text-muted-foreground">Banner Type</p>
                    <Badge className={typeStyle.className} variant="outline">
                        {typeStyle.label}
                    </Badge>
                </div>
                <div className="space-y-1">
                    <p className="font-medium text-muted-foreground">Pool ID</p>
                    <p className="font-mono">{banner.poolId}</p>
                </div>
                <div className="space-y-1">
                    <p className="font-medium text-muted-foreground">First Pull</p>
                    <p>{formatDateTime(banner.firstPullTimestamp)}</p>
                </div>
                <div className="space-y-1">
                    <p className="font-medium text-muted-foreground">Last Pull</p>
                    <p>{formatDateTime(banner.lastPullTimestamp)}</p>
                </div>
            </div>

            {/* Rarity breakdown */}
            <div className="grid grid-cols-4 gap-4 text-center">
                <div>
                    <p className="font-bold text-lg" style={{ color: RARITY_COLORS[6] }}>
                        {banner.sixStarCount}
                    </p>
                    <p className="text-muted-foreground text-xs">6★ ({banner.totalPulls > 0 ? formatRate(banner.sixStarCount / banner.totalPulls) : "0%"})</p>
                </div>
                <div>
                    <p className="font-bold text-lg" style={{ color: RARITY_COLORS[5] }}>
                        {banner.fiveStarCount}
                    </p>
                    <p className="text-muted-foreground text-xs">5★ ({banner.totalPulls > 0 ? formatRate(banner.fiveStarCount / banner.totalPulls) : "0%"})</p>
                </div>
                <div>
                    <p className="font-bold text-lg text-purple-500">{banner.fourStarCount}</p>
                    <p className="text-muted-foreground text-xs">4★ ({banner.totalPulls > 0 ? formatRate(banner.fourStarCount / banner.totalPulls) : "0%"})</p>
                </div>
                <div>
                    <p className="font-bold text-blue-500 text-lg">{banner.threeStarCount}</p>
                    <p className="text-muted-foreground text-xs">3★ ({banner.totalPulls > 0 ? formatRate(banner.threeStarCount / banner.totalPulls) : "0%"})</p>
                </div>
            </div>

            {/* Allocation bar */}
            <div className="space-y-1.5">
                <div className="flex items-center justify-between text-xs">
                    <span className="text-muted-foreground">Share of total pulls</span>
                    <span className="font-medium">
                        {banner.totalPulls} / {totalPulls} ({allocationPct.toFixed(1)}%)
                    </span>
                </div>
                <Progress className="h-2" value={allocationPct} />
            </div>
        </div>
    );
}

function PityTimeline({ banner }: { banner: BannerBreakdownEntry }) {
    const avgPity = banner.pityHistory.length > 0 ? banner.pityHistory.reduce((sum, e) => sum + e.pityCount, 0) / banner.pityHistory.length : 0;

    return (
        <div className="space-y-3">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <Target className="h-4 w-4 text-orange-500" />
                    <p className="font-medium text-sm">6★ Pity History</p>
                </div>
                {banner.pityHistory.length > 0 && <span className="text-muted-foreground text-xs">Avg: {avgPity.toFixed(1)} pulls per 6★</span>}
            </div>

            {/* Pity entries */}
            {banner.pityHistory.length > 0 && (
                <div className="space-y-2">
                    {banner.pityHistory.map((entry, index) => (
                        <PityEntry
                            entry={entry}
                            index={index}
                            // biome-ignore lint/suspicious/noArrayIndexKey: entries are chronological and stable
                            key={index}
                        />
                    ))}
                </div>
            )}

            {/* Current pity */}
            <div className="space-y-1.5 rounded-md border border-border/50 bg-muted/30 p-3">
                <div className="flex items-center justify-between text-xs">
                    <span className="font-medium">Current Pity</span>
                    <span className="text-muted-foreground">
                        {banner.currentPity} / {HARD_PITY}
                    </span>
                </div>
                <Progress className="h-1.5" value={(banner.currentPity / HARD_PITY) * 100} />
                <p className="text-muted-foreground text-xs">{banner.currentPity >= SOFT_PITY_START ? "In soft pity range" : `${SOFT_PITY_START - banner.currentPity} pulls until soft pity`}</p>
            </div>
        </div>
    );
}

function PityEntry({ entry, index }: { entry: SixStarPityEntry; index: number }) {
    return (
        <div className="flex items-center gap-3 rounded-md bg-orange-500/5 px-3 py-2">
            <span className="shrink-0 font-mono text-muted-foreground text-xs">#{index + 1}</span>
            <div className="relative h-8 w-8 shrink-0 overflow-hidden rounded border border-orange-500/30">
                <Image alt={entry.charName} className="object-cover" fill sizes="32px" src={`/api/cdn/avatar/${encodeURIComponent(entry.charId)}`} />
            </div>
            <div className="min-w-0 flex-1">
                <p className="truncate font-semibold text-sm" style={{ color: RARITY_COLORS[6] }}>
                    {entry.charName}
                </p>
                <p className="text-muted-foreground text-xs">{formatDateTime(entry.timestamp)}</p>
            </div>
            <div className="shrink-0 text-right">
                <p className={`font-bold text-sm ${entry.inSoftPity ? "text-red-500" : "text-foreground"}`}>{entry.pityCount} pulls</p>
                {entry.inSoftPity && <p className="text-red-500/70 text-xs">soft pity</p>}
            </div>
        </div>
    );
}

const INITIAL_PULL_DISPLAY = 30;

function PullHistorySection({ banner, showAll, onToggle }: { banner: BannerBreakdownEntry; showAll: boolean; onToggle: () => void }) {
    // Pulls are already sorted oldest first from buildBannerBreakdown, reverse for newest first display
    const pullsNewestFirst = useMemo(() => [...banner.pulls].reverse(), [banner.pulls]);
    const displayedPulls = showAll ? pullsNewestFirst : pullsNewestFirst.slice(0, INITIAL_PULL_DISPLAY);
    const hasMore = pullsNewestFirst.length > INITIAL_PULL_DISPLAY;

    // Compute pull number for each pull (1-indexed, oldest = #1)
    const totalCount = banner.pulls.length;

    return (
        <div className="space-y-3">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <Hash className="h-4 w-4 text-muted-foreground" />
                    <p className="font-medium text-sm">Full Pull History</p>
                </div>
                <span className="text-muted-foreground text-xs">{banner.totalPulls} pulls</span>
            </div>

            <div className="space-y-0.5">
                {displayedPulls.map((pull, displayIndex) => {
                    const pullNumber = totalCount - displayIndex;
                    return <PullRow key={`${pull.charId}-${pull.at}`} pull={pull} pullNumber={pullNumber} />;
                })}
            </div>

            {hasMore && (
                <button className="w-full rounded-md py-1.5 text-center text-muted-foreground text-xs transition-colors hover:bg-accent hover:text-accent-foreground" onClick={onToggle} type="button">
                    {showAll ? "Show fewer" : `Show all ${banner.totalPulls} pulls`}
                </button>
            )}
        </div>
    );
}

function PullRow({ pull, pullNumber }: { pull: GachaItem; pullNumber: number }) {
    const rarity = parseRarity(pull.star);
    const rarityColor = RARITY_COLORS[rarity] ?? "#ffffff";
    const textColor = RARITY_TEXT_COLORS[rarity] ?? "text-foreground";
    const bgColor = RARITY_BG_COLORS[rarity] ?? "";

    return (
        <div className={`flex items-center gap-2 rounded px-2 py-1.5 ${rarity >= 5 ? bgColor : ""}`}>
            {/* Pull number */}
            <span className="w-8 shrink-0 text-right font-mono text-muted-foreground text-xs">#{pullNumber}</span>

            {/* Rarity bar */}
            <div className="h-5 w-0.5 shrink-0 rounded-full" style={{ backgroundColor: rarityColor }} />

            {/* Avatar */}
            <div className="relative h-7 w-7 shrink-0 overflow-hidden rounded border border-border/50">
                <Image alt={pull.charName} className="object-cover" fill sizes="28px" src={`/api/cdn/avatar/${encodeURIComponent(pull.charId)}`} />
            </div>

            {/* Operator name + stars */}
            <div className="min-w-0 flex-1">
                <span className={`truncate font-medium text-xs ${rarity >= 5 ? textColor : ""}`}>{pull.charName}</span>
            </div>

            {/* Rarity stars */}
            <div className="flex shrink-0 items-center gap-0.5">
                {Array.from({ length: rarity }).map((_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: Static star display
                    <span className="text-[8px]" key={i} style={{ color: rarityColor }}>
                        ★
                    </span>
                ))}
            </div>

            {/* Timestamp */}
            <span className="hidden shrink-0 text-muted-foreground text-xs sm:inline">{formatShortDate(pull.at)}</span>
        </div>
    );
}
