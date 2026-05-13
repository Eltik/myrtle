import { Link } from "@tanstack/react-router";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ITierListBrowseItem } from "#/lib/api/tier-lists";
import { formatNumberCompact, getAvatarById } from "#/lib/utils";
import styles from "./BrowseCard.module.css";
import { buildThumbRows, MAX_THUMB_TIERS } from "./shared";

type Size = "default" | "trending";

interface IBrowseCardProps {
    tl: ITierListBrowseItem;
    size?: Size;
    rank?: number;
    onOpen?: (slug: string) => void;
}

export default function BrowseCard({ tl, size = "default", rank, onOpen }: IBrowseCardProps) {
    const rows = buildThumbRows(tl);
    const isOfficial = tl.listType === "official";
    const trending = size === "trending";
    const hasOps = rows.some((r) => r.visible.length > 0);
    const showCornerRibbon = trending || typeof rank === "number";
    const showOfficial = isOfficial;
    const showFlair = !isOfficial && Boolean(tl.flairLabel);

    return (
        <Link to="/tier-lists/$id" params={{ id: tl.slug }} className={`${styles.card} group`} aria-labelledby={`tl-${tl.id}-title`} onClick={() => onOpen?.(tl.slug)}>
            <div className={styles.thumb} data-rows={hasOps ? rows.length : 0}>
                {showCornerRibbon && (
                    <span className={styles.cornerRibbon}>
                        <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                            <path d="M13.5 2c.5 3.4 4.2 4.6 4.2 8 0 2.5-2 4.4-4.7 4.4-2 0-3.5-1-4-2.4-.6 1-1.5 1.7-1.5 3.2 0 2 1.5 3.8 4.5 3.8 3.5 0 6-2.4 6-5.7 0-4.5-4.5-6.6-4.5-11.3Z" />
                        </svg>
                        {typeof rank === "number" ? `#${rank}` : "Hot"}
                    </span>
                )}

                {showOfficial && (
                    <span className={`${styles.cornerBadge} ${styles.cornerBadgeOfficial}`}>
                        <svg viewBox="0 0 24 24" aria-hidden="true">
                            <path d="M12 2 9.6 4.4 6.3 4l-.6 3.3L2.5 9 4 12l-1.5 3 3.2 1.7.6 3.3 3.3-.4L12 22l2.4-2.4 3.3.4.6-3.3L21.5 15 20 12l1.5-3-3.2-1.7-.6-3.3L14.4 4.4Zm-1.2 13.4-3.4-3.4 1.4-1.4 2 2 4.4-4.4 1.4 1.4Z" />
                        </svg>
                        Official
                    </span>
                )}

                {showFlair && (
                    <span className={`${styles.cornerBadge} ${styles.cornerBadgeFlair}`} style={tl.flairColor ? { background: `color-mix(in srgb, ${tl.flairColor} 90%, oklch(0 0 0 / 0.2))`, color: "white" } : { background: "oklch(0 0 0 / 0.6)", color: "white" }}>
                        {tl.flairLabel}
                    </span>
                )}

                {!hasOps ? (
                    <div className={styles.emptyThumb}>Empty draft</div>
                ) : (
                    <>
                        {rows.map((row) => (
                            <div key={row.name} className={styles.tierRow} style={{ ["--row-color" as string]: row.color }}>
                                <span className={styles.tierPill} title={`Tier ${row.name}`}>
                                    {row.name}
                                </span>
                                <div className={styles.tierOps}>
                                    {row.visible.map((op) => (
                                        <span key={op.id} className={styles.op} title={op.name}>
                                            <OperatorAvatar charId={op.id} name={op.name} />
                                        </span>
                                    ))}
                                    {row.overflow > 0 && <span className={styles.opOverflow}>+{row.overflow}</span>}
                                </div>
                            </div>
                        ))}
                        {tl.tiers.length > MAX_THUMB_TIERS && <div className={styles.thumbFade} aria-hidden="true" />}
                    </>
                )}
            </div>

            <div className="flex flex-1 flex-col gap-2 px-3.5 pt-3 pb-3">
                <h3 id={`tl-${tl.id}-title`} className={`m-0 font-sans font-semibold leading-snug tracking-tight text-foreground transition-colors group-hover:text-primary ${trending ? "line-clamp-2 text-[16px]" : "line-clamp-1 text-[15px]"}`} title={tl.title}>
                    {tl.title}
                </h3>

                <div className="flex min-w-0 items-center gap-2 font-mono text-[11px] leading-none tabular-nums text-muted-foreground">
                    <span className="flex min-w-0 items-center gap-1.5">
                        <Avatar className="h-4 w-4 shrink-0 rounded-full border border-border bg-linear-to-br from-muted to-border font-sans text-[8px] font-semibold text-foreground">
                            {tl.author.avatarId && <AvatarImage src={getAvatarById(tl.author.avatarId)} alt="" />}
                            <AvatarFallback>{tl.author.name.charAt(0).toUpperCase()}</AvatarFallback>
                        </Avatar>
                        <span className="min-w-0 truncate font-sans text-[11.5px] font-medium text-foreground">{tl.author.name}</span>
                        <span className="shrink-0 opacity-50">·</span>
                        <span className="shrink-0 text-[10.5px]">{tl.updated}</span>
                    </span>

                    <span className="ml-auto flex shrink-0 items-center gap-2.5">
                        <span className="inline-flex items-center gap-1" title={`${tl.views.toLocaleString()} views`}>
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3 opacity-70" aria-hidden="true">
                                <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
                                <circle cx="12" cy="12" r="3" />
                            </svg>
                            <span className="font-semibold text-foreground">{formatNumberCompact(tl.views)}</span>
                        </span>
                        <span className="inline-flex items-center gap-1" title={`${tl.favorites.toLocaleString()} favorites`}>
                            <svg viewBox="0 0 24 24" fill="currentColor" className="h-2.75 w-2.75 opacity-70" aria-hidden="true">
                                <path d="M12 21s-7-4.5-9.5-9C.7 8.7 2.5 5 6 5c2 0 3.5 1 4 2.5C10.5 6 12 5 14 5c3.5 0 5.3 3.7 3.5 7-2.5 4.5-9.5 9-9.5 9Z" />
                            </svg>
                            <span className="font-semibold text-foreground">{formatNumberCompact(tl.favorites)}</span>
                        </span>
                        {trending && tl.views24h > 0 && (
                            <span className="inline-flex items-center gap-0.5 text-primary" title={`${tl.views24h.toLocaleString()} views in the last 24h`}>
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3" aria-hidden="true">
                                    <path d="m6 14 6-6 6 6" />
                                </svg>
                                <span className="font-bold">{formatNumberCompact(tl.views24h)}</span>
                            </span>
                        )}
                    </span>
                </div>
            </div>
        </Link>
    );
}
