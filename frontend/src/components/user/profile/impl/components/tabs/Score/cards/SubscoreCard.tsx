import { ChevronDown } from "lucide-react";
import { useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IImprovementsResponse, IUserScore } from "#/lib/api/user";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { Bar, CARD_PADDING, KICKER_TEXT, Kicker, StatCard } from "../../Stats/primitives";
import { formatPct, type ISubscore, toPct, weightShare } from "../helpers";
import type { messages as helperMessages } from "../helpers.messages";
import { ImprovementsPanel } from "../improvements/ImprovementsPanel";
import type { messages } from "./SubscoreCard.messages";

interface ISubscoreCardProps {
    sub: ISubscore;
    score: number | null | undefined;
    improvements: IImprovementsResponse | null | undefined;
    isImprovementsLoading: boolean;
    /** The full stored score row, for panels that read more than the headline number. */
    scoreRow: IUserScore;
}

export function SubscoreCard({ sub, score, improvements, isImprovementsLoading, scoreRow }: ISubscoreCardProps) {
    /** Section names and blurbs are declared once, in `helpers.messages.ts`. */
    const t: TypedT<typeof messages & typeof helperMessages> = useT("user");
    const pct = toPct(score);
    const wShare = weightShare(sub.weight);
    const contribution = (pct * wShare) / 100;
    const [open, setOpen] = useState(false);

    return (
        <Collapsible open={open} onOpenChange={setOpen}>
            <StatCard color={sub.color} className={open ? "ring-1 ring-border/40" : undefined}>
                <CollapsibleTrigger render={<button type="button" className={cn("group flex w-full flex-col gap-5 text-left", CARD_PADDING)} aria-expanded={open} aria-label={t(open ? "score.sub.details.collapse" : "score.sub.details.expand", { label: t(sub.labelKey) })} />}>
                    <div className="flex items-center justify-between">
                        <Kicker icon={sub.icon} label={t(sub.labelKey)} />
                        <Tooltip>
                            <TooltipTrigger
                                render={
                                    <span className="cursor-default rounded-md border border-border/50 px-1.5 py-0.5 font-mono font-semibold text-[9.5px] text-muted-foreground/80 uppercase tabular-nums tracking-wider" style={{ background: `color-mix(in oklch, ${sub.color} 8%, transparent)` }}>
                                        {t("score.sub.shareBadge", { share: wShare.toFixed(0) })}
                                    </span>
                                }
                            />
                            <TooltipContent sideOffset={5}>
                                <p>{t("score.sub.shareTooltip", { share: wShare.toFixed(0), pct: pct.toFixed(1), contribution: contribution.toFixed(1) })}</p>
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
                            <span className={KICKER_TEXT}>{t(sub.descriptionKey)}</span>
                        </div>

                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <span className={KICKER_TEXT}>{t("score.sub.progress")}</span>
                                <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{formatPct(score, 2)}</span>
                            </div>
                            <Bar color={sub.color} pct={pct} />
                        </div>

                        <div className="flex items-center justify-between border-border/30 border-t pt-2.5">
                            <span className="font-mono text-[10px] text-muted-foreground/70 uppercase tracking-wider transition-colors group-hover:text-foreground/85">{open ? t("score.sub.hideBreakdown") : t("score.sub.whatToImprove")}</span>
                            <ChevronDown aria-hidden className={cn("size-3.5 shrink-0 text-muted-foreground/65 transition-[transform,color] duration-200 group-hover:text-foreground/85", open && "rotate-180")} />
                        </div>
                    </div>
                </CollapsibleTrigger>
                <CollapsibleContent className="overflow-hidden">
                    <div className="border-border/30 border-t">
                        <ImprovementsPanel sub={sub} improvements={improvements} isLoading={isImprovementsLoading} score={scoreRow} />
                    </div>
                </CollapsibleContent>
            </StatCard>
        </Collapsible>
    );
}
