import { Sparkles } from "lucide-react";
import { cn } from "#/lib/utils";
import type { IMasteryGapDetails } from "../helpers";
import { PALETTE } from "../palette";
import { Bar, CARD_PADDING, GapList, type IGapItem, KICKER_TEXT, Kicker, StatCard, Tile } from "../primitives";

interface IMasteryCardProps {
    masteries: {
        m3Count: number;
        m6Count: number;
        m9Count: number;
        totalMasteryLevels: number;
        maxPossibleMasteryLevels: number;
        e2Count: number;
        details: IMasteryGapDetails;
    };
}

export function MasteryCard({ masteries }: IMasteryCardProps) {
    const { totalMasteryLevels, maxPossibleMasteryLevels, e2Count, m3Count, details } = masteries;
    const masteryPct = maxPossibleMasteryLevels > 0 ? (totalMasteryLevels / maxPossibleMasteryLevels) * 100 : 0;

    const gaps: IGapItem[] = [
        {
            key: "levels",
            label: "levels",
            value: Math.max(0, maxPossibleMasteryLevels - totalMasteryLevels),
            color: PALETTE.mastery.accent,
            tooltip: "Skill levels remaining to reach Mastery 3 across all E2 operators",
        },
        {
            key: "pendingM3",
            label: "pending M3",
            value: Math.max(0, e2Count - m3Count),
            color: PALETTE.mastery.m3,
            tooltip: "Click to view E2 operators without any skill at Mastery 3 yet",
            details: details.pendingM3,
        },
        {
            key: "pendingM6",
            label: "pending M6",
            value: details.pendingM6.length,
            color: PALETTE.mastery.m6,
            tooltip: "Click to view operators with 1 skill at M3 still needing a second",
            details: details.pendingM6,
        },
        {
            key: "pendingM9",
            label: "pending M9",
            value: details.pendingM9.length,
            color: PALETTE.mastery.m9,
            tooltip: "Click to view operators with 2 skills at M3 still needing a third",
            details: details.pendingM9,
        },
    ];

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
                            <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                <span className="font-semibold text-foreground">{totalMasteryLevels}</span>
                                <span className="text-muted-foreground/50"> / {maxPossibleMasteryLevels}</span>
                            </span>
                        </div>
                        <Bar color={PALETTE.mastery.accent} pct={masteryPct} />
                        <p className="text-center font-mono text-[10px] text-muted-foreground/50">{masteryPct.toFixed(1)}% of max possible (E2 only)</p>
                    </div>
                    <div className="border-border/60 border-t pt-3">
                        <GapList items={gaps} />
                    </div>
                </div>
            </div>
        </StatCard>
    );
}
