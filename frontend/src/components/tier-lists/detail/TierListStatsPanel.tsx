import { Eye, Flame, Heart, Share2, UserRound } from "lucide-react";
import type { ITierListDetail } from "#/lib/api/tier-lists";
import { type IFormatters, useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./TierListStatsPanel.messages";

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

function formatTimeline(iso: string, f: IFormatters): string {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return "-";
    return f.date(d, { year: "numeric", month: "short", day: "numeric" });
}

export function TierListStatsPanel({ detail }: ITierListStatsPanelProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const f = useFormatters();
    const stats = detail.stats;

    const rows: IStatRow[] = stats
        ? [
              { icon: Eye, label: t("detail.stats.views"), value: f.compact(stats.viewCount), title: t("detail.stats.views.title", { count: f.number(stats.viewCount) }) },
              { icon: UserRound, label: t("detail.stats.uniqueViews"), value: f.compact(stats.uniqueViewCount), title: t("detail.stats.uniqueViews.title", { count: f.number(stats.uniqueViewCount) }) },
              { icon: Heart, label: t("detail.stats.favorites"), value: f.compact(stats.favoriteCount), title: t("detail.stats.favorites.title", { count: f.number(stats.favoriteCount) }) },
              { icon: Share2, label: t("detail.stats.shares"), value: f.compact(stats.shareCount), title: t("detail.stats.shares.title", { count: f.number(stats.shareCount) }) },
              { icon: Flame, label: t("detail.stats.views24h"), value: f.compact(stats.viewsLast24h), title: t("detail.stats.views24h.title", { count: f.number(stats.viewsLast24h) }), accent: stats.viewsLast24h > 0 },
          ]
        : [];

    return (
        <aside aria-label={t("detail.stats.panelLabel")} className="lg:sticky lg:top-20">
            <div className="rounded-2xl border border-border bg-card/60 backdrop-blur-sm">
                <header className="flex items-baseline justify-between gap-3 border-border/60 border-b px-4 py-3">
                    <h2 className="m-0 font-sans font-semibold text-foreground text-sm tracking-tight">{t("detail.stats.title")}</h2>
                    {stats?.isTrending && <span className="inline-flex items-center gap-1 font-bold font-mono text-[10.5px] text-primary uppercase tracking-wider">{t("detail.stats.trending")}</span>}
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
                                    <dd className={`m-0 font-mono font-semibold text-[13.5px] tabular-nums ${row.accent ? "text-primary" : "text-foreground"}`}>{row.value}</dd>
                                </div>
                            );
                        })}
                    </dl>
                ) : (
                    <p className="px-4 py-6 text-center font-sans text-[12.5px] text-muted-foreground">{t("detail.stats.empty")}</p>
                )}

                <footer className="space-y-1 border-border/60 border-t px-4 py-3 font-mono text-[10.5px] text-muted-foreground uppercase tracking-wider">
                    <div className="flex items-center justify-between gap-2">
                        <span>{t("detail.stats.created")}</span>
                        <span className="text-foreground/80">{formatTimeline(detail.createdAt, f)}</span>
                    </div>
                    <div className="flex items-center justify-between gap-2">
                        <span>{t("detail.stats.updated")}</span>
                        <span className="text-foreground/80">{formatTimeline(detail.updatedAt, f)}</span>
                    </div>
                </footer>
            </div>
        </aside>
    );
}
