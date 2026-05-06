import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import { cn } from "#/lib/utils";
import { Bar, CARD_PADDING, KICKER_TEXT, Kicker, StatCard } from "../../Stats/primitives";
import { formatPct, type ISubscore, toPct, weightShare } from "../helpers";

interface ISubscoreCardProps {
    sub: ISubscore;
    score: number | null | undefined;
}

export function SubscoreCard({ sub, score }: ISubscoreCardProps) {
    const pct = toPct(score);
    const wShare = weightShare(sub.weight);

    return (
        <StatCard color={sub.color}>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <div className="flex items-center justify-between">
                    <Kicker icon={sub.icon} label={sub.label} />
                    <Tooltip>
                        <TooltipTrigger
                            render={
                                <span className="cursor-default rounded-md border border-border/50 px-1.5 py-0.5 font-mono text-[9.5px] font-semibold tabular-nums uppercase tracking-wider text-muted-foreground/80" style={{ background: `color-mix(in oklch, ${sub.color} 8%, transparent)` }}>
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
                            <span className="font-mono text-base font-medium tabular-nums text-muted-foreground/50">%</span>
                        </div>
                        <span className={KICKER_TEXT}>{sub.description}</span>
                    </div>

                    <div className="space-y-2">
                        <div className="flex items-center justify-between">
                            <span className={KICKER_TEXT}>Progress</span>
                            <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">{formatPct(score, 2)}</span>
                        </div>
                        <Bar color={sub.color} pct={pct} />
                    </div>
                </div>
            </div>
        </StatCard>
    );
}
