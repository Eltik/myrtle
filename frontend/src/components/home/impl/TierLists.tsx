import { useState } from "react";
import { Kicker } from "#/components/ui/kicker";
import { cn } from "#/lib/utils";
import { type Operator, TIER_LISTS } from "./data";
import TierListCard from "./TierListCard";

export default function TierLists({ onSelect }: { onSelect: (op: Operator) => void }) {
    const [filter, setFilter] = useState("All");
    const filters = ["All", "Endgame", "Beginner", "Roguelike", "Niche"];
    const list = TIER_LISTS.filter((t) => filter === "All" || t.tag === filter);

    return (
        <section className="page-wrap my-20">
            <div className="mb-5.5">
                <div className="flex flex-wrap items-end justify-between gap-4">
                    <div>
                        <Kicker>Community</Kicker>
                        <h2 className="m-0 mb-1.5 font-sans text-4xl font-bold leading-tight tracking-tight text-foreground">Tier lists, at a glance.</h2>
                        <p className="mt-1.5 max-w-130 font-sans text-sm leading-normal text-muted-foreground">Previews from the 24 most-watched lists. Click any card to open the full ranking.</p>
                    </div>
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
                </div>
            </div>

            <div className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
                {list.map((tl) => (
                    <TierListCard key={tl.id} tl={tl} onSelect={onSelect} />
                ))}
            </div>

            <a
                href="#"
                className="mt-4.5 inline-flex w-max cursor-pointer items-center gap-2 rounded-lg border border-dashed border-border bg-transparent px-3.5 py-2.5 font-sans text-[12.5px] font-medium leading-none text-muted-foreground no-underline transition-colors hover:border-primary hover:bg-[color-mix(in_srgb,var(--primary)_5%,transparent)] hover:text-foreground"
            >
                <span className="h-1.5 w-1.5 rounded-full bg-primary shadow-[0_0_8px_color-mix(in_srgb,var(--primary)_50%,transparent)]" aria-hidden="true" />
                <span>Browse all 24 tier lists</span>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3">
                    <path d="M5 12h14" />
                    <path d="m12 5 7 7-7 7" />
                </svg>
            </a>
        </section>
    );
}
