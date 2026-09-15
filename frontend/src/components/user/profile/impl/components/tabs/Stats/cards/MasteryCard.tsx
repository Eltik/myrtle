import { Sparkles } from "lucide-react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IMasteryGapDetails } from "../helpers";
import { PALETTE } from "../palette";
import { Bar, CARD_PADDING, GapList, type IGapItem, KICKER_TEXT, Kicker, StatCard, Tile } from "../primitives";
import type { messages } from "./MasteryCard.messages";

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
    const t: TypedT<typeof messages> = useT("user");
    const { totalMasteryLevels, maxPossibleMasteryLevels, e2Count, m3Count, details } = masteries;
    const masteryPct = maxPossibleMasteryLevels > 0 ? (totalMasteryLevels / maxPossibleMasteryLevels) * 100 : 0;

    const gaps: IGapItem[] = [
        {
            key: "levels",
            label: t("profile.stats.mastery.gap.levels"),
            value: Math.max(0, maxPossibleMasteryLevels - totalMasteryLevels),
            color: PALETTE.mastery.accent,
            tooltip: t("profile.stats.mastery.gap.levels.tooltip"),
        },
        {
            key: "pendingM3",
            label: t("profile.stats.mastery.gap.m3"),
            value: Math.max(0, e2Count - m3Count),
            color: PALETTE.mastery.m3,
            tooltip: t("profile.stats.mastery.gap.m3.tooltip"),
            details: details.pendingM3,
        },
        {
            key: "pendingM6",
            label: t("profile.stats.mastery.gap.m6"),
            value: details.pendingM6.length,
            color: PALETTE.mastery.m6,
            tooltip: t("profile.stats.mastery.gap.m6.tooltip"),
            details: details.pendingM6,
        },
        {
            key: "pendingM9",
            label: t("profile.stats.mastery.gap.m9"),
            value: details.pendingM9.length,
            color: PALETTE.mastery.m9,
            tooltip: t("profile.stats.mastery.gap.m9.tooltip"),
            details: details.pendingM9,
        },
    ];

    return (
        <StatCard color={PALETTE.mastery.accent}>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <Kicker icon={Sparkles} label={t("profile.stats.mastery.title")} />
                <div className="flex flex-1 flex-col justify-center gap-5">
                    {/* M3/M6/M9 are rank symbols, like the grade letters in `helpers.messages.ts` - not words to translate. */}
                    <div className="grid grid-cols-3 gap-2">
                        <Tile color={PALETTE.mastery.m3} sub="M3" tooltip={t("profile.stats.mastery.m3.tooltip")} value={masteries.m3Count} />
                        <Tile color={PALETTE.mastery.m6} sub="M6" tooltip={t("profile.stats.mastery.m6.tooltip")} value={masteries.m6Count} />
                        <Tile color={PALETTE.mastery.m9} sub="M9" tooltip={t("profile.stats.mastery.m9.tooltip")} value={masteries.m9Count} />
                    </div>
                    <div className="space-y-2">
                        <div className="flex items-center justify-between">
                            <span className={KICKER_TEXT}>{t("profile.stats.mastery.totalLevels")}</span>
                            <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                <span className="font-semibold text-foreground">{totalMasteryLevels}</span>
                                <span className="text-muted-foreground/50"> / {maxPossibleMasteryLevels}</span>
                            </span>
                        </div>
                        <Bar color={PALETTE.mastery.accent} pct={masteryPct} />
                        <p className="text-center font-mono text-[10px] text-muted-foreground/50">{t("profile.stats.mastery.ofMax", { pct: masteryPct.toFixed(1) })}</p>
                    </div>
                    <div className="border-border/60 border-t pt-3">
                        <GapList items={gaps} />
                    </div>
                </div>
            </div>
        </StatCard>
    );
}
