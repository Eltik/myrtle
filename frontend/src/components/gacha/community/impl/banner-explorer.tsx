"use client";

import { Calendar, Clock, Layers, Search } from "lucide-react";
import { useMemo, useState } from "react";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Badge } from "~/components/ui/shadcn/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Input } from "~/components/ui/shadcn/input";
import { Progress } from "~/components/ui/shadcn/progress";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Tabs, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import type { DatePullData, GachaPoolClient } from "~/types/api";
import { type BannerSortMode, INITIAL_BANNER_DISPLAY, POOL_TYPE_STYLES, POOL_TYPE_TABS, type PoolTypeFilter } from "./banner-constants";
import { buildDatePullMap, filterBanners, type ProcessedBanner, processPoolData, searchBanners, sortBanners } from "./banner-helpers";

interface BannerExplorerProps {
    pools: GachaPoolClient[];
    byDate?: DatePullData[];
}

export function BannerExplorer({ pools, byDate }: BannerExplorerProps) {
    const [searchQuery, setSearchQuery] = useState("");
    const [sortMode, setSortMode] = useState<BannerSortMode>("date");
    const [filterTab, setFilterTab] = useState<PoolTypeFilter>("all");
    const [showAll, setShowAll] = useState(false);

    const byDateMap = useMemo(() => buildDatePullMap(byDate), [byDate]);

    const processedBanners = useMemo(() => processPoolData(pools, byDateMap), [pools, byDateMap]);

    const filteredBanners = useMemo(() => {
        const filtered = filterBanners(processedBanners, filterTab);
        const searched = searchBanners(filtered, searchQuery);
        return sortBanners(searched, sortMode);
    }, [processedBanners, filterTab, searchQuery, sortMode]);

    const maxEstimatedPulls = useMemo(() => {
        return Math.max(1, ...filteredBanners.map((b) => b.estimatedPulls));
    }, [filteredBanners]);

    const activeCount = useMemo(() => processedBanners.filter((b) => b.isActive).length, [processedBanners]);

    const displayedBanners = showAll ? filteredBanners : filteredBanners.slice(0, INITIAL_BANNER_DISPLAY);
    const hasMore = filteredBanners.length > INITIAL_BANNER_DISPLAY;

    if (pools.length === 0) return null;

    return (
        <InView once transition={{ duration: 0.5, ease: "easeOut" }} variants={{ hidden: { opacity: 0, y: 20 }, visible: { opacity: 1, y: 0 } }}>
            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <Layers className="h-5 w-5" />
                        Banner Explorer
                    </CardTitle>
                    <CardDescription>
                        {pools.length} banners{activeCount > 0 ? ` · ${activeCount} currently active` : ""}
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    {/* Controls */}
                    <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
                        <div className="relative flex-1">
                            <Search className="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                            <Input className="pl-9" onChange={(e) => setSearchQuery(e.target.value)} placeholder="Search banners..." value={searchQuery} />
                        </div>
                        <Select onValueChange={(v) => setSortMode(v as BannerSortMode)} value={sortMode}>
                            <SelectTrigger className="w-full sm:w-40" size="sm">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="date">Most Recent</SelectItem>
                                <SelectItem value="activity">Most Activity</SelectItem>
                                <SelectItem value="duration">Longest</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>

                    {/* Filter Tabs */}
                    <Tabs
                        onValueChange={(v) => {
                            setFilterTab(v as PoolTypeFilter);
                            setShowAll(false);
                        }}
                        value={filterTab}
                    >
                        <TabsList>
                            {POOL_TYPE_TABS.map((tab) => (
                                <TabsTrigger key={tab} value={tab}>
                                    {tab === "all" ? "All" : POOL_TYPE_STYLES[tab].label}
                                </TabsTrigger>
                            ))}
                        </TabsList>
                    </Tabs>

                    {/* Result count */}
                    <p className="text-muted-foreground text-xs">
                        {filteredBanners.length} banner{filteredBanners.length !== 1 ? "s" : ""} found
                        {searchQuery && ` matching "${searchQuery}"`}
                    </p>

                    {/* Banner list */}
                    <div className="space-y-2">
                        {displayedBanners.map((banner) => (
                            <BannerCard banner={banner} key={banner.gachaPoolId} maxEstimatedPulls={maxEstimatedPulls} />
                        ))}
                    </div>

                    {/* Empty state */}
                    {filteredBanners.length === 0 && <div className="py-8 text-center text-muted-foreground text-sm">No banners found matching your criteria.</div>}

                    {/* Show more/fewer */}
                    {hasMore && (
                        <button className="w-full rounded-md py-2 text-center text-muted-foreground text-sm transition-colors hover:bg-accent hover:text-accent-foreground" onClick={() => setShowAll((v) => !v)} type="button">
                            {showAll ? "Show fewer" : `Show all ${filteredBanners.length} banners`}
                        </button>
                    )}
                </CardContent>
            </Card>
        </InView>
    );
}

function BannerCard({ banner, maxEstimatedPulls }: { banner: ProcessedBanner; maxEstimatedPulls: number }) {
    const typeStyle = POOL_TYPE_STYLES[banner.poolType];
    const accentColor = banner.cdPrimColor ? `#${banner.cdPrimColor}` : typeStyle.color;
    const activityPct = maxEstimatedPulls > 0 ? (banner.estimatedPulls / maxEstimatedPulls) * 100 : 0;

    return (
        <div className="flex overflow-hidden rounded-lg border border-border/50 transition-colors hover:border-border">
            {/* Color accent strip */}
            <div className="w-1 shrink-0" style={{ backgroundColor: accentColor }} />

            <div className="flex min-w-0 flex-1 flex-col gap-2 p-3">
                {/* Top row: name + badges */}
                <div className="flex flex-wrap items-center gap-2">
                    <span className="min-w-0 truncate font-semibold text-sm">{!banner.gachaPoolName.includes("Rare Operators") ? banner.gachaPoolName : banner.gachaPoolId}</span>
                    <Badge className={typeStyle.className} variant="outline">
                        {typeStyle.label}
                    </Badge>
                    {banner.isActive && (
                        <Badge className="border-green-500/30 bg-green-500/10 text-green-500" variant="outline">
                            Active
                        </Badge>
                    )}
                </div>

                {/* Date range + duration */}
                <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-muted-foreground text-xs">
                    <span className="flex items-center gap-1">
                        <Calendar className="h-3 w-3" />
                        {banner.openDateStr} — {banner.endDateStr}
                    </span>
                    <span className="flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        {banner.durationDays} day{banner.durationDays !== 1 ? "s" : ""}
                    </span>
                </div>

                {/* Estimated activity */}
                {banner.estimatedPulls > 0 && (
                    <div className="space-y-1">
                        <div className="flex items-center justify-between text-xs">
                            <span className="text-muted-foreground">Est. community activity</span>
                            <span className="font-medium">{banner.estimatedPulls.toLocaleString()} pulls</span>
                        </div>
                        <Progress className="h-1.5" value={activityPct} />
                    </div>
                )}
            </div>
        </div>
    );
}
