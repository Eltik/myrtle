import { useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import { Link, useParams } from "@tanstack/react-router";
import { useEffect, useRef } from "react";
import { type ITierListDetail, recordTierListViewFn, tierListDetailQueryOptions } from "#/lib/api/tier-lists";
import { TierListBoard } from "./TierListBoard";
import { TierListHero } from "./TierListHero";
import { TierListStatsPanel } from "./TierListStatsPanel";

export function TierListDetail() {
    const { id } = useParams({ from: "/tier-lists_/$id" });
    const { data: detail } = useSuspenseQuery(tierListDetailQueryOptions(id));
    const queryClient = useQueryClient();

    const recordedRef = useRef<string | null>(null);
    const slug = detail?.slug;

    useEffect(() => {
        if (!slug) return;
        if (recordedRef.current === slug) return;
        recordedRef.current = slug;
        recordTierListViewFn({ data: slug })
            .then(({ unique }) => {
                if (!unique) return;
                queryClient.setQueryData<ITierListDetail>(tierListDetailQueryOptions(id).queryKey, (prev) => {
                    if (!prev?.stats) return prev;
                    return {
                        ...prev,
                        stats: {
                            ...prev.stats,
                            viewCount: prev.stats.viewCount + 1,
                            uniqueViewCount: prev.stats.uniqueViewCount + 1,
                        },
                    };
                });
            })
            .catch(() => {});
    }, [slug, id, queryClient]);

    if (!detail) {
        return (
            <main className="mx-auto w-[min(720px,calc(100%-2rem))] py-20 text-center">
                <h1 className="m-0 font-sans text-2xl font-bold tracking-tight text-foreground">Tier list not found</h1>
                <p className="mt-3 font-sans text-sm text-muted-foreground">It may have been removed or the link could be wrong.</p>
                <Link to="/tier-lists" search={{ type: "all", sort: "trending", q: "", flair: [] }} className="mt-6 inline-flex items-center gap-1.5 rounded-lg border border-border bg-popover px-3.5 py-2 font-sans text-sm font-medium leading-none text-foreground no-underline transition-colors hover:bg-accent">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-4 w-4" aria-hidden="true">
                        <path d="m12 19-7-7 7-7" />
                        <path d="M19 12H5" />
                    </svg>
                    Browse all lists
                </Link>
            </main>
        );
    }

    return (
        <main className="min-h-dvh pb-20">
            <TierListHero detail={detail} />

            <div className="mx-auto mt-4 grid w-[min(1200px,calc(100%-1.5rem))] gap-4 sm:mt-6 sm:w-[min(1200px,calc(100%-2rem))] sm:gap-6 lg:grid-cols-[minmax(0,1fr)_280px] lg:items-start lg:gap-8">
                <div className="min-w-0">
                    <TierListBoard detail={detail} />
                </div>
                <TierListStatsPanel detail={detail} />
            </div>
        </main>
    );
}
