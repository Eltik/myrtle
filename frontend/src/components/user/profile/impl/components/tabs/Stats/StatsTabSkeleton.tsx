import { Skeleton } from "#/components/ui/skeleton";
import { cn } from "#/lib/utils";
import { PALETTE } from "./palette";
import { CARD_PADDING, StatCard } from "./primitives";

function KickerSkeleton() {
    return (
        <div className="flex items-center gap-2">
            <Skeleton className="h-3.5 w-3.5 rounded-sm" />
            <Skeleton className="h-2.5 w-28 rounded-sm" />
        </div>
    );
}

function MetricRowSkeleton() {
    return (
        <div className="space-y-2">
            <div className="flex items-center justify-between">
                <Skeleton className="h-2.5 w-20 rounded-sm" />
                <Skeleton className="h-2.5 w-10 rounded-sm" />
            </div>
            <Skeleton className="h-1.5 w-full rounded-full" />
        </div>
    );
}

function TileSkeleton() {
    return (
        <div className="flex flex-col items-center gap-2 rounded-lg border border-border/40 bg-muted/20 p-3">
            <Skeleton className="h-6 w-10 rounded-md" />
            <Skeleton className="h-2.5 w-12 rounded-sm" />
        </div>
    );
}

function CollectionSkeleton() {
    return (
        <StatCard color={PALETTE.collection}>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <KickerSkeleton />
                <div className="flex flex-1 flex-col justify-between gap-5">
                    <div className="flex flex-col items-center gap-2 py-1">
                        <div className="flex items-baseline gap-2">
                            <Skeleton className="h-11 w-20 rounded-lg" />
                            <Skeleton className="h-4 w-12 rounded-sm" />
                        </div>
                        <Skeleton className="h-2.5 w-32 rounded-sm" />
                    </div>
                    <MetricRowSkeleton />
                </div>
            </div>
        </StatCard>
    );
}

function ElitePromotionSkeleton() {
    return (
        <StatCard color={PALETTE.elite}>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <KickerSkeleton />
                <div className="flex flex-1 flex-col justify-center gap-4">
                    {[0, 1, 2].map((i) => (
                        <div className="space-y-1.5" key={i}>
                            <div className="flex items-center justify-between">
                                <Skeleton className="h-2.5 w-14 rounded-sm" />
                                <div className="flex items-baseline gap-1.5">
                                    <Skeleton className="h-3 w-6 rounded-sm" />
                                    <Skeleton className="h-2 w-7 rounded-sm" />
                                </div>
                            </div>
                            <Skeleton className="h-1.5 w-full rounded-full" />
                        </div>
                    ))}
                </div>
            </div>
        </StatCard>
    );
}

function ClassBreakdownSkeleton() {
    return (
        <StatCard className="sm:col-span-2" color={PALETTE.collection}>
            <div className={cn(CARD_PADDING, "pb-2")}>
                <KickerSkeleton />
            </div>
            <div className="flex flex-col gap-1 px-4 pb-4 sm:px-5 sm:pb-5">
                {[0, 1, 2, 3, 4, 5, 6, 7].map((i) => (
                    <div className="flex items-center gap-3 py-2.5" key={i}>
                        <Skeleton className="h-9 w-9 shrink-0 rounded-xl" />
                        <div className="flex w-full min-w-0 flex-col gap-1.5">
                            <div className="flex items-center justify-between">
                                <Skeleton className="h-3 w-24 rounded-sm" />
                                <div className="flex items-baseline gap-1">
                                    <Skeleton className="h-3 w-6 rounded-sm" />
                                    <Skeleton className="h-2 w-7 rounded-sm" />
                                </div>
                            </div>
                            <Skeleton className="h-1.5 w-full rounded-full" />
                        </div>
                    </div>
                ))}
            </div>
        </StatCard>
    );
}

function GapListSkeleton() {
    return (
        <div className="flex flex-wrap items-center gap-x-2.5 gap-y-1">
            <Skeleton className="h-2.5 w-10 rounded-sm" />
            <Skeleton className="h-2.5 w-16 rounded-sm" />
            <Skeleton className="h-2.5 w-20 rounded-sm" />
        </div>
    );
}

function MasterySkeleton() {
    return (
        <StatCard color={PALETTE.mastery.accent}>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <KickerSkeleton />
                <div className="flex flex-1 flex-col justify-center gap-5">
                    <div className="grid grid-cols-3 gap-2">
                        <TileSkeleton />
                        <TileSkeleton />
                        <TileSkeleton />
                    </div>
                    <div className="space-y-2">
                        <div className="flex items-center justify-between">
                            <Skeleton className="h-2.5 w-32 rounded-sm" />
                            <Skeleton className="h-2.5 w-16 rounded-sm" />
                        </div>
                        <Skeleton className="h-1.5 w-full rounded-full" />
                        <div className="flex justify-center">
                            <Skeleton className="h-2 w-40 rounded-sm" />
                        </div>
                    </div>
                    <div className="border-border/60 border-t pt-3">
                        <GapListSkeleton />
                    </div>
                </div>
            </div>
        </StatCard>
    );
}

function ModulesSkinsSkeleton() {
    return (
        <StatCard color={PALETTE.modules.accent}>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <div className="space-y-4">
                    <KickerSkeleton />
                    <div className="grid grid-cols-2 gap-2">
                        <TileSkeleton />
                        <TileSkeleton />
                    </div>
                    <MetricRowSkeleton />
                    <GapListSkeleton />
                </div>

                <div className="border-border/60 border-t" />

                <div className="space-y-4">
                    <KickerSkeleton />
                    <TileSkeleton />
                    <MetricRowSkeleton />
                    <GapListSkeleton />
                </div>
            </div>
        </StatCard>
    );
}

function TopOperatorTileSkeleton({ rank }: { rank: number }) {
    return (
        <div className="relative flex flex-col gap-2.5 bg-card p-3 sm:p-4">
            <div className="absolute top-2.5 right-2.5 font-bold font-mono text-[10px] text-muted-foreground/35 tabular-nums">#{rank}</div>
            <div className="flex items-center gap-2.5 pt-0.5">
                <Skeleton className="h-10 w-10 shrink-0 rounded-lg" />
                <div className="min-w-0 flex-1 space-y-1.5">
                    <Skeleton className="h-3.5 w-24 rounded-sm" />
                    <Skeleton className="h-2.5 w-12 rounded-sm" />
                </div>
            </div>
            <div className="flex gap-1">
                <Skeleton className="h-4 w-4 rounded-sm" />
                <Skeleton className="h-4 w-4 rounded-sm" />
                <Skeleton className="h-4 w-4 rounded-sm" />
            </div>
        </div>
    );
}

function TopOperatorsSkeleton() {
    return (
        <StatCard className="sm:col-span-2" color={PALETTE.top}>
            <div className="p-4 pb-3 sm:p-5 sm:pb-3">
                <KickerSkeleton />
            </div>
            <div className="grid grid-cols-2 gap-px bg-border sm:grid-cols-4">
                {[1, 2, 3, 4, 5, 6, 7, 8].map((rank) => (
                    <TopOperatorTileSkeleton key={rank} rank={rank} />
                ))}
            </div>
        </StatCard>
    );
}

export function StatsTabSkeleton() {
    return (
        <div className="grid gap-3 pb-8 sm:grid-cols-2">
            <CollectionSkeleton />
            <ElitePromotionSkeleton />
            <ClassBreakdownSkeleton />
            <MasterySkeleton />
            <ModulesSkinsSkeleton />
            <TopOperatorsSkeleton />
        </div>
    );
}
