import { Link } from "@tanstack/react-router";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import type { ITierListBrowseItem } from "#/lib/api/tier-lists";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { getAvatarById } from "#/lib/utils";
import type { messages } from "./BrowseCard.messages";
import styles from "./BrowseCard.module.css";
import { CardStat } from "./CardStat";
import { buildThumbRows, MAX_THUMB_TIERS } from "./shared";
import { ThumbTierRow } from "./ThumbTierRow";

interface IBrowseCardProps {
    tl: ITierListBrowseItem;
    /** "trending" gives the title two lines and shows the 24h view count. */
    size?: "default" | "trending";
    rank?: number;
    onOpen?: (slug: string) => void;
}

export default function BrowseCard({ tl, size = "default", rank, onOpen }: IBrowseCardProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const f = useFormatters();
    const rows = buildThumbRows(tl);
    const isOfficial = tl.listType === "official";
    const trending = size === "trending";
    const hasOps = rows.some((r) => r.operators.length > 0);
    const showCornerRibbon = trending || typeof rank === "number";

    return (
        <Link to="/tier-lists/$id" params={{ id: tl.slug }} className={`${styles.card} group`} aria-labelledby={`tl-${tl.id}-title`} onClick={() => onOpen?.(tl.slug)}>
            <div className={styles.thumb} data-rows={hasOps ? rows.length : 0}>
                {showCornerRibbon && (
                    <span className={styles.cornerRibbon}>
                        <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                            <path d="M13.5 2c.5 3.4 4.2 4.6 4.2 8 0 2.5-2 4.4-4.7 4.4-2 0-3.5-1-4-2.4-.6 1-1.5 1.7-1.5 3.2 0 2 1.5 3.8 4.5 3.8 3.5 0 6-2.4 6-5.7 0-4.5-4.5-6.6-4.5-11.3Z" />
                        </svg>
                        {typeof rank === "number" ? `#${rank}` : t("browse.card.hot")}
                    </span>
                )}

                {isOfficial && (
                    <span className={`${styles.cornerBadge} ${styles.cornerBadgeOfficial}`}>
                        <svg viewBox="0 0 24 24" aria-hidden="true">
                            <path d="M12 2 9.6 4.4 6.3 4l-.6 3.3L2.5 9 4 12l-1.5 3 3.2 1.7.6 3.3 3.3-.4L12 22l2.4-2.4 3.3.4.6-3.3L21.5 15 20 12l1.5-3-3.2-1.7-.6-3.3L14.4 4.4Zm-1.2 13.4-3.4-3.4 1.4-1.4 2 2 4.4-4.4 1.4 1.4Z" />
                        </svg>
                        {t("browse.card.official")}
                    </span>
                )}

                {!hasOps ? (
                    <div className={styles.emptyThumb}>{t("browse.card.emptyDraft")}</div>
                ) : (
                    <>
                        {rows.map((row) => (
                            <ThumbTierRow key={row.name} row={row} styles={styles} title={t("browse.card.tier", { name: row.name })} />
                        ))}
                        {tl.tiers.length > MAX_THUMB_TIERS && <div className={styles.thumbFade} aria-hidden="true" />}
                    </>
                )}
            </div>

            <div className="flex flex-1 flex-col gap-2 px-3.5 pt-3 pb-3">
                <h3 id={`tl-${tl.id}-title`} className={`m-0 font-sans font-semibold text-foreground leading-snug tracking-tight transition-colors group-hover:text-primary ${trending ? "line-clamp-2 text-[16px]" : "line-clamp-1 text-[15px]"}`} title={tl.title}>
                    {tl.title}
                </h3>

                <div className="flex min-w-0 items-center gap-2 font-sans text-[11px] text-muted-foreground tabular-nums leading-none">
                    <span className="flex min-w-0 items-center gap-1.5">
                        <Avatar className="h-4 w-4 shrink-0 rounded-full border border-border bg-linear-to-br from-muted to-border font-sans font-semibold text-[8px] text-foreground">
                            {tl.author.avatarId && <AvatarImage src={getAvatarById(tl.author.avatarId)} alt="" />}
                            <AvatarFallback>{tl.author.name.charAt(0).toUpperCase()}</AvatarFallback>
                        </Avatar>
                        <span className="min-w-0 truncate font-medium font-sans text-[11.5px] text-foreground">{tl.author.name}</span>
                        <span className="shrink-0 opacity-50">·</span>
                        <span className="shrink-0 text-[10.5px]">{tl.updated}</span>
                    </span>

                    <span className="ml-auto flex shrink-0 items-center gap-2.5">
                        <CardStat kind="views" value={f.compact(tl.views)} title={t("browse.card.views", { count: f.number(tl.views) })} />
                        <CardStat kind="favorites" value={f.compact(tl.favorites)} title={t("browse.card.favorites", { count: f.number(tl.favorites) })} />
                        {trending && tl.views24h > 0 && <CardStat kind="views24h" value={f.compact(tl.views24h)} title={t("browse.card.views24h", { count: f.number(tl.views24h) })} />}
                    </span>
                </div>
            </div>
        </Link>
    );
}
