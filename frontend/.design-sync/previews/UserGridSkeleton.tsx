import { Search, Trophy } from "lucide-react";
import { InputGroup, InputGroupAddon, InputGroupInput, UserGridSkeleton } from "frontend";

/**
 * UserGridSkeleton renders a fixed PAGE_SIZE (24) placeholder cards, so it is
 * taller than one screen at every breakpoint the capture viewport reaches — the
 * stories below frame the top of the grid the way the search route does.
 */
export const SearchResultsLoading = () => <UserGridSkeleton />;

export const UnderSearchChrome = () => (
    <div className="flex flex-col gap-4">
        <h1 className="m-0 font-bold font-sans text-[30px] text-foreground leading-[1.1] tracking-tight">Search Doctors</h1>
        <InputGroup className="max-w-xl">
            <InputGroupAddon>
                <Search aria-hidden="true" />
            </InputGroupAddon>
            <InputGroupInput aria-label="Search doctors" placeholder="Search by nickname…" />
        </InputGroup>
        <div className="flex items-center justify-between gap-3 font-sans text-[12.5px] text-muted-foreground leading-none">
            <span className="inline-flex items-center gap-1.5">
                <Trophy className="h-3 w-3" aria-hidden="true" />
                Browsing public profiles by total score
            </span>
        </div>
        <UserGridSkeleton />
    </div>
);
