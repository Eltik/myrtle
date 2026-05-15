import { BarChart3 } from "lucide-react";
import { cn } from "#/lib/utils";
import { PALETTE } from "../palette";
import { Bar, CARD_PADDING, KICKER_TEXT, Kicker, StatCard } from "../primitives";

interface IElitePromotionCardProps {
    eliteBreakdown: { e0: number; e1: number; e2: number; total: number };
}

const ELITE_ROWS = [
    { key: "e2", label: "Elite 2" },
    { key: "e1", label: "Elite 1" },
    { key: "e0", label: "Elite 0" },
] as const;

export function ElitePromotionCard({ eliteBreakdown }: IElitePromotionCardProps) {
    return (
        <StatCard color={PALETTE.elite}>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <Kicker icon={BarChart3} label="Elite Promotion" />
                <div className="flex flex-1 flex-col justify-center gap-4">
                    {ELITE_ROWS.map(({ key, label }, i) => {
                        const count = eliteBreakdown[key];
                        const pct = eliteBreakdown.total > 0 ? (count / eliteBreakdown.total) * 100 : 0;
                        const isPrimary = i === 0;
                        return (
                            <div className="space-y-1.5" key={key}>
                                <div className="flex items-center justify-between">
                                    <span className={KICKER_TEXT}>{label}</span>
                                    <div className="flex items-baseline gap-1.5">
                                        <span className="font-bold text-sm tabular-nums" style={isPrimary ? { color: PALETTE.elite } : undefined}>
                                            {count}
                                        </span>
                                        <span className="font-mono text-[10px] text-muted-foreground/50 tabular-nums">{pct.toFixed(0)}%</span>
                                    </div>
                                </div>
                                <Bar color={PALETTE.elite} dim={!isPrimary} pct={pct} />
                            </div>
                        );
                    })}
                </div>
            </div>
        </StatCard>
    );
}
