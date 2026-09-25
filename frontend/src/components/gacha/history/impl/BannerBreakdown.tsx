import { useMemo } from "react";
import { Kicker } from "#/components/ui/kicker";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import type { ClientGachaGroup, IBanner, IClientGachaRecords, IGachaItem } from "#/lib/api/gacha";
import { type IFormatters, useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { rarityStarColor } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import { BANNER_GROUP_LABEL_KEYS, BANNER_RULE_TYPE_LABEL_KEYS, BANNER_STATUS_LABEL_KEYS, type GachaMessageKey } from "../../constants";
import type { messages as gachaConstantsMessages } from "../../constants.messages";
import type { messages } from "./BannerBreakdown.messages";

/** This panel renders its own chrome plus the shared banner bucket, status and rule-type labels. */
type BreakdownT = TypedT<typeof messages & typeof gachaConstantsMessages>;

interface IBannerBreakdownProps {
    records: IClientGachaRecords | null;
    bannersById: Map<string, IBanner>;
    operatorsById: Map<string, IOperatorIndexEntry>;
    isLoading: boolean;
}

const FEATURED_AVATAR_CAP = 6;

interface IBannerStat {
    poolId: string;
    poolName: string;
    gachaType: ClientGachaGroup;
    typeLabelKey: GachaMessageKey;
    total: number;
    sixStars: number;
    fiveStars: number;
    lastPullAt: number;
}

const TYPE_COLORS: Record<ClientGachaGroup, string> = {
    limited: "oklch(0.85 0.18 80)",
    linkage: "oklch(0.78 0.16 320)",
    regular: "#bcabdb",
    special: "#88c8e3",
};

function groupByPool(items: IGachaItem[], gachaType: ClientGachaGroup, bannersById: Map<string, IBanner>): IBannerStat[] {
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
            // Prefer static-data banner name - the Yostar API often echoes a
            // blank `pool_name`, and old rows persisted before the lookup
            // existed had to fall back to the raw pool_id (the IDs aren't
            // user-friendly).
            const banner = bannersById.get(item.poolId);
            const resolvedName = banner?.gachaPoolName || item.poolName || item.poolId;
            map.set(key, {
                poolId: item.poolId,
                poolName: resolvedName,
                gachaType,
                typeLabelKey: BANNER_GROUP_LABEL_KEYS[gachaType],
                total: 1,
                sixStars: item.star === "6" ? 1 : 0,
                fiveStars: item.star === "5" ? 1 : 0,
                lastPullAt: item.at,
            });
        }
    }
    return Array.from(map.values()).sort((a, b) => b.lastPullAt - a.lastPullAt);
}

function fmtDate(ts: number, f: IFormatters): string {
    if (!ts) return "-";
    return f.date(new Date(ts), { month: "short", day: "numeric", year: "numeric" });
}

/** Banner table uses unix-seconds (static data); pull records use ms. Convert. */
function fmtDateFromSeconds(secs: number, f: IFormatters): string {
    if (!secs) return "-";
    return f.date(new Date(secs * 1000), { month: "short", day: "numeric", year: "numeric" });
}

function FeaturedRow({ ids, star, operatorsById }: { ids: string[]; star: number; operatorsById: Map<string, IOperatorIndexEntry> }) {
    const t: BreakdownT = useT("gacha");
    if (ids.length === 0) return null;
    const visible = ids.slice(0, FEATURED_AVATAR_CAP);
    const extra = ids.length - visible.length;
    const ringColor = rarityStarColor(star);

    return (
        <div className="flex flex-col gap-1.5">
            <span className="font-mono text-[9.5px] uppercase tracking-[0.14em]" style={{ color: ringColor }}>
                {t("history.breakdown.featured", { rarity: star })}
            </span>
            <div className="flex flex-wrap gap-1.5">
                {visible.map((charId) => {
                    const op = operatorsById.get(charId);
                    const opName = op?.name ?? charId;
                    return (
                        <span key={charId} className="flex flex-col items-center gap-0.5" title={opName}>
                            <span className="relative inline-flex h-8 w-8 shrink-0 overflow-hidden rounded-md ring-1" style={{ background: "var(--muted)", boxShadow: `inset 0 0 0 1px color-mix(in oklch, ${ringColor} 60%, transparent)` }}>
                                <OperatorAvatar charId={charId} name={opName} className="block h-full w-full object-cover" />
                            </span>
                            <span className="max-w-12 truncate text-center font-mono text-[9px] text-muted-foreground leading-none">{opName.split(" ")[0]}</span>
                        </span>
                    );
                })}
                {extra > 0 ? (
                    <span className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-muted font-mono text-[10px] text-muted-foreground tabular-nums" title={t("history.breakdown.moreOperators", { count: extra })}>
                        +{extra}
                    </span>
                ) : null}
            </div>
        </div>
    );
}

function BannerNameCell({ banner, name, typeColor, typeLabelKey, sixStars, fiveStars, total, operatorsById }: { banner: IBanner | undefined; name: string; typeColor: string; typeLabelKey: GachaMessageKey; sixStars: number; fiveStars: number; total: number; operatorsById: Map<string, IOperatorIndexEntry> }) {
    const t: BreakdownT = useT("gacha");
    const f = useFormatters();
    const trigger = (
        <button type="button" className="flex cursor-pointer flex-col items-start gap-0.5 rounded-sm text-left focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/40">
            <span className="font-medium font-sans text-[13px] text-foreground leading-snug">{name}</span>
            <span className="font-mono text-[9.5px] uppercase tracking-[0.14em]" style={{ color: typeColor }}>
                {t(typeLabelKey)}
            </span>
        </button>
    );

    if (!banner) {
        // Banner not present in static data - happens for older retired pools
        // outside the current gamedata snapshot. Show the name without hover.
        return trigger;
    }

    const ruleMessageKey = BANNER_RULE_TYPE_LABEL_KEYS[banner.gachaRuleType];
    const ruleLabel = ruleMessageKey ? t(ruleMessageKey) : banner.gachaRuleType;
    const now = Date.now() / 1000;
    const isActive = banner.openTime <= now && now <= banner.endTime;
    const isUpcoming = now < banner.openTime;

    return (
        <HoverCard>
            <HoverCardTrigger render={trigger} />
            <HoverCardContent className="w-80 p-4">
                <div className="flex flex-col gap-3">
                    <div className="flex flex-col gap-1">
                        <span className="font-mono text-[9.5px] uppercase tracking-[0.14em]" style={{ color: typeColor }}>
                            {t(typeLabelKey)} · {ruleLabel}
                        </span>
                        <span className="font-sans font-semibold text-[14px] text-foreground leading-snug">{banner.gachaPoolName}</span>
                        {banner.gachaPoolSummary && banner.gachaPoolSummary !== "-" ? <span className="font-sans text-[12px] text-muted-foreground leading-snug">{banner.gachaPoolSummary}</span> : null}
                    </div>

                    <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
                        <span className="font-sans text-[12px] text-foreground tabular-nums leading-none">{t("history.breakdown.dateRange", { open: fmtDateFromSeconds(banner.openTime, f), close: fmtDateFromSeconds(banner.endTime, f) })}</span>
                        <span className="inline-flex items-center gap-1.5 font-mono text-[9.5px] uppercase leading-none tracking-[0.14em]" style={{ color: isActive ? "oklch(0.78 0.18 145)" : isUpcoming ? "oklch(0.78 0.16 220)" : "var(--muted-foreground)" }}>
                            <span className="block h-1.5 w-1.5 rounded-full bg-current" aria-hidden />
                            {t(BANNER_STATUS_LABEL_KEYS[isActive ? "active" : isUpcoming ? "upcoming" : "ended"])}
                        </span>
                    </div>

                    <div className="grid grid-cols-3 gap-2">
                        <div className="flex flex-col gap-0.5">
                            <span className="font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em]">{t("history.breakdown.stat.pulls")}</span>
                            <span className="font-sans font-semibold text-[14px] text-foreground tabular-nums">{f.number(total)}</span>
                        </div>
                        <div className="flex flex-col gap-0.5">
                            <span className="font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em]">{t("history.breakdown.stat.sixStars")}</span>
                            <span className="font-sans font-semibold text-[14px] tabular-nums" style={{ color: "#ff7f27" }}>
                                {f.number(sixStars)}
                            </span>
                        </div>
                        <div className="flex flex-col gap-0.5">
                            <span className="font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em]">{t("history.breakdown.stat.fiveStars")}</span>
                            <span className="font-sans font-semibold text-[14px] tabular-nums" style={{ color: "#e9d28a" }}>
                                {f.number(fiveStars)}
                            </span>
                        </div>
                    </div>

                    {banner.guarantee5Avail ? <div className="font-mono text-[10.5px] text-muted-foreground">{t("history.breakdown.guarantee", { name: banner.guaranteeName ?? t("history.breakdown.guaranteeDefault"), count: banner.guarantee5Count })}</div> : null}

                    {banner.featured6.length > 0 || banner.featured5.length > 0 ? (
                        <div className="flex flex-col gap-2.5 border-border/60 border-t pt-3">
                            <FeaturedRow ids={banner.featured6} star={6} operatorsById={operatorsById} />
                            <FeaturedRow ids={banner.featured5} star={5} operatorsById={operatorsById} />
                        </div>
                    ) : null}

                    <div className="break-all font-mono text-[10px] text-muted-foreground/70 tabular-nums">{banner.gachaPoolId}</div>
                </div>
            </HoverCardContent>
        </HoverCard>
    );
}

export function BannerBreakdown({ records, bannersById, operatorsById, isLoading }: IBannerBreakdownProps) {
    const t: BreakdownT = useT("gacha");
    const f = useFormatters();
    const banners = useMemo<IBannerStat[]>(() => {
        if (!records) return [];
        return [...groupByPool(records.limited.records, "limited", bannersById), ...groupByPool(records.linkage.records, "linkage", bannersById), ...groupByPool(records.regular.records, "regular", bannersById), ...groupByPool(records.special.records, "special", bannersById)].sort((a, b) => b.lastPullAt - a.lastPullAt);
    }, [records, bannersById]);

    if (isLoading) {
        return (
            <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-[22px_24px]">
                <div className="h-4 w-32 animate-pulse rounded bg-muted" />
                <div className="flex flex-col gap-2">
                    {Array.from({ length: 5 }).map((_, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: skeleton
                        <div key={i} className="flex items-center gap-4 not-last:border-border/60 not-last:border-b py-2">
                            <div className="h-3 flex-1 animate-pulse rounded bg-muted" />
                            <div className="h-3 w-14 animate-pulse rounded bg-muted" />
                            <div className="h-3 w-10 animate-pulse rounded bg-muted" />
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
                <Kicker>{t("history.breakdown.kicker")}</Kicker>
                <div className="py-6 text-center font-sans text-muted-foreground text-sm">{t("history.breakdown.empty")}</div>
            </section>
        );
    }

    const maxTotal = Math.max(...banners.map((b) => b.total));

    return (
        <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-[22px_24px]">
            <header>
                <Kicker className="mb-1.5">{t("history.breakdown.kicker")}</Kicker>
                <h2 className="m-0 font-sans font-semibold text-[20px] text-foreground leading-[1.15] tracking-[-0.02em] sm:text-[22px]">{t("history.breakdown.title")}</h2>
            </header>

            <div className="-mx-1 max-h-120 overflow-y-auto px-1 [scrollbar-color:var(--border)_transparent] [scrollbar-width:thin]">
                <table className="w-full border-collapse">
                    <thead className="sticky top-0 z-10 bg-card">
                        <tr>
                            <th className="whitespace-nowrapd border-border border-b bg-card px-2 py-2 text-left font-medium font-mono text-[9.5px] text-muted-foregroun uppercase tracking-[0.14em]">{t("history.breakdown.col.banner")}</th>
                            <th className="hidden whitespace-nowrap border-border border-b bg-card px-2 py-2 text-left font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em] sm:table-cell">{t("history.breakdown.col.lastPull")}</th>
                            <th className="whitespace-nowrap border-border border-b bg-card px-2 py-2 text-right font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em]">{t("history.breakdown.col.pulls")}</th>
                            <th className="hidden whitespace-nowrap border-border border-b bg-card px-2 py-2 text-right font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em] sm:table-cell">{t("history.breakdown.col.sixStars")}</th>
                            <th className="hidden w-36 whitespace-nowrap border-border border-b bg-card px-2 py-2 text-left font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em] lg:table-cell">{t("history.breakdown.col.distribution")}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {banners.map((banner) => {
                            const barPct = (banner.total / maxTotal) * 100;
                            const typeColor = TYPE_COLORS[banner.gachaType];
                            const meta = bannersById.get(banner.poolId);
                            return (
                                <tr key={`${banner.gachaType}-${banner.poolId}`} className="not-last:border-border/50 not-last:border-b">
                                    <td className="px-2 py-2.5 align-middle">
                                        <BannerNameCell banner={meta} name={banner.poolName} typeColor={typeColor} typeLabelKey={banner.typeLabelKey} sixStars={banner.sixStars} fiveStars={banner.fiveStars} total={banner.total} operatorsById={operatorsById} />
                                    </td>
                                    <td className="hidden whitespace-nowrap px-2 py-2.5 align-middle font-mono text-[11px] text-muted-foreground tabular-nums sm:table-cell">{fmtDate(banner.lastPullAt, f)}</td>
                                    <td className="whitespace-nowrap px-2 py-2.5 text-right align-middle font-mono font-semibold text-[12px] text-foreground tabular-nums">{f.number(banner.total)}</td>
                                    <td className="hidden px-2 py-2.5 text-right align-middle font-mono text-[11px] tabular-nums sm:table-cell">
                                        <span style={{ color: "#ff7f27" }}>{banner.sixStars > 0 ? `${banner.sixStars}×6★` : "-"}</span>
                                    </td>
                                    <td className="hidden w-36 px-2 py-2.5 align-middle lg:table-cell">
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
