import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";
import { Kicker } from "#/components/ui/kicker";
import { Skeleton } from "#/components/ui/skeleton";
import { homeTierListsQueryOptions, recordTierListViewFn } from "#/lib/api/tier-lists";
import { cn } from "#/lib/utils";
import TierListCard from "./TierListCard";

export default function TierLists() {
    const queryClient = useQueryClient();
    const { data, isLoading, isError } = useQuery(homeTierListsQueryOptions());
    const tierLists = data ?? [];

    const recordView = useMutation({
        mutationFn: (slug: string) => recordTierListViewFn({ data: slug }),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ["tier-lists", "home"] }),
    });

    const handleOpen = useCallback(
        (slug: string) => {
            recordView.mutate(slug);
        },
        [recordView],
    );

    const filters = useMemo(() => {
        const tags = new Set<string>();
        for (const tl of tierLists) tags.add(tl.tag);
        return ["All", ...Array.from(tags)];
    }, [tierLists]);

    const [filter, setFilter] = useState("All");
    const list = tierLists.filter((t) => filter === "All" || t.tag === filter);

    return (
        <section className="mx-auto my-20 w-[min(1080px,calc(100%-2rem))]">
            <div className="mb-5.5">
                <div className="flex flex-wrap items-end justify-between gap-4">
                    <div>
                        <Kicker>Community</Kicker>
                        <h2 className="m-0 mb-1.5 font-sans text-4xl font-bold leading-tight tracking-tight text-foreground">Tier lists, at a glance.</h2>
                        <p className="mt-1.5 max-w-130 font-sans text-sm leading-normal text-muted-foreground">Previews from the most-watched community lists. Click any card to open the full ranking.</p>
                    </div>
                    {filters.length > 1 && (
                        <div role="tablist" className="inline-flex gap-0.5 rounded-[10px] border border-border bg-muted p-0.75">
                            {filters.map((f) => {
                                const active = filter === f;
                                return (
                                    <button
                                        key={f}
                                        role="tab"
                                        type="button"
                                        onClick={() => setFilter(f)}
                                        className={cn(
                                            "h-7 cursor-pointer rounded-[7px] border-0 px-3 font-sans text-xs font-medium leading-none transition-colors",
                                            active ? "bg-primary text-primary-foreground shadow-[0_2px_6px_color-mix(in_srgb,var(--primary)_30%,transparent)]" : "bg-transparent text-muted-foreground hover:bg-muted/60 hover:text-foreground",
                                        )}
                                    >
                                        {f}
                                    </button>
                                );
                            })}
                        </div>
                    )}
                </div>
            </div>

            {isLoading ? (
                <div className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
                    {["s1", "s2", "s3", "s4", "s5", "s6"].map((k) => (
                        <Skeleton key={k} className="h-60 rounded-lg" />
                    ))}
                </div>
            ) : isError ? (
                <div className="rounded-lg border border-dashed border-border bg-muted/20 px-5 py-8 text-center font-sans text-sm text-muted-foreground">Failed to load tier lists. Try refreshing the page.</div>
            ) : list.length === 0 ? (
                <div className="rounded-lg border border-dashed border-border bg-muted/20 px-5 py-8 text-center font-sans text-sm text-muted-foreground">No tier lists yet. Be the first to publish one.</div>
            ) : (
                <div className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
                    {list.map((tl) => (
                        <TierListCard key={tl.id} tl={tl} onOpen={handleOpen} />
                    ))}
                </div>
            )}

            <Link
                to="/tier-lists"
                search={{ type: "all", sort: "trending", q: "", flair: [] }}
                className="mt-4.5 inline-flex w-max cursor-pointer items-center gap-2 rounded-lg border border-dashed border-border bg-transparent px-3.5 py-2.5 font-sans text-[12.5px] font-medium leading-none text-muted-foreground no-underline transition-colors hover:border-primary hover:bg-[color-mix(in_srgb,var(--primary)_5%,transparent)] hover:text-foreground"
            >
                <span className="h-1.5 w-1.5 rounded-full bg-primary shadow-[0_0_8px_color-mix(in_srgb,var(--primary)_50%,transparent)]" aria-hidden="true" />
                <span>Browse all tier lists</span>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3" role="image" aria-label="Right arrow">
                    <path d="M5 12h14" />
                    <path d="m12 5 7 7-7 7" />
                </svg>
            </Link>
        </section>
    );
}
