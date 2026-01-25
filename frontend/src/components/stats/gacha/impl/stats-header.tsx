import { InView } from "~/components/ui/motion-primitives/in-view";

interface StatsHeaderProps {
    totalUsers: number;
    computedAt: string;
    cached: boolean;
}

export function StatsHeader({ totalUsers, computedAt, cached }: StatsHeaderProps) {
    return (
        <InView
            once
            transition={{ duration: 0.6, ease: "easeOut" }}
            variants={{
                hidden: { opacity: 0, y: 20 },
                visible: { opacity: 1, y: 0 },
            }}
        >
            <div className="space-y-2 text-center">
                <h1 className="text-balance font-bold text-4xl md:text-5xl">Global Gacha Statistics</h1>
                <p className="mx-auto max-w-2xl text-balance text-lg text-muted-foreground">Community-wide pull data from {totalUsers.toLocaleString()} players sharing their statistics anonymously.</p>
                <p className="text-muted-foreground text-sm">
                    Last updated:{" "}
                    {new Date(computedAt).toLocaleString(undefined, {
                        year: "numeric",
                        month: "short",
                        day: "numeric",
                        hour: "2-digit",
                        minute: "2-digit",
                    })}{" "}
                    {cached && "(Cached)"}
                </p>
            </div>
        </InView>
    );
}
