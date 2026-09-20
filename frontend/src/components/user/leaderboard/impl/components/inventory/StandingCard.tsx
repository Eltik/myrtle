import { Link } from "@tanstack/react-router";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Button } from "#/components/ui/button";
import { ItemIcon } from "#/components/user/profile/impl/components/tabs/Items/ItemIcon";
import type { IItemStanding } from "#/lib/api/user";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { playerIdentity } from "../../identity";
import { toIconEntry } from "../../inventory.helpers";
import type { ICatalogItem } from "../../inventory.types";
import type { messages } from "./StandingCard.messages";

interface IStandingCardProps {
    item: ICatalogItem;
    /** The signed-in player, as the leaderboard identifies them. */
    me: { uid: string; server: string; nickname?: string | null; avatar_id?: string | null };
    /** `null` when the player holds none of the item. */
    standing: IItemStanding | null;
    isLoading?: boolean;
}

/**
 * The signed-in player's standing for the ranked item. Same skeleton as
 * `YouCard` (avatar, rank line, three metrics, profile button) so the sidebar
 * keeps its shape when the ranking changes.
 */
export function StandingCard({ item, me, standing, isLoading }: IStandingCardProps) {
    const t: TypedT<typeof messages> = useT("user");
    const f = useFormatters();

    if (isLoading) return null;

    const { nickname, initials, avatarSrc } = playerIdentity(me);
    const rank = standing?.rank_global ?? null;
    const percentile = standing && standing.holders_global > 0 ? Math.max(0.1, (standing.rank_global / standing.holders_global) * 100) : null;

    return (
        <aside className="relative overflow-hidden rounded-xl border border-border bg-card p-4 shadow-[0_1px_2px_rgb(0_0_0/0.04)]">
            <div aria-hidden className="pointer-events-none absolute inset-0" style={{ background: "radial-gradient(ellipse 80% 60% at 50% 0%, color-mix(in srgb, var(--primary) 8%, transparent), transparent 60%)" }} />
            <div className="relative mb-3 flex items-center justify-between gap-2 font-medium font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.16em]">
                <span className="min-w-0 truncate">{t("leaderboard.itemYou.title")}</span>
                <span className="shrink-0 rounded-full border border-border bg-card px-2 py-0.5 text-foreground tabular-nums">#{rank ?? "-"}</span>
            </div>
            <div className="relative flex items-center gap-3">
                <Avatar className="size-12 rounded-2xl">
                    <AvatarImage src={avatarSrc} alt={nickname} />
                    <AvatarFallback className="rounded-2xl text-sm">{initials}</AvatarFallback>
                </Avatar>
                <div className="min-w-0">
                    <div className="truncate font-bold font-sans text-base text-foreground leading-tight tracking-tight">{nickname}</div>
                    <div className="mt-1 font-mono text-[11px] text-muted-foreground leading-none">
                        {t("leaderboard.itemYou.rank")} <b className="font-bold text-foreground">#{rank ?? "-"}</b>
                        {percentile != null && (
                            <>
                                {t("leaderboard.itemYou.topPercentile")}
                                <b className="font-bold text-foreground">{percentile.toFixed(1)}%</b>
                            </>
                        )}
                    </div>
                </div>
            </div>
            {standing ? (
                <div className="relative mt-3.5 grid grid-cols-3 gap-2">
                    <Metric k={t("leaderboard.itemYou.metric.held")}>
                        <span className="inline-flex min-w-0 items-center gap-1.5">
                            <ItemIcon item={toIconEntry(item)} size={18} className="rounded-sm" />
                            <span className="truncate">{f.compact(standing.quantity)}</span>
                        </span>
                    </Metric>
                    <Metric k={t("leaderboard.itemYou.metric.server", { server: me.server.toUpperCase() })}>#{f.number(standing.rank_server)}</Metric>
                    <Metric k={t("leaderboard.itemYou.metric.holders")}>{f.compact(standing.holders_global)}</Metric>
                </div>
            ) : (
                <p className="relative mt-3.5 mb-0 font-sans text-[12.5px] text-muted-foreground leading-snug">{t("leaderboard.itemYou.none")}</p>
            )}
            <div className="relative mt-3.5">
                <Button
                    variant="default"
                    size="sm"
                    className="w-full"
                    render={
                        <Link to="/user/$id" params={{ id: me.uid }}>
                            {t("leaderboard.itemYou.viewProfile")}
                        </Link>
                    }
                />
            </div>
        </aside>
    );
}

function Metric({ k, children }: { k: string; children: React.ReactNode }) {
    return (
        <div className="flex min-w-0 flex-col gap-1.5 rounded-[10px] border border-border bg-muted/60 p-2.5">
            <span className="truncate font-medium font-mono text-[9.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{k}</span>
            <span className="flex min-w-0 items-center font-bold font-sans text-base text-foreground tabular-nums leading-none tracking-tight">{children}</span>
        </div>
    );
}
