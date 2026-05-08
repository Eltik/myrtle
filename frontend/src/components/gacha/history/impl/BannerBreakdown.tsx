import { useMemo } from "react";
import { Kicker } from "#/components/ui/kicker";
import type { ClientGachaGroup, IClientGachaRecords, IGachaItem } from "#/lib/api/gacha";
import { formatNumber } from "#/lib/utils";

interface IBannerBreakdownProps {
    records: IClientGachaRecords | null;
    isLoading: boolean;
}

interface IBannerStat {
    poolId: string;
    poolName: string;
    gachaType: ClientGachaGroup;
    typeLabel: string;
    total: number;
    sixStars: number;
    fiveStars: number;
    lastPullAt: number;
}

const TYPE_LABELS: Record<ClientGachaGroup, string> = {
    limited: "Limited",
    linkage: "Collab",
    regular: "Standard",
    special: "Kernel",
};

const TYPE_COLORS: Record<ClientGachaGroup, string> = {
    limited: "oklch(0.85 0.18 80)",
    linkage: "oklch(0.78 0.16 320)",
    regular: "#bcabdb",
    special: "#88c8e3",
};

function groupByPool(items: IGachaItem[], gachaType: ClientGachaGroup): IBannerStat[] {
    const map = new Map<string, IBannerStat>();
    for (const item of items) {
        const key = item.poolId || item.poolName;
        const existing = map.get(key);
        if (existing) {
            existing.total++;
            if (item.star === "6") existing.sixStars++;
            if (item.star === "5") existing.fiveStars++;
            if (item.at > existing.lastPullAt) existing.lastPullAt = item.at;
        } else {
            map.set(key, {
                poolId: item.poolId,
                poolName: item.poolName || item.poolId,
                gachaType,
                typeLabel: TYPE_LABELS[gachaType],
                total: 1,
                sixStars: item.star === "6" ? 1 : 0,
                fiveStars: item.star === "5" ? 1 : 0,
                lastPullAt: item.at,
            });
        }
    }
    return Array.from(map.values()).sort((a, b) => b.lastPullAt - a.lastPullAt);
}

function fmtDate(ts: number): string {
    if (!ts) return "-";
    return new Date(ts).toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
}

export function BannerBreakdown({ records, isLoading }: IBannerBreakdownProps) {
    const banners = useMemo<IBannerStat[]>(() => {
        if (!records) return [];
        return [...groupByPool(records.limited.records, "limited"), ...groupByPool(records.linkage.records, "linkage"), ...groupByPool(records.regular.records, "regular"), ...groupByPool(records.special.records, "special")].sort((a, b) => b.lastPullAt - a.lastPullAt);
    }, [records]);

    if (isLoading) {
        return (
            <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-[22px_24px]">
                <div className="h-4 w-32 rounded bg-muted animate-pulse" />
                <div className="flex flex-col gap-2">
                    {Array.from({ length: 5 }).map((_, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: skeleton
                        <div key={i} className="flex items-center gap-4 py-2 not-last:border-b not-last:border-border/60">
                            <div className="h-3 flex-1 rounded bg-muted animate-pulse" />
                            <div className="h-3 w-14 rounded bg-muted animate-pulse" />
                            <div className="h-3 w-10 rounded bg-muted animate-pulse" />
                        </div>
                    ))}
                </div>
            </section>
        );
    }

    if (!records) return null;

    if (banners.length === 0) {
        return (
            <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-[22px_24px]">
                <Kicker>Banner breakdown</Kicker>
                <div className="py-6 text-center font-sans text-sm text-muted-foreground">No banner data yet.</div>
            </section>
        );
    }

    const maxTotal = Math.max(...banners.map((b) => b.total));

    return (
        <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-[22px_24px]">
            <header>
                <Kicker className="mb-1.5">Banner breakdown</Kicker>
                <h2 className="m-0 font-sans text-[20px] font-semibold leading-[1.15] tracking-[-0.02em] text-foreground sm:text-[22px]">Where your pulls went.</h2>
            </header>

            <div className="-mx-1 max-h-120 overflow-y-auto px-1 [scrollbar-color:var(--border)_transparent] [scrollbar-width:thin]">
                <table className="w-full border-collapse">
                    <thead className="sticky top-0 z-10 bg-card">
                        <tr>
                            <th className="border-b border-border bg-card px-2 py-2 text-left font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground">Banner</th>
                            <th className="hidden border-b border-border bg-card px-2 py-2 text-left font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground sm:table-cell">Last pull</th>
                            <th className="border-b border-border bg-card px-2 py-2 text-right font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground">Pulls</th>
                            <th className="hidden border-b border-border bg-card px-2 py-2 text-right font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground sm:table-cell">6★</th>
                            <th className="hidden border-b border-border bg-card px-2 py-2 text-left font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground lg:table-cell w-36">Distribution</th>
                        </tr>
                    </thead>
                    <tbody>
                        {banners.map((banner) => {
                            const barPct = (banner.total / maxTotal) * 100;
                            const typeColor = TYPE_COLORS[banner.gachaType];
                            return (
                                <tr key={`${banner.gachaType}-${banner.poolId}`} className="not-last:border-b not-last:border-border/50">
                                    <td className="px-2 py-2.5 align-middle">
                                        <div className="flex flex-col gap-0.5">
                                            <span className="font-sans text-[13px] font-medium text-foreground leading-snug">{banner.poolName}</span>
                                            <span className="font-mono text-[9.5px] uppercase tracking-[0.14em]" style={{ color: typeColor }}>
                                                {banner.typeLabel}
                                            </span>
                                        </div>
                                    </td>
                                    <td className="hidden px-2 py-2.5 align-middle font-mono text-[11px] text-muted-foreground tabular-nums sm:table-cell whitespace-nowrap">{fmtDate(banner.lastPullAt)}</td>
                                    <td className="px-2 py-2.5 text-right align-middle font-mono text-[12px] font-semibold tabular-nums text-foreground whitespace-nowrap">{formatNumber(banner.total)}</td>
                                    <td className="hidden px-2 py-2.5 text-right align-middle font-mono text-[11px] tabular-nums sm:table-cell">
                                        <span style={{ color: "#f7a452" }}>{banner.sixStars > 0 ? `${banner.sixStars}×6★` : "-"}</span>
                                    </td>
                                    <td className="hidden px-2 py-2.5 align-middle lg:table-cell w-36">
                                        <div className="h-1.5 w-full overflow-hidden rounded-full bg-muted">
                                            <div className="h-full rounded-full" style={{ width: `${barPct}%`, background: typeColor }} />
                                        </div>
                                    </td>
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
            </div>
        </section>
    );
}
