import { Info } from "lucide-react";
import { ItemIcon } from "#/components/user/profile/impl/components/tabs/Items/ItemIcon";
import { type TypedRichT, useFormatters, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { rarityStarColor } from "#/lib/utils";
import { toIconEntry } from "../inventory.helpers";
import type { ICatalogItem } from "../inventory.types";
import type { messages } from "./Hero.messages";

interface IHeroProps {
    /** Set while the table is ranked by an item; the pill and stats follow it. */
    item: ICatalogItem | null;
    rankedDoctors: number | null;
    /** Top score (0..1) for a score ranking, top quantity for an item ranking. */
    topValue: number | null;
    updatedAt: string | null;
    isLoading?: boolean;
}

function formatRelative(iso: string | null, t: TypedT<typeof messages>): string {
    if (!iso) return "-";
    const ts = Date.parse(iso);
    if (Number.isNaN(ts)) return "-";
    const seconds = Math.max(0, Math.floor((Date.now() - ts) / 1000));
    if (seconds < 60) return t("leaderboard.hero.updated.moments");
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return t("leaderboard.hero.updated.minutes", { count: minutes });
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return t("leaderboard.hero.updated.hours", { count: hours });
    const days = Math.floor(hours / 24);
    return t("leaderboard.hero.updated.days", { count: days });
}

export function Hero({ item, rankedDoctors, topValue, updatedAt, isLoading }: IHeroProps) {
    const t: TypedT<typeof messages> = useT("user");
    const rt: TypedRichT<typeof messages> = useRichT("user");
    const f = useFormatters();
    // The item's rarity colour tints the backdrop, so switching what is
    // ranked reads as the same page in a different light, not a new page.
    const tint = item ? rarityStarColor(item.rarityNum) : "var(--primary)";

    return (
        <section className="relative overflow-hidden rounded-2xl border border-border bg-card shadow-sm">
            <div aria-hidden className="pointer-events-none absolute inset-x-0 -top-px h-px bg-linear-to-r from-transparent via-primary/60 to-transparent" />
            <div
                aria-hidden
                className="pointer-events-none absolute inset-0 transition-[background] duration-300"
                style={{
                    background: `radial-gradient(ellipse 80% 60% at 60% 40%, color-mix(in oklab, ${tint} 7%, transparent), transparent 70%), radial-gradient(ellipse 60% 80% at 80% 20%, color-mix(in oklab, var(--chart-2) 6%, transparent), transparent 70%)`,
                }}
            />

            <div className="relative grid gap-x-7 gap-y-4 px-4 py-4 sm:grid-cols-[1fr_auto] sm:gap-y-5 sm:px-6 sm:py-6">
                <div className="flex min-w-0 flex-col gap-1.5">
                    <span className="font-medium font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.16em]">{t("leaderboard.hero.eyebrow")}</span>
                    <h1 className="m-0 flex flex-wrap items-center gap-2.5 font-bold font-sans text-[22px] text-foreground leading-[1.1] tracking-tight sm:text-[28px]">
                        {t("leaderboard.hero.title")}
                        {item ? (
                            <span
                                className="inline-flex h-6 max-w-full items-center gap-1.5 rounded-full border py-0.5 pr-2.5 pl-1 font-sans font-semibold text-[12px] leading-none"
                                style={{ color: `color-mix(in srgb, ${tint} 45%, var(--foreground))`, borderColor: `color-mix(in srgb, ${tint} 45%, transparent)`, background: `color-mix(in srgb, ${tint} 14%, transparent)` }}
                            >
                                <ItemIcon item={toIconEntry(item)} size={18} className="rounded-sm" />
                                <span className="truncate">{item.name}</span>
                            </span>
                        ) : (
                            <span className="inline-flex h-5.5 items-center rounded-full border border-primary/30 bg-primary/10 px-2 font-mono font-semibold text-[10.5px] text-primary uppercase leading-none tracking-[0.18em]">{t("leaderboard.hero.allTime")}</span>
                        )}
                    </h1>
                    <p className="m-0 max-w-[60ch] font-sans text-[13.5px] text-muted-foreground leading-snug">{item ? t("leaderboard.hero.blurb.item", { item: item.name }) : t("leaderboard.hero.blurb")}</p>
                </div>

                <div className="grid grid-cols-3 items-center gap-3 self-center sm:flex sm:gap-5">
                    <HeroStat label={item ? t("leaderboard.hero.stat.holders") : t("leaderboard.hero.stat.ranked")} value={rankedDoctors == null ? "-" : f.compact(rankedDoctors)} title={rankedDoctors == null ? undefined : f.number(rankedDoctors)} loading={isLoading} />
                    <span aria-hidden className="hidden h-10 w-px bg-border sm:block" />
                    <HeroStat label={item ? t("leaderboard.hero.stat.topHolding") : t("leaderboard.hero.stat.topScore")} value={topValue == null ? "-" : f.compact(topValue)} title={item && topValue != null ? f.number(topValue) : undefined} loading={isLoading} />
                    <span aria-hidden className="hidden h-10 w-px bg-border sm:block" />
                    {item ? <HeroStat label={t("leaderboard.hero.stat.asOf")} value={t("leaderboard.hero.stat.asOf.value")} small loading={isLoading} /> : <HeroStat label={t("leaderboard.hero.stat.updated")} value={formatRelative(updatedAt, t)} small loading={isLoading} />}
                </div>

                <div role="note" className="col-span-full flex items-start gap-2 rounded-lg border border-info/25 bg-info/8 px-3 py-2.5 font-medium font-sans text-[12px] text-foreground/90 leading-snug">
                    <Info className="mt-px size-3.5 shrink-0 text-info" aria-hidden />
                    <span>{rt("leaderboard.hero.notice", { lead: <b className="font-semibold text-foreground">{t("leaderboard.hero.notice.lead")}</b> })}</span>
                </div>
            </div>
        </section>
    );
}

function HeroStat({ label, value, small, loading, title }: { label: string; value: string; small?: boolean; loading?: boolean; title?: string }) {
    return (
        <div className="flex min-w-0 flex-col gap-1" title={title}>
            <span className={`font-bold font-sans text-foreground tabular-nums tracking-tight ${small ? "font-semibold text-sm" : "text-[22px] leading-none"}`}>{loading ? "…" : value}</span>
            <span className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.16em]">{label}</span>
        </div>
    );
}
