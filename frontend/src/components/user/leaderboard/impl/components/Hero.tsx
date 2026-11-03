import type React from "react";
import { Button } from "#/components/ui/button";
import { Kicker } from "#/components/ui/kicker";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { ItemIcon } from "#/components/user/profile/impl/components/tabs/Items/ItemIcon";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { rarityStarColor } from "#/lib/utils";
import { toIconEntry } from "../inventory.helpers";
import type { ICatalogItem } from "../inventory.types";
import type { messages } from "./Hero.messages";

interface IHeroProps {
    /** Set while the table is ranked by an item; the pill and stats follow it. */
    item: ICatalogItem | null;
    rankedPlayers: number | null;
    /** Top score (0..1) for a score ranking, top quantity for an item ranking. */
    topValue: number | null;
    updatedAt: string | null;
    isLoading?: boolean;
    /** The query behind the count and top value failed; those two slots say so and offer a retry. */
    statsError?: boolean;
    onRetryStats?: () => void;
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

export function Hero({ item, rankedPlayers, topValue, updatedAt, isLoading, statsError, onRetryStats }: IHeroProps) {
    const t: TypedT<typeof messages> = useT("user");
    const f = useFormatters();
    // The item's rarity colour tints the backdrop and the item pill, so switching what is
    // ranked reads as the same page in a different light, not a new page.
    const tint = item ? rarityStarColor(item.rarityNum) : "var(--primary)";
    const failure =
        statsError && onRetryStats ? (
            <Button type="button" variant="link" size="xs" onClick={onRetryStats} className="h-auto self-start px-0 font-normal text-muted-foreground text-xs underline sm:h-auto sm:text-xs">
                {t("leaderboard.hero.stat.failed")}
            </Button>
        ) : null;

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
                            <span className="font-medium text-base text-muted-foreground tracking-normal sm:text-lg">{t("leaderboard.hero.allTime")}</span>
                        )}
                    </h1>
                    <p className="m-0 max-w-[60ch] font-sans text-[13.5px] text-muted-foreground leading-snug">{item ? t("leaderboard.hero.blurb.item", { item: item.name }) : t("leaderboard.hero.blurb")}</p>
                </div>

                <div className="grid grid-cols-3 items-start gap-3 self-center sm:flex sm:gap-5">
                    <HeroStat label={item ? t("leaderboard.hero.stat.holders") : t("leaderboard.hero.stat.ranked")} value={rankedPlayers == null ? "–" : f.compact(rankedPlayers)} title={rankedPlayers == null ? undefined : f.number(rankedPlayers)} loading={isLoading} failure={failure} />
                    <span aria-hidden className="hidden h-10 w-px bg-border sm:block" />
                    <HeroStat label={item ? t("leaderboard.hero.stat.topHolding") : t("leaderboard.hero.stat.topScore")} value={topValue == null ? "–" : f.compact(topValue)} title={item && topValue != null ? f.number(topValue) : undefined} loading={isLoading} failure={failure} />
                    <span aria-hidden className="hidden h-10 w-px bg-border sm:block" />
                    {item ? <HeroStat label={t("leaderboard.hero.stat.asOf")} value={t("leaderboard.hero.stat.asOf.value")} small loading={isLoading} /> : <HeroStat label={t("leaderboard.hero.stat.updated")} value={formatRelative(updatedAt, t)} small loading={isLoading} />}
                </div>

                <p className="col-span-full m-0 font-sans text-muted-foreground text-xs leading-snug">{t("leaderboard.hero.notice")}</p>
            </div>
        </section>
    );
}

function HeroStat({ label, value, small, loading, title, failure }: { label: string; value: string; small?: boolean; loading?: boolean; title?: string; failure?: React.ReactNode }) {
    const body = (
        <>
            <span className={`font-bold font-sans text-foreground tabular-nums tracking-tight ${small ? "font-semibold text-sm" : "text-[22px] leading-none"}`}>{loading ? "…" : failure ? "–" : value}</span>
            <Kicker>{label}</Kicker>
        </>
    );
    if (failure)
        return (
            <div className="flex min-w-0 flex-col gap-1">
                {body}
                {failure}
            </div>
        );
    if (!title) return <div className="flex min-w-0 flex-col gap-1">{body}</div>;
    return (
        <Tooltip>
            <TooltipTrigger render={<div className="flex min-w-0 flex-col gap-1" />}>{body}</TooltipTrigger>
            <TooltipPopup>{title}</TooltipPopup>
        </Tooltip>
    );
}
