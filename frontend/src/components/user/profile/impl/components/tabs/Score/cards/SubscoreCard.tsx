import { ChevronDown } from "lucide-react";
import { useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IImprovementsResponse } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { Bar, CARD_PADDING, KICKER_TEXT, Kicker, StatCard } from "../../Stats/primitives";
import { formatPct, type ISubscore, toPct, weightShare } from "../helpers";
import { ImprovementsPanel } from "../improvements/ImprovementsPanel";

interface ISubscoreCardProps {
    sub: ISubscore;
    score: number | null | undefined;
    improvements: IImprovementsResponse | null | undefined;
    isImprovementsLoading: boolean;
}

export function SubscoreCard({ sub, score, improvements, isImprovementsLoading }: ISubscoreCardProps) {
    const pct = toPct(score);
    const wShare = weightShare(sub.weight);
    const [open, setOpen] = useState(false);

    return (
        <Collapsible open={open} onOpenChange={setOpen}>
            <StatCard color={sub.color} className={open ? "ring-1 ring-border/40" : undefined}>
                <CollapsibleTrigger render={<button type="button" className={cn("group flex w-full flex-col gap-5 text-left", CARD_PADDING)} aria-expanded={open} aria-label={`${sub.label} details - ${open ? "collapse" : "expand"}`} />}>
                    <div className="flex items-center justify-between">
                        <Kicker icon={sub.icon} label={sub.label} />
                        <Tooltip>
                            <TooltipTrigger
                                render={
                                    <span className="cursor-default rounded-md border border-border/50 px-1.5 py-0.5 font-mono font-semibold text-[9.5px] text-muted-foreground/80 uppercase tabular-nums tracking-wider" style={{ background: `color-mix(in oklch, ${sub.color} 8%, transparent)` }}>
                                        ×{sub.weight.toFixed(1)}
                                    </span>
                                }
                            />
                            <TooltipContent sideOffset={5}>
                                <p>
                                    Weight ×{sub.weight.toFixed(1)} · {wShare.toFixed(0)}% of composite
                                </p>
                            </TooltipContent>
                        </Tooltip>
                    </div>

                    <div className="flex flex-1 flex-col justify-between gap-4">
                        <div className="flex flex-col gap-1">
                            <div className="flex items-baseline gap-1.5">
                                <span
                                    className="font-bold tabular-nums leading-none"
                                    style={{
                                        fontSize: "clamp(2rem, 2.6vw + 0.75rem, 2.5rem)",
                                        letterSpacing: "-0.03em",
                                        color: sub.color,
                                    }}
                                >
                                    {pct.toFixed(1)}
                                </span>
                                <span className="font-medium font-mono text-base text-muted-foreground/50 tabular-nums">%</span>
                            </div>
                            <span className={KICKER_TEXT}>{sub.description}</span>
                        </div>

                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <span className={KICKER_TEXT}>Progress</span>
                                <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{formatPct(score, 2)}</span>
                            </div>
                            <Bar color={sub.color} pct={pct} />
                        </div>

                        <div className="flex items-center justify-between border-border/30 border-t pt-2.5">
                            <span className="font-mono text-[10px] text-muted-foreground/70 uppercase tracking-wider transition-colors group-hover:text-foreground/85">{open ? "Hide breakdown" : "What can I improve?"}</span>
                            <ChevronDown aria-hidden className={cn("size-3.5 shrink-0 text-muted-foreground/65 transition-[transform,color] duration-200 group-hover:text-foreground/85", open && "rotate-180")} />
                        </div>
                    </div>
                </CollapsibleTrigger>
                <CollapsibleContent className="overflow-hidden">
                    <div className="border-border/30 border-t">
                        <ImprovementsPanel sub={sub} improvements={improvements} isLoading={isImprovementsLoading} />
                    </div>
                </CollapsibleContent>
            </StatCard>
        </Collapsible>
    );
}
