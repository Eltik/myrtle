"use client";

import { useEffect, useMemo, useState } from "react";
import { AnimatedGroup } from "~/components/ui/motion-primitives/animated-group";
import { Separator } from "~/components/ui/shadcn/separator";
import { getOperatorAttributeStats } from "~/lib/operator-stats";
import type { AttributeData, Operator, Range } from "~/types/api";
import { HandbookSection } from "./handbook-section";
import { ModuleDetails } from "./module-details";
import { OperatorControls } from "./operator-controls";
import { OperatorRange } from "./operator-range";
import { StatsDisplay } from "./stats-display";
import { TagsDisplay } from "./tags-display";
import { TopDescription } from "./top-description";
import { TopInfo } from "./top-info";

interface InfoTabProps {
    operator: Operator;
}

export function InfoTab({ operator }: InfoTabProps) {
    const [phaseIndex, setPhaseIndex] = useState(operator.phases.length - 1);
    const [level, setLevel] = useState(operator.phases[operator.phases.length - 1]?.MaxLevel ?? 1);
    const [favorPoint, setFavorPoint] = useState(100);
    const [potentialRank, setPotentialRank] = useState(0);
    const [currentModule, setCurrentModule] = useState("");
    const [currentModuleLevel, setCurrentModuleLevel] = useState(0);
    const [ranges, setRanges] = useState<Range[]>([]);
    const [currentRangeId, setCurrentRangeId] = useState("");
    const [showControls, setShowControls] = useState(true);

    // Memoize current range
    const currentRange = useMemo(() => {
        return ranges.find((range) => range.id === currentRangeId) ?? null;
    }, [ranges, currentRangeId]);

    // Fetch ranges on mount
    useEffect(() => {
        const fetchRanges = async () => {
            const allRanges: Range[] = [];
            for (const phase of operator.phases) {
                if (phase.RangeId && !allRanges.find((r) => r.id === phase.RangeId)) {
                    try {
                        const response = await fetch("/api/static", {
                            method: "POST",
                            headers: { "Content-Type": "application/json" },
                            body: JSON.stringify({ type: "ranges", id: phase.RangeId }),
                        });
                        const data = (await response.json()) as { data: Range };
                        if (data.data) {
                            allRanges.push({ ...data.data, id: phase.RangeId });
                        }
                    } catch (error) {
                        console.error("Failed to fetch range:", error);
                    }
                }
            }
            setRanges(allRanges);
            if (operator.phases[phaseIndex]?.RangeId) {
                setCurrentRangeId(operator.phases[phaseIndex].RangeId!);
            }
        };
        void fetchRanges();
    }, [operator, phaseIndex]);

    // Update range when phase changes
    useEffect(() => {
        const phase = operator.phases[phaseIndex];
        if (phase?.RangeId) {
            setCurrentRangeId(phase.RangeId);
        }
    }, [operator.phases, phaseIndex]);

    const attributeStats: AttributeData | null = useMemo(() => {
        return getOperatorAttributeStats(
            operator,
            {
                phaseIndex,
                favorPoint,
                potentialRank,
                moduleId: currentModule,
                moduleLevel: currentModuleLevel,
            },
            level,
        );
    }, [operator, phaseIndex, level, favorPoint, potentialRank, currentModule, currentModuleLevel]);

    return (
        <div className="w-full space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <h2 className="font-bold text-2xl md:text-3xl">Operator Info</h2>
            </div>
            <Separator />

            <AnimatedGroup className="space-y-6" preset="blur-slide">
                {/* Top Info Section */}
                <TopInfo operator={operator} />

                {/* Description */}
                <TopDescription operator={operator} />

                {/* Controls */}
                <OperatorControls
                    currentModule={currentModule}
                    currentModuleLevel={currentModuleLevel}
                    favorPoint={favorPoint}
                    level={level}
                    onFavorPointChange={setFavorPoint}
                    onLevelChange={setLevel}
                    onModuleChange={setCurrentModule}
                    onModuleLevelChange={setCurrentModuleLevel}
                    onPhaseChange={setPhaseIndex}
                    onPotentialChange={setPotentialRank}
                    onToggleControls={() => setShowControls(!showControls)}
                    operator={operator}
                    phaseIndex={phaseIndex}
                    potentialRank={potentialRank}
                    showControls={showControls}
                />

                {/* Stats */}
                <StatsDisplay
                    attributeStats={attributeStats}
                    onPhaseChange={(idx) => {
                        setPhaseIndex(idx);
                        setLevel(operator.phases[idx]?.MaxLevel ?? 1);
                    }}
                    operator={operator}
                    phaseIndex={phaseIndex}
                />

                {/* Tags */}
                <TagsDisplay tags={operator.tagList} />

                {/* Range */}
                {currentRange && (
                    <div className="space-y-2">
                        <h3 className="font-semibold text-lg">Attack Range</h3>
                        <OperatorRange range={currentRange} />
                    </div>
                )}

                {/* Module Details */}
                {operator.modules && operator.modules.length > 0 && <ModuleDetails modules={operator.modules} />}

                {/* Handbook */}
                {operator.handbook && <HandbookSection handbook={operator.handbook} />}
            </AnimatedGroup>
        </div>
    );
}
