"use client";

import { useEffect, useMemo, useState } from "react";
import type { Operator, Range } from "~/types/api";
import { AnimatedGroup } from "~/components/ui/motion-primitives/animated-group";
import { Separator } from "~/components/ui/shadcn/separator";
import { TopInfo } from "./top-info";
import { TopDescription } from "./top-description";
import { OperatorControls } from "./operator-controls";
import { StatsDisplay } from "./stats-display";
import { OperatorRange } from "./operator-range";
import { TagsDisplay } from "./tags-display";
import { ModuleDetails } from "./module-details";
import { HandbookSection } from "./handbook-section";

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

    // Calculate attribute stats
    const attributeStats = useMemo(() => {
        const phase = operator.phases[phaseIndex];
        if (!phase?.AttributesKeyFrames?.length) return null;

        const maxLevel = phase.MaxLevel;
        const start = phase.AttributesKeyFrames[0]?.Data;
        const end = phase.AttributesKeyFrames[phase.AttributesKeyFrames.length - 1]?.Data;

        if (!start || !end) return null;

        const lerp = (a: number, b: number) => Math.round(a + ((level - 1) * (b - a)) / (maxLevel - 1));

        return {
            maxHp: lerp(start.MaxHp, end.MaxHp),
            atk: lerp(start.Atk, end.Atk),
            def: lerp(start.Def, end.Def),
            magicResistance: lerp(start.MagicResistance, end.MagicResistance),
            attackSpeed: start.AttackSpeed,
            blockCnt: start.BlockCnt,
            respawnTime: start.RespawnTime,
            cost: start.Cost,
        };
    }, [operator.phases, phaseIndex, level]);

    return (
        <div className="w-full space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <h2 className="text-2xl font-bold md:text-3xl">Operator Info</h2>
            </div>
            <Separator />

            <AnimatedGroup preset="blur-slide" className="space-y-6">
                {/* Top Info Section */}
                <TopInfo operator={operator} />

                {/* Description */}
                <TopDescription operator={operator} />

                {/* Controls */}
                <OperatorControls operator={operator} phaseIndex={phaseIndex} level={level} favorPoint={favorPoint} showControls={showControls} onPhaseChange={setPhaseIndex} onLevelChange={setLevel} onFavorPointChange={setFavorPoint} onToggleControls={() => setShowControls(!showControls)} />

                {/* Stats */}
                <StatsDisplay
                    operator={operator}
                    attributeStats={attributeStats}
                    phaseIndex={phaseIndex}
                    onPhaseChange={(idx) => {
                        setPhaseIndex(idx);
                        setLevel(operator.phases[idx]?.MaxLevel ?? 1);
                    }}
                />

                {/* Tags */}
                <TagsDisplay tags={operator.tagList} />

                {/* Range */}
                {currentRange && (
                    <div className="space-y-2">
                        <h3 className="text-lg font-semibold">Attack Range</h3>
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
