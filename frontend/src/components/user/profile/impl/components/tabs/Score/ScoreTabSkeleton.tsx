import { Skeleton } from "#/components/ui/skeleton";
import { cn } from "#/lib/utils";
import { CARD_PADDING, StatCard } from "../Stats/primitives";
import { SUBSCORES } from "./helpers";
import { SCORE_PALETTE } from "./palette";

function KickerSkeleton() {
    return (
        <div className="flex items-center gap-2">
            <Skeleton className="h-3.5 w-3.5 rounded-sm" />
            <Skeleton className="h-2.5 w-24 rounded-sm" />
        </div>
    );
}

function OverallSkeleton() {
    return (
        <StatCard className="sm:col-span-2" color={SCORE_PALETTE.overall}>
            <div className={cn("grid gap-6 sm:grid-cols-[auto_1fr] sm:gap-8", CARD_PADDING)}>
                <div className="flex flex-col items-start gap-4">
                    <KickerSkeleton />
                    <div className="flex items-baseline gap-3">
                        <Skeleton className="h-24 w-28 rounded-xl" />
                        <div className="flex flex-col gap-1.5">
                            <Skeleton className="h-7 w-20 rounded-md" />
                            <Skeleton className="h-2.5 w-16 rounded-sm" />
                        </div>
                    </div>
                    <Skeleton className="h-2.5 w-44 rounded-sm" />
                </div>
                <div className="flex flex-col justify-center gap-3 sm:border-border/50 sm:border-l sm:pl-8">
                    <Skeleton className="h-2.5 w-32 rounded-sm" />
                    {[0, 1, 2].map((i) => (
                        <div className="space-y-1.5" key={i}>
                            <div className="flex items-center justify-between">
                                <Skeleton className="h-3 w-24 rounded-sm" />
                                <Skeleton className="h-2.5 w-10 rounded-sm" />
                            </div>
                            <Skeleton className="h-1 w-full rounded-full" />
                        </div>
                    ))}
                </div>
            </div>
        </StatCard>
    );
}

function SubscoreSkeleton({ color }: { color: string }) {
    return (
        <StatCard color={color}>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <div className="flex items-center justify-between">
                    <KickerSkeleton />
                    <Skeleton className="h-4 w-9 rounded-md" />
                </div>
                <div className="flex flex-1 flex-col justify-between gap-4">
                    <div className="flex flex-col gap-2">
                        <Skeleton className="h-9 w-24 rounded-lg" />
                        <Skeleton className="h-2.5 w-36 rounded-sm" />
                    </div>
                    <div className="space-y-2">
                        <div className="flex items-center justify-between">
                            <Skeleton className="h-2.5 w-16 rounded-sm" />
                            <Skeleton className="h-2.5 w-12 rounded-sm" />
                        </div>
                        <Skeleton className="h-1.5 w-full rounded-full" />
                    </div>
                </div>
            </div>
        </StatCard>
    );
}

export function ScoreTabSkeleton() {
    return (
        <div className="grid gap-3 pb-8 sm:grid-cols-2">
            <OverallSkeleton />
            {SUBSCORES.map((sub) => (
                <SubscoreSkeleton color={sub.color} key={sub.key} />
            ))}
        </div>
    );
}
