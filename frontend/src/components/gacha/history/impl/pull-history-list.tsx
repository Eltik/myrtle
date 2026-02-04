"use client";

import Image from "next/image";
import { useState } from "react";
import { RARITY_COLORS } from "~/components/collection/operators/list/constants";
import { Pagination } from "~/components/collection/operators/list/ui/impl/pagination";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/shadcn/collapsible";
import { Skeleton } from "~/components/ui/shadcn/skeleton";
import { formatPullDate } from "~/lib/gacha-utils";
import { cn } from "~/lib/utils";
import type { GachaRecordEntry } from "~/types/api";

interface PullHistoryListProps {
    records: GachaRecordEntry[];
    loading?: boolean;
    currentPage: number;
    totalPages: number;
    onPageChange: (page: number) => void;
    isPageLoading?: boolean;
}

export function PullHistoryList({ records, loading, currentPage, totalPages, onPageChange, isPageLoading }: PullHistoryListProps) {
    const [expandedPull, setExpandedPull] = useState<string | null>(null);

    if (loading) {
        return (
            <div className="space-y-1">
                {Array.from({ length: 10 }).map((_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: Static skeleton placeholders
                    <Skeleton className="h-16 w-full" key={i} />
                ))}
            </div>
        );
    }

    if (records.length === 0) {
        return (
            <Card>
                <CardContent className="flex min-h-50 items-center justify-center py-8">
                    <div className="text-center text-muted-foreground">
                        <p className="font-medium">No pull history found</p>
                        <p className="text-sm">Your pull history will appear here once you sync your gacha data</p>
                    </div>
                </CardContent>
            </Card>
        );
    }

    return (
        <div className="min-w-0 space-y-1">
            {records.map((record) => {
                const rarity = record.rarity;
                const isExpanded = expandedPull === record.id;
                const rarityColor = RARITY_COLORS[rarity] ?? "#ffffff";

                return (
                    <Collapsible key={record.id} onOpenChange={(open) => setExpandedPull(open ? record.id : null)} open={isExpanded}>
                        <div className={cn("card-hover-transition group relative rounded-lg border border-transparent bg-card/50 contain-layout hover:border-border hover:bg-card", isExpanded && "border-border bg-card")}>
                            <CollapsibleTrigger asChild>
                                <div className="flex cursor-pointer items-center gap-3 px-3 py-2.5">
                                    {/* Rarity indicator line on left */}
                                    <div className="absolute top-1/2 left-0 h-10 w-0.5 -translate-y-1/2 rounded-full opacity-60 opacity-transition group-hover:opacity-100" style={{ backgroundColor: rarityColor }} />

                                    {/* Operator Portrait */}
                                    <div className="relative h-12 w-12 shrink-0 overflow-hidden rounded-md border border-border/50 bg-background">
                                        <div className="absolute inset-0 transform-gpu transition-transform duration-200 ease-out group-hover:scale-110">
                                            <Image alt={record.charName} className="object-cover" fill sizes="48px" src={`/api/cdn/avatar/${encodeURIComponent(record.charId)}`} />
                                        </div>
                                    </div>

                                    {/* Pull Info */}
                                    <div className="flex min-w-0 flex-1 flex-col gap-1">
                                        {/* Name row with stars */}
                                        <div className="flex items-center gap-2">
                                            <span className="truncate font-semibold text-foreground text-sm uppercase tracking-wide">{record.charName}</span>
                                            {/* Rarity Stars */}
                                            <div className="flex shrink-0 items-center gap-0.5">
                                                {Array.from({ length: rarity }).map((_, i) => (
                                                    // biome-ignore lint/suspicious/noArrayIndexKey: Static star icons
                                                    <span className="text-xs" key={i} style={{ color: rarityColor }}>
                                                        ★
                                                    </span>
                                                ))}
                                            </div>
                                        </div>

                                        {/* Info row: Banner, Pool, Date */}
                                        <div className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-0.5 text-xs">
                                            <span className="shrink-0 text-muted-foreground">{record.gachaType}</span>
                                            <span className="hidden text-muted-foreground/50 sm:inline">·</span>
                                            <span className="hidden truncate text-muted-foreground sm:inline">{record.poolName}</span>
                                            <span className="text-muted-foreground/50">·</span>
                                            <span className="text-muted-foreground">{formatPullDate(record.pullTimestamp)}</span>
                                        </div>
                                    </div>

                                    {/* Expand indicator */}
                                    <div className="flex shrink-0 items-center justify-center opacity-40 transition-opacity group-hover:opacity-70">
                                        <svg aria-label="Expand details" className={cn("h-5 w-5 transition-transform", isExpanded && "rotate-180")} fill="none" role="img" stroke="currentColor" strokeWidth={2} viewBox="0 0 24 24">
                                            <path d="M19 9l-7 7-7-7" strokeLinecap="round" strokeLinejoin="round" />
                                        </svg>
                                    </div>
                                </div>
                            </CollapsibleTrigger>

                            {/* Expanded Details */}
                            <CollapsibleContent>
                                <div className="border-border border-t px-3 pt-2 pb-3">
                                    <div className="grid grid-cols-2 gap-3 text-xs md:grid-cols-4">
                                        <div>
                                            <p className="font-medium text-muted-foreground">Pool ID</p>
                                            <p className="mt-0.5 font-mono">{record.poolId}</p>
                                        </div>
                                        <div>
                                            <p className="font-medium text-muted-foreground">Character ID</p>
                                            <p className="mt-0.5 font-mono">{record.charId}</p>
                                        </div>
                                        <div>
                                            <p className="font-medium text-muted-foreground">Timestamp</p>
                                            <p className="mt-0.5 font-mono">{record.pullTimestamp}</p>
                                        </div>
                                        <div>
                                            <p className="font-medium text-muted-foreground">Rarity</p>
                                            <p className="mt-0.5">{rarity}-Star</p>
                                        </div>
                                    </div>
                                </div>
                            </CollapsibleContent>
                        </div>
                    </Collapsible>
                );
            })}

            {/* Pagination */}
            {totalPages > 1 && (
                <div className="relative">
                    {isPageLoading && (
                        <div className="absolute inset-0 z-10 flex items-center justify-center bg-background/50">
                            <div className="h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                        </div>
                    )}
                    <Pagination currentPage={currentPage} onPageChange={onPageChange} totalPages={totalPages} />
                </div>
            )}
        </div>
    );
}
