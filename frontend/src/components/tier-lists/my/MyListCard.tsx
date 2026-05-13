import { Link } from "@tanstack/react-router";
import { CopyIcon, ExternalLinkIcon, HeartIcon, MoreHorizontalIcon, PencilIcon, ShieldCheckIcon, TrashIcon } from "lucide-react";
import type { IOperator } from "#/components/home/impl/data";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "#/components/ui/menu";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ITierListBrowseItem } from "#/lib/api/tier-lists";
import { FALLBACK_TIER_COLORS, formatNumberCompact } from "#/lib/utils";
import styles from "./MyListCard.module.css";

const MAX_THUMB_TIERS = 5;
const OPS_PER_ROW_BY_COUNT: Record<number, number> = { 1: 4, 2: 6, 3: 7 };
const DEFAULT_OPS_PER_ROW = 9;

interface IMyListCardProps {
    tl: ITierListBrowseItem;
    onEdit: (slug: string) => void;
    onDelete: (slug: string) => void;
    onCopyLink: (slug: string) => void;
}

interface IThumbRow {
    name: string;
    color: string;
    visible: IOperator[];
    overflow: number;
}

function buildThumb(tl: ITierListBrowseItem): IThumbRow[] {
    const slice = tl.tiers.slice(0, MAX_THUMB_TIERS);
    const opsPerRow = OPS_PER_ROW_BY_COUNT[slice.length] ?? DEFAULT_OPS_PER_ROW;
    return slice.map((t, idx): IThumbRow => {
        const color = t.color ?? FALLBACK_TIER_COLORS[idx % FALLBACK_TIER_COLORS.length] ?? "var(--primary)";
        const visible = t.operators.slice(0, opsPerRow);
        const overflow = Math.max(0, t.operators.length - visible.length);
        return { name: t.name, color, visible, overflow };
    });
}

export function MyListCard({ tl, onEdit, onDelete, onCopyLink }: IMyListCardProps) {
    const rows = buildThumb(tl);
    const hasOps = rows.some((r) => r.visible.length > 0);
    const isEmpty = !hasOps;
    const isOfficial = tl.listType === "official";

    return (
        <article className={`${styles.card} group`} aria-labelledby={`my-tl-${tl.id}-title`}>
            <Link to="/tier-lists/$id" params={{ id: tl.slug }} className={styles.thumbLink}>
                <div className={styles.thumb} data-rows={hasOps ? rows.length : 0}>
                    {isEmpty && <span className={`${styles.cornerBadge} ${styles.cornerBadgeDraft}`}>Empty draft</span>}
                    {isOfficial && (
                        <span className={`${styles.cornerBadge} ${styles.cornerBadgeOfficial}`}>
                            <ShieldCheckIcon className="h-2.5 w-2.5" aria-hidden="true" />
                            Official
                        </span>
                    )}
                    {!isOfficial && tl.flairLabel && (
                        <span className={`${styles.cornerBadge} ${styles.cornerBadgeFlair}`} style={tl.flairColor ? { background: `color-mix(in srgb, ${tl.flairColor} 90%, oklch(0 0 0 / 0.2))`, color: "white" } : { background: "oklch(0 0 0 / 0.6)", color: "white" }}>
                            {tl.flairLabel}
                        </span>
                    )}

                    {isEmpty ? (
                        <div className={styles.emptyThumb}>No operators placed yet</div>
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
            </Link>

            <div className="flex flex-1 flex-col gap-2 px-3.5 pt-3 pb-3">
                <div className="flex min-w-0 items-start gap-2">
                    <Link to="/tier-lists/$id" params={{ id: tl.slug }} className="min-w-0 flex-1 no-underline">
                        <h3 id={`my-tl-${tl.id}-title`} className="m-0 line-clamp-1 font-sans text-[15px] font-semibold leading-snug tracking-tight text-foreground transition-colors group-hover:text-primary" title={tl.title}>
                            {tl.title || "Untitled list"}
                        </h3>
                        {tl.description && <p className="m-0 mt-0.5 line-clamp-1 font-sans text-[12px] leading-snug text-muted-foreground">{tl.description}</p>}
                    </Link>

                    <Menu>
                        <MenuTrigger aria-label="List actions" className="inline-flex h-7 w-7 shrink-0 cursor-pointer items-center justify-center rounded-md border border-transparent text-muted-foreground transition-colors hover:border-border hover:bg-accent hover:text-foreground">
                            <MoreHorizontalIcon className="h-4 w-4" />
                        </MenuTrigger>
                        <MenuPopup align="end" sideOffset={6} className="min-w-44">
                            <MenuItem render={<Link to="/tier-lists/$id" params={{ id: tl.slug }} />}>
                                <ExternalLinkIcon />
                                Open
                            </MenuItem>
                            <MenuItem onClick={() => onEdit(tl.slug)}>
                                <PencilIcon />
                                Edit details
                            </MenuItem>
                            <MenuItem onClick={() => onCopyLink(tl.slug)}>
                                <CopyIcon />
                                Copy share link
                            </MenuItem>
                            <MenuItem onClick={() => onDelete(tl.slug)} variant="destructive">
                                <TrashIcon />
                                Delete
                            </MenuItem>
                        </MenuPopup>
                    </Menu>
                </div>

                <div className="mt-auto flex items-center gap-3 font-mono text-[11px] leading-none tabular-nums text-muted-foreground">
                    <span className="inline-flex items-center gap-1" title={`${tl.views.toLocaleString()} views`}>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3 opacity-70" aria-hidden="true">
                            <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
                            <circle cx="12" cy="12" r="3" />
                        </svg>
                        <span className="font-semibold text-foreground">{formatNumberCompact(tl.views)}</span>
                    </span>
                    <span className="inline-flex items-center gap-1" title={`${tl.favorites.toLocaleString()} favorites`}>
                        <HeartIcon className="h-2.75 w-2.75 fill-current opacity-70" aria-hidden="true" />
                        <span className="font-semibold text-foreground">{formatNumberCompact(tl.favorites)}</span>
                    </span>
                    {tl.views24h > 0 && (
                        <span className="inline-flex items-center gap-0.5 text-primary" title={`${tl.views24h.toLocaleString()} views in the last 24h`}>
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="h-3 w-3" aria-hidden="true">
                                <path d="m6 14 6-6 6 6" />
                            </svg>
                            <span className="font-bold">{formatNumberCompact(tl.views24h)}</span>
                        </span>
                    )}
                    <span className="ml-auto shrink-0 text-[10.5px] text-muted-foreground/80">{tl.updated}</span>
                </div>
            </div>
        </article>
    );
}
