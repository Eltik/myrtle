import type { ITierListBrowseItem } from "#/lib/api/tier-lists";
import BrowseCard from "./BrowseCard";

interface Props {
    lists: ITierListBrowseItem[];
    onOpen: (slug: string) => void;
    isTrendingMode: boolean;
}

export function TrendingStrip({ lists, onOpen, isTrendingMode }: Props) {
    if (lists.length === 0) return null;

    return (
        <section className="mx-auto mt-6 w-[min(1080px,calc(100%-2rem))]">
            <header className="mb-3 flex flex-wrap items-end justify-between gap-2">
                <div className="flex items-center gap-2">
                    <h2 className="m-0 font-sans text-base font-semibold tracking-tight text-foreground">{isTrendingMode ? "Trending now" : "Most viewed (24h)"}</h2>
                    <span className="inline-flex items-center gap-1 rounded-full border border-[color-mix(in_srgb,var(--primary)_36%,transparent)] bg-[color-mix(in_srgb,var(--primary)_10%,transparent)] px-2 py-0.5 font-mono text-[10.5px] font-medium leading-none tracking-wider uppercase text-primary">
                        <svg viewBox="0 0 24 24" fill="currentColor" className="h-2.75 w-2.75" aria-hidden="true">
                            <path d="M12 2c.6 4 4.7 5.4 4.7 9.5 0 2.6-2 4.5-4.7 4.5s-4.7-1.9-4.7-4.5C7.3 9.4 9.6 8 9 4c2.4 1.4 3 4 3 4Z" />
                        </svg>
                        Hot
                    </span>
                </div>
                <p className="m-0 font-sans text-[12.5px] text-muted-foreground">{isTrendingMode ? "Weighted by 24-hour views and new favorites." : "Ranked by activity over the past day."}</p>
            </header>

            <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
                {lists.map((tl, i) => (
                    <BrowseCard key={tl.id} tl={tl} size="trending" rank={i + 1} onOpen={onOpen} />
                ))}
            </div>
        </section>
    );
}
