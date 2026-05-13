import type { ITierListBrowseItem } from "#/lib/api/tier-lists";
import BrowseCard from "./BrowseCard";

interface IOfficialRailProps {
    lists: ITierListBrowseItem[];
    onOpen: (slug: string) => void;
    onViewAll: () => void;
}

export function OfficialRail({ lists, onOpen, onViewAll }: IOfficialRailProps) {
    if (lists.length === 0) return null;

    return (
        <section className="mx-auto mt-4 w-[min(1080px,calc(100%-2rem))]">
            <header className="mb-3 flex items-end justify-between gap-3">
                <div className="flex items-center gap-2">
                    <h2 className="m-0 font-sans text-base font-semibold tracking-tight text-foreground">Official</h2>
                    <span className="rounded-full border border-border bg-muted px-2 py-0.5 font-mono text-[10.5px] font-medium leading-none tracking-wider text-muted-foreground uppercase">From the team</span>
                </div>
                <button type="button" onClick={onViewAll} className="inline-flex items-center gap-1 font-sans text-[12.5px] font-medium leading-none text-muted-foreground transition-colors hover:text-foreground [&>svg]:h-3 [&>svg]:w-3 [&>svg]:transition-transform hover:[&>svg]:translate-x-0.5">
                    View all
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                        <path d="M5 12h14" />
                        <path d="m12 5 7 7-7 7" />
                    </svg>
                </button>
            </header>

            <div className="-mx-4 px-4 sm:mx-0 sm:px-0">
                <div className="flex snap-x snap-mandatory gap-3 overflow-x-auto pb-3 [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden">
                    {lists.map((tl) => (
                        <div key={tl.id} className="w-70 shrink-0 snap-start">
                            <BrowseCard tl={tl} onOpen={onOpen} />
                        </div>
                    ))}
                </div>
            </div>
        </section>
    );
}
