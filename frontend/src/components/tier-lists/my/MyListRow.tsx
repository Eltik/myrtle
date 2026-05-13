import { Link } from "@tanstack/react-router";
import { CopyIcon, ExternalLinkIcon, HeartIcon, LayoutGridIcon, MoreHorizontalIcon, PencilIcon, ShieldCheckIcon, TrashIcon } from "lucide-react";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "#/components/ui/menu";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ITierListBrowseItem } from "#/lib/api/tier-lists";
import { formatNumberCompact } from "#/lib/utils";

interface IMyListRowProps {
    tl: ITierListBrowseItem;
    onEdit: (slug: string) => void;
    onDelete: (slug: string) => void;
    onCopyLink: (slug: string) => void;
}

const PREVIEW_OPS = 6;

export function MyListRow({ tl, onEdit, onDelete, onCopyLink }: IMyListRowProps) {
    const previewOps = tl.tiers.flatMap((t) => t.operators).slice(0, PREVIEW_OPS);
    const totalOps = tl.tiers.reduce((sum, t) => sum + t.operators.length, 0);
    const overflow = Math.max(0, totalOps - previewOps.length);
    const isOfficial = tl.listType === "official";
    const isEmpty = totalOps === 0;

    return (
        <article className="group relative flex items-center gap-3 rounded-lg border border-border bg-card px-3 py-2.5 shadow-[0_1px_2px_oklch(0_0_0/0.04)] transition-[border-color,box-shadow] hover:border-[color-mix(in_srgb,var(--foreground)_22%,var(--border))] hover:shadow-[0_6px_16px_oklch(0_0_0/0.06)]">
            <div className="flex shrink-0 items-center gap-0.5">
                {previewOps.length === 0 ? (
                    <div className="flex h-9 w-24 items-center justify-center rounded-md border border-dashed border-border bg-muted/30 font-sans text-[10.5px] italic text-muted-foreground">empty</div>
                ) : (
                    <>
                        {previewOps.map((op) => (
                            <span key={op.id} className="inline-flex h-9 w-9 shrink-0 items-center justify-center overflow-hidden rounded-md border border-border bg-muted" title={op.name}>
                                <OperatorAvatar charId={op.id} name={op.name} />
                            </span>
                        ))}
                        {overflow > 0 && <span className="inline-flex h-9 min-w-9 items-center justify-center rounded-md border border-border bg-muted/60 px-1 font-mono text-[11px] font-bold tabular-nums text-muted-foreground">+{overflow}</span>}
                    </>
                )}
            </div>

            <Link to="/tier-lists/$id" params={{ id: tl.slug }} className="min-w-0 flex-1 no-underline">
                <div className="flex min-w-0 items-center gap-2">
                    <h3 className="m-0 truncate font-sans text-[14px] font-semibold leading-tight tracking-tight text-foreground transition-colors group-hover:text-primary" title={tl.title}>
                        {tl.title || "Untitled list"}
                    </h3>
                    {isOfficial && (
                        <span className="inline-flex shrink-0 items-center gap-1 rounded-full bg-[oklch(0.92_0.13_85)] px-1.5 py-0.5 font-mono text-[9px] font-bold uppercase leading-none tracking-wider text-[oklch(0.32_0.12_75)] shadow-[0_1px_4px_oklch(0.6_0.16_75/0.4)]">
                            <ShieldCheckIcon className="h-2 w-2" aria-hidden="true" />
                            Official
                        </span>
                    )}
                    {!isOfficial && tl.flairLabel && (
                        <span
                            className="inline-flex shrink-0 items-center gap-1 rounded-full px-1.5 py-0.5 font-mono text-[9px] font-bold uppercase leading-none tracking-wider"
                            style={
                                tl.flairColor ? { background: `color-mix(in srgb, ${tl.flairColor} 25%, transparent)`, color: tl.flairColor, border: `1px solid color-mix(in srgb, ${tl.flairColor} 50%, transparent)` } : { background: "var(--muted)", color: "var(--muted-foreground)", border: "1px solid var(--border)" }
                            }
                        >
                            <span className="h-1 w-1 rounded-full" style={{ background: tl.flairColor ?? "currentColor" }} aria-hidden="true" />
                            {tl.flairLabel}
                        </span>
                    )}
                    {isEmpty && <span className="inline-flex shrink-0 items-center rounded-full border border-dashed border-border bg-transparent px-1.5 py-0.5 font-mono text-[9px] font-medium uppercase leading-none tracking-wider text-muted-foreground">Draft</span>}
                </div>
                {tl.description && <p className="m-0 mt-0.5 truncate font-sans text-[11.5px] leading-snug text-muted-foreground">{tl.description}</p>}
            </Link>

            <div className="hidden shrink-0 items-center gap-4 font-mono text-[11px] leading-none tabular-nums text-muted-foreground sm:flex">
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
                <span className="w-16 shrink-0 text-right text-[10.5px] text-muted-foreground/80">{tl.updated}</span>
            </div>

            <Menu>
                <MenuTrigger aria-label="List actions" className="inline-flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center rounded-md border border-transparent text-muted-foreground transition-colors hover:border-border hover:bg-accent hover:text-foreground">
                    <MoreHorizontalIcon className="h-4 w-4" />
                </MenuTrigger>
                <MenuPopup align="end" sideOffset={6} className="min-w-44">
                    <MenuItem render={<Link to="/tier-lists/$id" params={{ id: tl.slug }} />}>
                        <ExternalLinkIcon />
                        Open
                    </MenuItem>
                    <MenuItem render={<Link to="/tier-lists/my/$id/edit" params={{ id: tl.slug }} />}>
                        <LayoutGridIcon />
                        Edit Tierlist
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
        </article>
    );
}
