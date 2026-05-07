import { Card } from "#/components/ui/card";
import { Skeleton } from "#/components/ui/skeleton";

export function UserCardSkeleton() {
    return (
        <Card>
            <div className="flex items-center gap-3.5 px-4 py-3.5">
                <Skeleton className="h-14 w-14 shrink-0 rounded-xl" />
                <div className="min-w-0 flex-1 flex flex-col gap-2">
                    <div className="flex items-center gap-1.5">
                        <Skeleton className="h-3.5 w-28 rounded-md" />
                        <Skeleton className="h-4 w-8 rounded-full" />
                    </div>
                    <div className="flex items-center gap-1.5">
                        <Skeleton className="h-2.5 w-16 rounded" />
                        <Skeleton className="h-3.5 w-10 rounded-sm" />
                    </div>
                    <div className="flex items-center gap-2">
                        <Skeleton className="h-2.5 w-12 rounded" />
                        <Skeleton className="h-2.5 w-20 rounded" />
                    </div>
                </div>
            </div>
        </Card>
    );
}
