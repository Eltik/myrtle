import { Card, CardHeader, CardPanel, CardTitle, Skeleton } from "frontend";

/** The tier-list grid's loading state, exactly as home/TierLists renders it. */
export const TierListGrid = () => (
    <div className="grid w-full grid-cols-1 gap-3 sm:grid-cols-3">
        {["s1", "s2", "s3"].map((k) => (
            <Skeleton className="h-40 rounded-lg" key={k} />
        ))}
    </div>
);

/** Roster rows: a round avatar placeholder plus two text lines of unequal width. */
export const OperatorRows = () => (
    <div className="flex w-full max-w-md flex-col gap-3">
        {["r1", "r2", "r3", "r4"].map((k) => (
            <div className="flex items-center gap-3" key={k}>
                <Skeleton className="size-10 rounded-full" />
                <div className="flex min-w-0 flex-1 flex-col gap-1.5">
                    <Skeleton className="h-3.5 w-40" />
                    <Skeleton className="h-3 w-24" />
                </div>
                <Skeleton className="h-6 w-16 rounded-md" />
            </div>
        ))}
    </div>
);

/** Inside a real card: the sanity-spent stat before the query resolves. */
export const StatCard = () => (
    <div className="grid w-full max-w-2xl gap-4 sm:grid-cols-2">
        <Card>
            <CardHeader>
                <CardTitle>Sanity spent</CardTitle>
            </CardHeader>
            <CardPanel className="flex flex-col gap-2">
                <Skeleton className="h-8 w-28" />
                <Skeleton className="h-3 w-40" />
            </CardPanel>
        </Card>
        <Card>
            <CardHeader>
                <CardTitle>Operators owned</CardTitle>
            </CardHeader>
            <CardPanel className="flex flex-col gap-2">
                <Skeleton className="h-8 w-20" />
                <Skeleton className="h-3 w-36" />
            </CardPanel>
        </Card>
    </div>
);

/** Prose placeholder — an operator's lore blurb while the gamedata request is in flight. */
export const TextBlock = () => (
    <div className="flex w-full max-w-lg flex-col gap-2.5">
        <Skeleton className="h-5 w-56 rounded-md" />
        <Skeleton className="h-3 w-full" />
        <Skeleton className="h-3 w-full" />
        <Skeleton className="h-3 w-64" />
    </div>
);
