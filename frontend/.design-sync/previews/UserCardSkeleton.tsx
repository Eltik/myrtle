import { UserCardSkeleton } from "frontend";

export const Single = () => (
    <div className="w-full max-w-sm">
        <UserCardSkeleton />
    </div>
);

export const LoadingPair = () => (
    <div className="grid w-full grid-cols-1 gap-3 sm:grid-cols-2">
        <UserCardSkeleton />
        <UserCardSkeleton />
    </div>
);

export const UnderResultsHeader = () => (
    <div className="flex w-full flex-col gap-4">
        <div className="flex items-center justify-between gap-3 font-sans text-[12.5px] text-muted-foreground leading-none">
            <span className="inline-flex items-center gap-1.5">Browsing public profiles by total score</span>
            <span className="font-mono text-[11px] uppercase leading-none tracking-[0.08em]">
                <strong className="text-foreground">2,417</strong> doctors
            </span>
        </div>
        <div className="grid w-full grid-cols-1 gap-3 sm:grid-cols-2">
            <UserCardSkeleton />
            <UserCardSkeleton />
            <UserCardSkeleton />
            <UserCardSkeleton />
        </div>
    </div>
);
