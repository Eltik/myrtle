import { Sparkles } from "lucide-react";
import { cn } from "#/lib/utils";
import { PALETTE } from "../palette";
import { Bar, CARD_PADDING, KICKER_TEXT, Kicker, StatCard, Tile } from "../primitives";

interface IMasteryCardProps {
    masteries: {
        m3Count: number;
        m6Count: number;
        m9Count: number;
        totalMasteryLevels: number;
        maxPossibleMasteryLevels: number;
    };
}

export function MasteryCard({ masteries }: IMasteryCardProps) {
    const { totalMasteryLevels, maxPossibleMasteryLevels } = masteries;
    const masteryPct = maxPossibleMasteryLevels > 0 ? (totalMasteryLevels / maxPossibleMasteryLevels) * 100 : 0;

    return (
        <StatCard color={PALETTE.mastery.accent}>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <Kicker icon={Sparkles} label="Skill Mastery" />
                <div className="flex flex-1 flex-col justify-center gap-5">
                    <div className="grid grid-cols-3 gap-2">
                        <Tile color={PALETTE.mastery.m3} sub="M3" tooltip="Operators with at least one Mastery 3 skill" value={masteries.m3Count} />
                        <Tile color={PALETTE.mastery.m6} sub="M6" tooltip="Operators with 2 skills at Mastery 3" value={masteries.m6Count} />
                        <Tile color={PALETTE.mastery.m9} sub="M9" tooltip="Operators with all 3 skills at Mastery 3" value={masteries.m9Count} />
                    </div>
                    <div className="space-y-2">
                        <div className="flex items-center justify-between">
                            <span className={KICKER_TEXT}>Total Mastery Levels</span>
                            <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
                                <span className="font-semibold text-foreground">{totalMasteryLevels}</span>
                                <span className="text-muted-foreground/50"> / {maxPossibleMasteryLevels}</span>
                            </span>
                        </div>
                        <Bar color={PALETTE.mastery.accent} pct={masteryPct} />
                        <p className="text-center font-mono text-[10px] text-muted-foreground/50">{masteryPct.toFixed(1)}% of max possible (E2 only)</p>
                    </div>
                </div>
            </div>
        </StatCard>
    );
}
