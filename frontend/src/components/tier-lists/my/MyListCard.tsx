import { Link } from "@tanstack/react-router";
import { CopyIcon, ExternalLinkIcon, LayoutGridIcon, MoreHorizontalIcon, PencilIcon, ShieldCheckIcon, TrashIcon } from "lucide-react";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "#/components/ui/menu";
import type { ITierListBrowseItem } from "#/lib/api/tier-lists";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { CardStat } from "../CardStat";
import { buildThumbRows, MAX_THUMB_TIERS } from "../shared";
import { ThumbTierRow } from "../ThumbTierRow";
import type { messages } from "./MyListCard.messages";
import styles from "./MyListCard.module.css";

interface IMyListCardProps {
    tl: ITierListBrowseItem;
    onEdit: (slug: string) => void;
    onDelete: (slug: string) => void;
    onCopyLink: (slug: string) => void;
}

export function MyListCard({ tl, onEdit, onDelete, onCopyLink }: IMyListCardProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const f = useFormatters();
    const rows = buildThumbRows(tl);
    const isEmpty = rows.every((r) => r.operators.length === 0);
    const isOfficial = tl.listType === "official";

    return (
        <article className={`${styles.card} group`} aria-labelledby={`my-tl-${tl.id}-title`}>
            <Link to="/tier-lists/my/$id/edit" params={{ id: tl.slug }} className={styles.thumbLink}>
                <div className={styles.thumb} data-rows={isEmpty ? 0 : rows.length}>
                    {isEmpty && <span className={`${styles.cornerBadge} ${styles.cornerBadgeDraft}`}>{t("my.card.emptyDraft")}</span>}
                    {isOfficial && (
                        <span className={`${styles.cornerBadge} ${styles.cornerBadgeOfficial}`}>
                            <ShieldCheckIcon className="h-2.5 w-2.5" aria-hidden="true" />
                            {t("my.card.official")}
                        </span>
                    )}
                    {!isOfficial && tl.flairLabel && (
                        <span className={`${styles.cornerBadge} ${styles.cornerBadgeFlair}`} style={tl.flairColor ? { background: `color-mix(in srgb, ${tl.flairColor} 90%, oklch(0 0 0 / 0.2))`, color: "white" } : { background: "oklch(0 0 0 / 0.6)", color: "white" }}>
                            {tl.flairLabel}
                        </span>
                    )}

                    {isEmpty ? (
                        <div className={styles.emptyThumb}>{t("my.card.emptyThumb")}</div>
                    ) : (
                        <>
                            {rows.map((row) => (
                                <ThumbTierRow key={row.name} row={row} styles={styles} title={t("my.card.tier", { name: row.name })} />
                            ))}
                            {tl.tiers.length > MAX_THUMB_TIERS && <div className={styles.thumbFade} aria-hidden="true" />}
                        </>
                    )}
                </div>
            </Link>

            <div className="flex flex-1 flex-col gap-2 px-3.5 pt-3 pb-3">
                <div className="flex min-w-0 items-start gap-2">
                    <Link to="/tier-lists/my/$id/edit" params={{ id: tl.slug }} className="min-w-0 flex-1 no-underline">
                        <h3 id={`my-tl-${tl.id}-title`} className="m-0 line-clamp-1 font-sans font-semibold text-[15px] text-foreground leading-snug tracking-tight transition-colors group-hover:text-primary" title={tl.title}>
                            {tl.title || t("my.card.untitled")}
                        </h3>
                        {tl.description && <p className="m-0 mt-0.5 line-clamp-1 font-sans text-[12px] text-muted-foreground leading-snug">{tl.description}</p>}
                    </Link>

                    <Menu>
                        <MenuTrigger aria-label={t("my.card.actions")} className="inline-flex h-7 w-7 shrink-0 cursor-pointer items-center justify-center rounded-md border border-transparent text-muted-foreground transition-colors hover:border-border hover:bg-accent hover:text-foreground">
                            <MoreHorizontalIcon className="h-4 w-4" />
                        </MenuTrigger>
                        <MenuPopup align="end" sideOffset={6} className="min-w-44">
                            <MenuItem render={<Link to="/tier-lists/$id" params={{ id: tl.slug }} />}>
                                <ExternalLinkIcon />
                                {t("my.card.open")}
                            </MenuItem>
                            <MenuItem render={<Link to="/tier-lists/my/$id/edit" params={{ id: tl.slug }} />}>
                                <LayoutGridIcon />
                                {t("my.card.editBoard")}
                            </MenuItem>
                            <MenuItem onClick={() => onEdit(tl.slug)}>
                                <PencilIcon />
                                {t("my.card.editDetails")}
                            </MenuItem>
                            <MenuItem onClick={() => onCopyLink(tl.slug)}>
                                <CopyIcon />
                                {t("my.card.copyLink")}
                            </MenuItem>
                            <MenuItem onClick={() => onDelete(tl.slug)} variant="destructive">
                                <TrashIcon />
                                {t("my.card.delete")}
                            </MenuItem>
                        </MenuPopup>
                    </Menu>
                </div>

                <div className="mt-auto flex items-center gap-3 font-mono text-[11px] text-muted-foreground tabular-nums leading-none">
                    <CardStat kind="views" value={f.compact(tl.views)} title={t("my.card.views", { count: f.number(tl.views) })} />
                    <CardStat kind="favorites" value={f.compact(tl.favorites)} title={t("my.card.favorites", { count: f.number(tl.favorites) })} />
                    {tl.views24h > 0 && <CardStat kind="views24h" value={f.compact(tl.views24h)} title={t("my.card.views24h", { count: f.number(tl.views24h) })} />}
                    <span className="ml-auto shrink-0 text-[10.5px] text-muted-foreground/80">{tl.updated}</span>
                </div>
            </div>
        </article>
    );
}
