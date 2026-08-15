import { LeaderboardTableSkeleton } from "frontend";
import type { ReactNode } from "react";

const Frame = ({ children }: { children: ReactNode }) => <div className="overflow-hidden rounded-2xl border border-border bg-card shadow-[0_1px_2px_rgb(0_0_0/0.04)]">{children}</div>;

export const LoadingPage = () => (
    <Frame>
        <LeaderboardTableSkeleton rows={6} />
    </Frame>
);

export const CompactLoading = () => (
    <Frame>
        <LeaderboardTableSkeleton rows={3} />
    </Frame>
);

export const WithPaginationFooter = () => (
    <Frame>
        <LeaderboardTableSkeleton rows={4} />
        <div className="flex flex-wrap items-center justify-between gap-3 border-border border-t bg-[color-mix(in_srgb,var(--muted)_30%,transparent)] px-4 py-3.5">
            <span className="font-mono text-muted-foreground text-xs tabular-nums leading-none">Loading Doctors…</span>
            <span className="h-8 w-40 rounded-lg bg-muted" />
        </div>
    </Frame>
);
