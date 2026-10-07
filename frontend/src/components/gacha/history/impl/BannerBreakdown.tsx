import { ChevronDown } from "lucide-react";
import { Fragment, useMemo, useState } from "react";
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

/** Own chrome plus the shared banner bucket, status and rule-type labels. */
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
    /** This banner's 6★ pulls, newest first. */
    sixStarPulls: IGachaItem[];
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
            if (item.star === "6") {
                existing.sixStars++;
                existing.sixStarPulls.push(item);
            }
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
                sixStarPulls: item.star === "6" ? [item] : [],
            });
        }
    }
    for (const stat of map.values()) stat.sixStarPulls.sort((a, b) => b.at - a.at);
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

/**
 * The 6★ operators one banner gave the player, revealed under its row. No pull
 * number per operator: a ten-pull shares one timestamp, so the order inside it
 * is not recoverable and any "pull #N" could be off by up to nine.
 */
function SixStarPulls({ pulls, banner, operatorsById }: { pulls: IGachaItem[]; banner: IBanner | undefined; operatorsById: Map<string, IOperatorIndexEntry> }) {
    const t: BreakdownT = useT("gacha");
    const f = useFormatters();
    const ringColor = rarityStarColor(6);
    // Only mark rate-up vs off-banner when the static data names the featured pool.
    const featured = banner && banner.featured6.length > 0 ? new Set(banner.featured6) : null;
    const rateUpCount = featured ? pulls.filter((item) => featured.has(item.charId)).length : 0;

    return (
        <div className="motion-safe:fade-in motion-safe:slide-in-from-top-1 flex flex-col gap-2 motion-safe:animate-in motion-safe:duration-200">
            {featured ? (
                <p className="m-0 font-mono text-[10px] text-muted-foreground uppercase tabular-nums tracking-[0.12em]">
                    <span style={{ color: ringColor }}>{t("history.breakdown.rateUpCount", { count: rateUpCount })}</span> · {t("history.breakdown.offBannerCount", { count: pulls.length - rateUpCount })}
                </p>
            ) : null}
            {/* Equal-width cells instead of a ragged wrap, so names line up and truncate at one width. */}
            <ul className="m-0 grid list-none grid-cols-[repeat(auto-fill,minmax(10.5rem,1fr))] gap-1.5 p-0">
                {pulls.map((item, i) => {
                    const name = item.charName || operatorsById.get(item.charId)?.name || item.charId;
                    const isFeatured = featured?.has(item.charId) ?? null;
                    return (
                        // biome-ignore lint/suspicious/noArrayIndexKey: the same operator can land twice in one ten-pull
                        <li key={`${item.charId}-${item.at}-${i}`} className="flex min-w-0 items-center gap-2 rounded-lg border border-border/60 bg-card py-1.5 pr-2.5 pl-1.5">
                            <span className="relative inline-flex h-8 w-8 shrink-0 overflow-hidden rounded-md" style={{ background: "var(--muted)", boxShadow: `inset 0 0 0 1px color-mix(in oklch, ${ringColor} 60%, transparent)` }}>
                                <OperatorAvatar charId={item.charId} name={name} className="block h-full w-full object-cover" />
                            </span>
                            <span className="flex min-w-0 flex-col gap-0.5">
                                <span className="truncate font-sans font-semibold text-[12.5px] text-foreground leading-snug" title={name}>
                                    {name}
                                </span>
                                <span className="flex items-center gap-1.5 font-mono text-[10px] text-muted-foreground tabular-nums leading-none">
                                    {fmtDate(item.at, f)}
                                    {isFeatured === null ? null : (
                                        <span className="uppercase tracking-[0.12em]" style={{ color: isFeatured ? ringColor : undefined }}>
                                            · {t(isFeatured ? "history.breakdown.rateUp" : "history.breakdown.offBanner")}
                                        </span>
                                    )}
                                </span>
                            </span>
                        </li>
                    );
                })}
            </ul>
        </div>
    );
}

export function BannerBreakdown({ records, bannersById, operatorsById, isLoading }: IBannerBreakdownProps) {
    const t: BreakdownT = useT("gacha");
    const f = useFormatters();
    const [expanded, setExpanded] = useState<Set<string>>(() => new Set());
    const toggle = (key: string) =>
        setExpanded((prev) => {
            const next = new Set(prev);
            if (next.has(key)) next.delete(key);
            else next.add(key);
            return next;
        });
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
                            <th className="whitespace-nowrap border-border border-b bg-card px-2 py-2 text-left font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em]">{t("history.breakdown.col.banner")}</th>
                            <th className="hidden whitespace-nowrap border-border border-b bg-card px-2 py-2 text-left font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em] sm:table-cell">{t("history.breakdown.col.lastPull")}</th>
                            <th className="whitespace-nowrap border-border border-b bg-card px-2 py-2 text-right font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em]">{t("history.breakdown.col.pulls")}</th>
                            <th className="whitespace-nowrap border-border border-b bg-card px-2 py-2 text-right font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em]">{t("history.breakdown.col.sixStars")}</th>
                            <th className="hidden w-36 whitespace-nowrap border-border border-b bg-card px-2 py-2 text-left font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em] lg:table-cell">{t("history.breakdown.col.distribution")}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {banners.map((banner) => {
                            const barPct = (banner.total / maxTotal) * 100;
                            const typeColor = TYPE_COLORS[banner.gachaType];
                            const meta = bannersById.get(banner.poolId);
                            const rowKey = `${banner.gachaType}-${banner.poolId}`;
                            const isOpen = expanded.has(rowKey);
                            const panelId = `six-star-pulls-${rowKey}`;
                            return (
                                <Fragment key={rowKey}>
                                    {/*
                                     * The whole row toggles for pointer users; the 6★ button is the
                                     * keyboard and screen-reader control, and its click bubbles up here.
                                     */}
                                    <tr onClick={banner.sixStars > 0 ? () => toggle(rowKey) : undefined} className={`${isOpen ? "bg-muted/40" : "not-last:border-border/50 not-last:border-b"} ${banner.sixStars > 0 ? "cursor-pointer transition-colors hover:bg-muted/40" : ""}`}>
                                        <td className="px-2 py-2.5 align-middle">
                                            <BannerNameCell banner={meta} name={banner.poolName} typeColor={typeColor} typeLabelKey={banner.typeLabelKey} sixStars={banner.sixStars} fiveStars={banner.fiveStars} total={banner.total} operatorsById={operatorsById} />
                                        </td>
                                        <td className="hidden whitespace-nowrap px-2 py-2.5 align-middle font-mono text-[11px] text-muted-foreground tabular-nums sm:table-cell">{fmtDate(banner.lastPullAt, f)}</td>
                                        <td className="whitespace-nowrap px-2 py-2.5 text-right align-middle font-mono font-semibold text-[12px] text-foreground tabular-nums">{f.number(banner.total)}</td>
                                        <td className="whitespace-nowrap px-2 py-2.5 text-right align-middle font-mono text-[11px] tabular-nums">
                                            {banner.sixStars > 0 ? (
                                                <button
                                                    type="button"
                                                    aria-expanded={isOpen}
                                                    aria-controls={panelId}
                                                    aria-label={t(isOpen ? "history.breakdown.hideSixStars" : "history.breakdown.showSixStars", { count: banner.sixStars, banner: banner.poolName })}
                                                    className="-my-1 -mr-1.5 inline-flex min-h-8 pointer-coarse:min-h-11 cursor-pointer touch-manipulation items-center gap-1 rounded-md px-1.5 transition-colors hover:bg-muted focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/40"
                                                    style={{ color: "#ff7f27" }}
                                                >
                                                    {banner.sixStars}×6★
                                                    <ChevronDown className={`h-3.5 w-3.5 transition-transform duration-200 motion-reduce:transition-none ${isOpen ? "rotate-180" : ""}`} aria-hidden />
                                                </button>
                                            ) : (
                                                <span className="text-muted-foreground">-</span>
                                            )}
                                        </td>
                                        <td className="hidden w-36 px-2 py-2.5 align-middle lg:table-cell">
                                            <div className="h-1.5 w-full overflow-hidden rounded-full bg-muted">
                                                <div className="h-full rounded-full" style={{ width: `${barPct}%`, background: typeColor }} />
                                            </div>
                                        </td>
                                    </tr>
                                    {isOpen ? (
                                        <tr id={panelId} className="not-last:border-border/50 not-last:border-b bg-muted/40">
                                            <td colSpan={5} className="px-2 pt-0.5 pb-3">
                                                <SixStarPulls pulls={banner.sixStarPulls} banner={meta} operatorsById={operatorsById} />
                                            </td>
                                        </tr>
                                    ) : null}
                                </Fragment>
                            );
                        })}
                    </tbody>
                </table>
            </div>
        </section>
    );
}
