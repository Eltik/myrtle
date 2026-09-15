import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";
import { Kicker } from "#/components/ui/kicker";
import { Skeleton } from "#/components/ui/skeleton";
import { homeTierListsQueryOptions, recordTierListViewFn } from "#/lib/api/tier-lists";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import TierListCard from "./TierListCard";
import type { messages } from "./TierLists.messages";

/** Sentinel tag value for "no filter". Compared against a list's own tag, so it is not a message. */
const ALL_TAGS = "All";

export default function TierLists() {
    const t: TypedT<typeof messages> = useT("home");
    const queryClient = useQueryClient();
    const { data, isLoading, isError } = useQuery(homeTierListsQueryOptions(useGamedataServer()));
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
        return [ALL_TAGS, ...Array.from(tags)];
    }, [tierLists]);

    const [filter, setFilter] = useState(ALL_TAGS);
    const list = tierLists.filter((tl) => filter === ALL_TAGS || tl.tag === filter);

    return (
        <section className="mx-auto my-20 w-[min(1080px,calc(100%-2rem))]">
            <div className="mb-5.5">
                <div className="flex flex-wrap items-end justify-between gap-4">
                    <div>
                        <Kicker>{t("tierLists.kicker")}</Kicker>
                        <h2 className="m-0 mb-1.5 font-bold font-sans text-4xl text-foreground leading-tight tracking-tight">{t("tierLists.title")}</h2>
                        <p className="mt-1.5 max-w-130 font-sans text-muted-foreground text-sm leading-normal">{t("tierLists.blurb")}</p>
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
                                            "h-7 cursor-pointer rounded-[7px] border-0 px-3 font-medium font-sans text-xs leading-none transition-colors",
                                            active ? "bg-primary text-primary-foreground shadow-[0_2px_6px_color-mix(in_srgb,var(--primary)_30%,transparent)]" : "bg-transparent text-muted-foreground hover:bg-muted/60 hover:text-foreground",
                                        )}
                                    >
                                        {f === ALL_TAGS ? t("tierLists.filter.all") : f}
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
                <div className="rounded-lg border border-border border-dashed bg-muted/20 px-5 py-8 text-center font-sans text-muted-foreground text-sm">{t("tierLists.error")}</div>
            ) : list.length === 0 ? (
                <div className="rounded-lg border border-border border-dashed bg-muted/20 px-5 py-8 text-center font-sans text-muted-foreground text-sm">{t("tierLists.empty")}</div>
            ) : (
                <div className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
                    {list.map((tl) => (
                        <TierListCard key={tl.id} tl={tl} onOpen={handleOpen} />
                    ))}
                </div>
            )}

            <Link
                to="/tier-lists"
                search={{ type: "all", sort: "recent", q: "", flair: [] }}
                className="mt-4.5 inline-flex w-max cursor-pointer items-center gap-2 rounded-lg border border-border border-dashed bg-transparent px-3.5 py-2.5 font-medium font-sans text-[12.5px] text-muted-foreground leading-none no-underline transition-colors hover:border-primary hover:bg-[color-mix(in_srgb,var(--primary)_5%,transparent)] hover:text-foreground"
            >
                <span className="h-1.5 w-1.5 rounded-full bg-primary shadow-[0_0_8px_color-mix(in_srgb,var(--primary)_50%,transparent)]" aria-hidden="true" />
                <span>{t("tierLists.browseAll")}</span>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3" role="image" aria-label={t("tierLists.arrowIcon")}>
                    <path d="M5 12h14" />
                    <path d="m12 5 7 7-7 7" />
                </svg>
            </Link>
        </section>
    );
}
