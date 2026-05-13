import { Eye, Flame, Heart, Share2, UserRound } from "lucide-react";
import type { ITierListDetail } from "#/lib/api/tier-lists";
import { formatNumber, formatNumberCompact } from "#/lib/utils";

interface ITierListStatsPanelProps {
    detail: ITierListDetail;
}

interface IStatRow {
    icon: typeof Eye;
    label: string;
    value: string;
    title: string;
    accent?: boolean;
}

function formatTimeline(iso: string): string {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return "—";
    return d.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
}

export function TierListStatsPanel({ detail }: ITierListStatsPanelProps) {
    const stats = detail.stats;

    const rows: IStatRow[] = stats
        ? [
              { icon: Eye, label: "All-time views", value: formatNumberCompact(stats.viewCount), title: `${formatNumber(stats.viewCount)} total views` },
              { icon: UserRound, label: "Unique viewers", value: formatNumberCompact(stats.uniqueViewCount), title: `${formatNumber(stats.uniqueViewCount)} unique viewers` },
              { icon: Heart, label: "Favorites", value: formatNumberCompact(stats.favoriteCount), title: `${formatNumber(stats.favoriteCount)} favorites` },
              { icon: Share2, label: "Shares", value: formatNumberCompact(stats.shareCount), title: `${formatNumber(stats.shareCount)} shares` },
              { icon: Flame, label: "Views · 24h", value: formatNumberCompact(stats.viewsLast24h), title: `${formatNumber(stats.viewsLast24h)} views in the last 24h`, accent: stats.viewsLast24h > 0 },
          ]
        : [];

    return (
        <aside aria-label="Tier list stats" className="lg:sticky lg:top-20">
            <div className="rounded-2xl border border-border bg-card/60 backdrop-blur-sm">
                <header className="flex items-baseline justify-between gap-3 border-b border-border/60 px-4 py-3">
                    <h2 className="m-0 font-sans text-sm font-semibold tracking-tight text-foreground">Stats</h2>
                    {stats?.isTrending && <span className="inline-flex items-center gap-1 font-mono text-[10.5px] font-bold uppercase tracking-wider text-primary">Trending now</span>}
                </header>

                {stats ? (
                    <dl className="divide-y divide-border/60">
                        {rows.map((row) => {
                            const Icon = row.icon;
                            return (
                                <div key={row.label} className="flex items-center justify-between gap-3 px-4 py-2.5" title={row.title}>
                                    <dt className="inline-flex items-center gap-2 font-sans text-[12.5px] text-muted-foreground">
                                        <Icon className="h-3.5 w-3.5 opacity-70" aria-hidden="true" />
                                        {row.label}
                                    </dt>
                                    <dd className={`m-0 font-mono text-[13.5px] font-semibold tabular-nums ${row.accent ? "text-primary" : "text-foreground"}`}>{row.value}</dd>
                                </div>
                            );
                        })}
                    </dl>
                ) : (
                    <p className="px-4 py-6 text-center font-sans text-[12.5px] text-muted-foreground">No stats yet.</p>
                )}

                <footer className="space-y-1 border-t border-border/60 px-4 py-3 font-mono text-[10.5px] uppercase tracking-wider text-muted-foreground">
                    <div className="flex items-center justify-between gap-2">
                        <span>Created</span>
                        <span className="text-foreground/80">{formatTimeline(detail.createdAt)}</span>
                    </div>
                    <div className="flex items-center justify-between gap-2">
                        <span>Updated</span>
                        <span className="text-foreground/80">{formatTimeline(detail.updatedAt)}</span>
                    </div>
                </footer>
            </div>
        </aside>
    );
}
