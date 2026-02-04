"use client";

import { motion } from "motion/react";
import { memo, useMemo } from "react";
import { Badge } from "~/components/ui/shadcn/badge";
import { descriptionToHtml } from "~/lib/description-parser";
import { computeSkillDiff, formatBlackboardValue, formatSkillLevel } from "~/lib/skill-helpers";
import { cn } from "~/lib/utils";
import type { SkillLevel as SkillLevelType } from "~/types/api/impl/skill";

interface SkillComparisonRowProps {
    levelIndex: number;
    levelData: SkillLevelType;
    prevLevelData: SkillLevelType | null;
    isFirst?: boolean;
    isLast?: boolean;
    showDifferencesOnly?: boolean;
}

export const SkillComparisonRow = memo(function SkillComparisonRow({ levelIndex, levelData, prevLevelData, isFirst, isLast, showDifferencesOnly }: SkillComparisonRowProps) {
    const descriptionHtml = useMemo(() => descriptionToHtml(levelData.description ?? "", levelData.blackboard ?? []), [levelData.description, levelData.blackboard]);

    const diff = useMemo(() => computeSkillDiff(prevLevelData, levelData), [prevLevelData, levelData]);

    const hasAnyChanges = diff.spCostChanged || diff.initSpChanged || diff.durationChanged || diff.blackboardChanges.size > 0;

    const diffSummary = useMemo(() => {
        if (!showDifferencesOnly || isFirst) return null;

        const changes: { label: string; value: string; type: "stat" | "param" }[] = [];

        if (diff.spCostChanged && prevLevelData) {
            changes.push({
                label: "SP Cost",
                value: `${prevLevelData.spData?.spCost} → ${levelData.spData?.spCost}`,
                type: "stat",
            });
        }
        if (diff.initSpChanged && prevLevelData) {
            changes.push({
                label: "Initial SP",
                value: `${prevLevelData.spData?.initSp} → ${levelData.spData?.initSp}`,
                type: "stat",
            });
        }
        if (diff.durationChanged && prevLevelData) {
            const prevDur = prevLevelData.duration && prevLevelData.duration > 0 ? `${prevLevelData.duration}s` : "-";
            const currDur = levelData.duration && levelData.duration > 0 ? `${levelData.duration}s` : "-";
            changes.push({
                label: "Duration",
                value: `${prevDur} → ${currDur}`,
                type: "stat",
            });
        }

        for (const [key, { prev, curr }] of diff.blackboardChanges) {
            const formattedPrev = formatBlackboardValue(key, prev);
            const formattedCurr = formatBlackboardValue(key, curr);
            const displayKey = key.charAt(0).toUpperCase() + key.slice(1).replace(/_/g, " ");
            changes.push({
                label: displayKey,
                value: `${formattedPrev} → ${formattedCurr}`,
                type: "param",
            });
        }

        return changes;
    }, [showDifferencesOnly, isFirst, diff, prevLevelData, levelData]);

    // In differences-only mode, show nothing if there are no changes (except first row)
    if (showDifferencesOnly && !isFirst && !hasAnyChanges) {
        return null;
    }

    return (
        <motion.div
            animate={{ opacity: 1 }}
            className={cn("flex flex-col gap-4 border-border bg-card/30 p-5 md:flex-row md:items-start md:gap-6 md:p-6", isFirst && "rounded-t-lg border", !isFirst && !isLast && "border-x border-b", isLast && "rounded-b-lg border-x border-b")}
            exit={{ opacity: 0 }}
            initial={{ opacity: 0 }}
            layout
            transition={{ duration: 0.15, delay: isFirst ? 0 : 0.03 }}
        >
            <div className="flex shrink-0 items-center gap-2 md:w-20 md:flex-col md:items-start md:gap-1.5">
                <Badge className={cn("rounded-sm font-mono text-sm shadow-sm", levelIndex >= 7 ? "border-border bg-muted text-foreground" : "border-border bg-muted/50 text-muted-foreground")} variant="secondary">
                    {formatSkillLevel(levelIndex)}
                </Badge>
            </div>

            <div className="min-w-0 flex-1">
                {showDifferencesOnly && !isFirst && diffSummary && diffSummary.length > 0 ? (
                    <div className="space-y-2.5">
                        {diffSummary.map((change, idx) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: Changes are dynamically generated
                            <div className="flex items-baseline gap-3 rounded-sm border border-border bg-muted/30 px-4 py-2.5" key={idx}>
                                <span className="shrink-0 font-medium text-muted-foreground text-xs uppercase tracking-wide">{change.label}</span>
                                <span className="font-medium font-mono text-foreground text-sm">{change.value}</span>
                            </div>
                        ))}
                    </div>
                ) : (
                    <p
                        className="text-foreground text-sm leading-relaxed"
                        // biome-ignore lint/security/noDangerouslySetInnerHtml: Intentional HTML rendering for skill descriptions
                        dangerouslySetInnerHTML={{ __html: descriptionHtml }}
                    />
                )}
            </div>

            <div className="flex shrink-0 items-center gap-2.5 md:gap-3">
                <div className={cn("flex flex-col items-center justify-center rounded-sm border px-4.5 py-1.5 transition-all duration-200", diff.spCostChanged ? "border-border bg-muted shadow-sm ring-1 ring-border" : "border-border/60 bg-muted/40")}>
                    <div className="font-medium text-[10px] text-muted-foreground uppercase tracking-wide">SP</div>
                    <div className={cn("mt-0.5 font-mono font-semibold text-base", "text-foreground")}>{levelData.spData?.spCost ?? "-"}</div>
                </div>
                <div className={cn("flex flex-col items-center justify-center rounded-sm border px-4.5 py-1.5 transition-all duration-200", diff.initSpChanged ? "border-border bg-muted shadow-sm ring-1 ring-border" : "border-border/60 bg-muted/40")}>
                    <div className="font-medium text-[10px] text-muted-foreground uppercase tracking-wide">Init</div>
                    <div className={cn("mt-0.5 font-mono font-semibold text-base", "text-foreground")}>{levelData.spData?.initSp ?? "-"}</div>
                </div>
                <div className={cn("flex flex-col items-center justify-center rounded-sm border px-4.5 py-1.5 transition-all duration-200", diff.durationChanged ? "border-border bg-muted shadow-sm ring-1 ring-border" : "border-border/60 bg-muted/40")}>
                    <div className="font-medium text-[10px] text-muted-foreground uppercase tracking-wide">Dur</div>
                    <div className={cn("mt-0.5 font-mono font-semibold text-base", "text-foreground")}>{levelData.duration && levelData.duration > 0 ? `${levelData.duration}s` : "-"}</div>
                </div>
            </div>
        </motion.div>
    );
});
