import { ArrowUpRight } from "lucide-react";
import { useState } from "react";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { Dialog, DialogPopup } from "#/components/ui/dialog";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import type { ITierEntryFull } from "#/lib/api/tier-lists";
import { FALLBACK_TIER_COLORS, formatProfession } from "#/lib/utils";
import { readableTextColor } from "./contrast";
import { OperatorTile } from "./OperatorTile";
import { TierDetailsDialog } from "./TierDetailsDialog";
import styles from "./TierListDetail.module.css";
import { computeTierStats } from "./tierStats";

function tierLabelSize(name: string): "xl" | "lg" | "md" | "sm" | "xs" {
    const trimmed = name.trim();
    const total = trimmed.length;
    const longestWord = trimmed.split(/\s+/).reduce((max, w) => Math.max(max, w.length), 0);
    if (total <= 2) return "xl";
    if (total <= 6 && longestWord <= 6) return "lg";
    if (total <= 12 && longestWord <= 8) return "md";
    if (total <= 20 && longestWord <= 10) return "sm";
    return "xs";
}

interface ITierRowProps {
    tier: ITierEntryFull;
    index: number;
}

export function TierRow({ tier, index }: ITierRowProps) {
    const color = tier.color ?? FALLBACK_TIER_COLORS[index % FALLBACK_TIER_COLORS.length] ?? "var(--primary)";
    const textColor = readableTextColor(color);
    const labelledById = `tier-${tier.id}-label`;
    const totalOps = tier.operators.length;
    const hasDescription = Boolean(tier.description?.trim());

    const [dialogOpen, setDialogOpen] = useState(false);

    const openDialog = () => setDialogOpen(true);

    return (
        <li
            className={styles.tierRow}
            style={{
                ["--row-color" as string]: color,
                ["--tier-fg" as string]: textColor,
                ["--tier-shadow" as string]: textColor === "white" ? "0 1px 0 oklch(0 0 0 / 0.3)" : "0 1px 0 oklch(1 0 0 / 0.5)",
            }}
            aria-labelledby={labelledById}
        >
            <HoverCard>
                <HoverCardTrigger
                    render={
                        <button type="button" onClick={openDialog} className={styles.tierLabel} id={labelledById} data-size={tierLabelSize(tier.name)} aria-label={`Tier ${tier.name}${hasDescription ? ` - ${tier.description}` : ""} - view details`}>
                            <span className={styles.tierLabelGlyph}>{tier.name}</span>
                        </button>
                    }
                />
                <HoverCardContent className="w-80 max-w-[calc(100vw-2rem)] overflow-hidden p-0" sideOffset={10} align="start">
                    <TierHoverCard tier={tier} color={color} totalOps={totalOps} hasDescription={hasDescription} onOpen={openDialog} />
                </HoverCardContent>
            </HoverCard>

            {totalOps === 0 ? (
                <div className={styles.emptyTier}>No operators in this tier yet.</div>
            ) : (
                <div className={styles.tierOps}>
                    {tier.operators.map((op) => (
                        <OperatorTile key={op.id} operator={op} />
                    ))}
                </div>
            )}

            <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
                <DialogPopup className="max-w-2xl overflow-hidden p-0">
                    <TierDetailsDialog tier={tier} color={color} />
                </DialogPopup>
            </Dialog>
        </li>
    );
}

interface IHoverProps {
    tier: ITierEntryFull;
    color: string;
    totalOps: number;
    hasDescription: boolean;
    onOpen: () => void;
}

function TierHoverCard({ tier, color, totalOps, hasDescription, onOpen }: IHoverProps) {
    const textColor = readableTextColor(color);
    const stats = computeTierStats(tier);
    const previewOps = tier.operators.slice(0, 5);
    const remaining = totalOps - previewOps.length;

    return (
        <button type="button" onClick={onOpen} className="group flex w-full cursor-pointer flex-col bg-popover text-left no-underline transition-colors" aria-label={`Open tier ${tier.name} details`}>
            <div className="relative flex items-center gap-3.5 border-border border-b px-4 pt-4 pb-3.5" style={{ background: `linear-gradient(135deg, color-mix(in srgb, ${color} 14%, transparent), transparent 70%)` }}>
                <span className="absolute inset-x-0 top-0 h-0.75" style={{ background: color }} aria-hidden="true" />
                <span
                    className="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg font-extrabold font-features-['ss01','cv11'] font-sans text-[22px] leading-none tracking-tight"
                    style={{
                        background: color,
                        color: textColor,
                        textShadow: textColor === "white" ? "0 1px 0 oklch(0 0 0 / 0.25)" : "0 1px 0 oklch(1 0 0 / 0.5)",
                    }}
                    aria-hidden="true"
                >
                    {tier.name.length <= 2 ? tier.name : tier.name.charAt(0)}
                </span>
                <div className="min-w-0 flex-1">
                    <p className="m-0 font-bold font-mono text-[10px] text-muted-foreground/80 uppercase leading-none tracking-[0.18em]">Tier</p>
                    <h3 className="wrap-break-word m-0 mt-1.5 font-bold font-features-['ss01','cv11'] font-sans text-[16.5px] text-foreground leading-[1.15] tracking-[-0.01em]">{tier.name}</h3>
                </div>
                <span className="shrink-0 self-start rounded-full border border-border bg-card px-2 py-1 font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tabular-nums leading-none tracking-[0.06em]">
                    <span className="text-foreground">{totalOps}</span>
                </span>
            </div>

            <div className="flex flex-col gap-3.5 px-4 py-3.5">
                {hasDescription ? <p className="m-0 line-clamp-3 font-sans text-[13.5px] text-foreground leading-[1.55] tracking-[-0.005em]">{tier.description}</p> : <p className="m-0 font-sans text-[12.5px] text-muted-foreground italic leading-[1.55]">No description provided for this tier.</p>}

                {totalOps > 0 && (
                    <>
                        <div className="flex items-center -space-x-1.5">
                            {previewOps.map((op) => (
                                <span key={op.id} className="flex h-7 w-7 shrink-0 items-center justify-center overflow-hidden rounded-full border-2 border-popover bg-muted font-semibold text-[10px] text-foreground" title={op.name}>
                                    <OperatorAvatar charId={op.id} name={op.name} />
                                </span>
                            ))}
                            {remaining > 0 && <span className="flex h-7 shrink-0 items-center justify-center rounded-full border-2 border-popover bg-muted px-2 font-bold font-mono text-[10.5px] text-muted-foreground tabular-nums">+{remaining}</span>}
                        </div>

                        {stats.profession.length > 0 && (
                            <div className="flex flex-wrap items-center gap-1.5">
                                {stats.profession.slice(0, 4).map(({ profession, count }) => (
                                    <span key={profession} className="inline-flex items-center gap-1 rounded-md border border-border bg-muted/50 px-1.5 py-1 font-sans font-semibold text-[11.5px] text-foreground leading-none" title={`${count} ${formatProfession(profession)}${count === 1 ? "" : "s"}`}>
                                        <ClassIcon profession={profession} size={12} />
                                        <span className="font-bold font-mono text-muted-foreground tabular-nums">{count}</span>
                                    </span>
                                ))}
                                {stats.profession.length > 4 && <span className="font-mono font-semibold text-[10.5px] text-muted-foreground">+{stats.profession.length - 4}</span>}
                            </div>
                        )}
                    </>
                )}
            </div>

            <div className="flex items-center justify-between gap-2 border-border border-t bg-muted/60 px-4 py-2.5 font-bold font-sans text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em] transition-colors group-hover:bg-[color-mix(in_srgb,var(--ring)_10%,transparent)] group-hover:text-foreground">
                <span>View tier details</span>
                <ArrowUpRight className="h-3.5 w-3.5 transition-transform group-hover:translate-x-0.5" aria-hidden="true" />
            </div>
        </button>
    );
}
