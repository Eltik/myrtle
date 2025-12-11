"use client";

import { useState, useEffect, useMemo, useCallback } from "react";
import { motion } from "motion/react";
import type { Operator, Range } from "~/types/api";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Disclosure, DisclosureTrigger, DisclosureContent } from "~/components/ui/motion-primitives/disclosure";
import { OperatorStats } from "../components/operator-stats";
import { OperatorControls } from "../components/operator-controls";
import { OperatorRange } from "../components/operator-range";
import { OperatorTags } from "../components/operator-tags";
import { OperatorModuleDetails } from "../components/operator-module-details";
import { OperatorHandbook } from "../components/operator-handbook";
import { ChibiViewer } from "../components/chibi-viewer";
import { getOperatorAttributeStats } from "~/lib/operator-stats";
import { ChevronDown } from "lucide-react";

interface InfoTabProps {
    operator: Operator;
}

export function InfoTab({ operator }: InfoTabProps) {
    const [showControls, setShowControls] = useState(true);
    const [isModuleDetailsOpen, setIsModuleDetailsOpen] = useState(true);
    const [isHandbookOpen, setIsHandbookOpen] = useState(true);

    // Stat calculation state
    const [phaseIndex, setPhaseIndex] = useState(operator.phases.length - 1);
    const [level, setLevel] = useState(1);
    const [favorPoint, setFavorPoint] = useState(100);
    const [potentialRank, setPotentialRank] = useState(0);
    const [currentModule, setCurrentModule] = useState("");
    const [currentModuleLevel, setCurrentModuleLevel] = useState(0);

    // Range state
    const [ranges, setRanges] = useState<Range[]>([]);
    const [currentRangeId, setCurrentRangeId] = useState("");

    // Initialize level to max for the phase
    useEffect(() => {
        setLevel(operator.phases[operator.phases.length - 1]?.MaxLevel ?? 1);
    }, [operator]);

    // Fetch ranges
    useEffect(() => {
        const fetchRanges = async () => {
            const rangeIds = new Set<string>();
            for (const phase of operator.phases) {
                if (phase.RangeId) {
                    rangeIds.add(phase.RangeId);
                }
            }

            const fetchedRanges: Range[] = [];
            for (const rangeId of rangeIds) {
                try {
                    const response = await fetch("/api/static", {
                        method: "POST",
                        headers: { "Content-Type": "application/json" },
                        body: JSON.stringify({ type: "ranges", id: rangeId }),
                    });
                    if (response.ok) {
                        const data = (await response.json()) as { data: Range };
                        if (data.data) {
                            fetchedRanges.push(data.data);
                        }
                    }
                } catch (error) {
                    console.error("Failed to fetch range:", error);
                }
            }
            setRanges(fetchedRanges);

            // Set current range to the current phase's range
            const currentPhase = operator.phases[phaseIndex];
            if (currentPhase?.RangeId) {
                setCurrentRangeId(currentPhase.RangeId);
            }
        };

        void fetchRanges();
    }, [operator, phaseIndex]);

    // Update current range when phase changes
    useEffect(() => {
        const currentPhase = operator.phases[phaseIndex];
        if (currentPhase?.RangeId) {
            setCurrentRangeId(currentPhase.RangeId);
        }

        // Reset module if not at E2
        if (phaseIndex !== 2) {
            setCurrentModule("");
            setCurrentModuleLevel(0);
        }
    }, [phaseIndex, operator.phases]);

    // Calculate attribute stats
    const attributeStats = useMemo(() => {
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

    // Get current range
    const currentRange = useMemo(() => {
        return ranges.find((range) => range.id === currentRangeId) ?? null;
    }, [ranges, currentRangeId]);

    const handlePhaseChange = useCallback(
        (newPhaseIndex: number) => {
            setPhaseIndex(newPhaseIndex);
            setLevel(operator.phases[newPhaseIndex]?.MaxLevel ?? 1);
        },
        [operator.phases],
    );

    return (
        <div className="space-y-6">
            {/* Operator Info Header */}
            <InView
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                transition={{ duration: 0.4 }}
            >
                <div className="rounded-xl border bg-card/50 p-6 backdrop-blur-sm">
                    <h2 className="mb-4 text-xl font-semibold">Operator Info</h2>

                    {/* Description */}
                    {operator.description && (
                        <div className="mb-6">
                            <p className="text-sm leading-relaxed text-muted-foreground">{operator.description}</p>
                        </div>
                    )}

                    {/* Trait */}
                    {operator.trait?.Candidates?.[0]?.OverrideDescription && (
                        <div className="mb-6 rounded-lg bg-muted/50 p-4">
                            <h3 className="mb-2 text-sm font-medium text-muted-foreground">Trait</h3>
                            <p className="text-sm">{operator.trait.Candidates[0].OverrideDescription}</p>
                        </div>
                    )}

                    {/* Controls */}
                    <OperatorControls
                        operator={operator}
                        showControls={showControls}
                        setShowControls={setShowControls}
                        phaseIndex={phaseIndex}
                        level={level}
                        setLevel={setLevel}
                        favorPoint={favorPoint}
                        setFavorPoint={setFavorPoint}
                        potentialRank={potentialRank}
                        setPotentialRank={setPotentialRank}
                        currentModule={currentModule}
                        setCurrentModule={setCurrentModule}
                        currentModuleLevel={currentModuleLevel}
                        setCurrentModuleLevel={setCurrentModuleLevel}
                    />

                    {/* Stats */}
                    <OperatorStats operator={operator} attributeStats={attributeStats} phaseIndex={phaseIndex} onPhaseChange={handlePhaseChange} />
                </div>
            </InView>

            {/* Tags */}
            <InView
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                transition={{ duration: 0.4, delay: 0.1 }}
            >
                <OperatorTags tags={operator.tagList} />
            </InView>

            {/* Range */}
            <InView
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                transition={{ duration: 0.4, delay: 0.15 }}
            >
                <div className="rounded-xl border bg-card/50 p-6 backdrop-blur-sm">
                    <h3 className="mb-4 text-lg font-semibold">Attack Range</h3>
                    {currentRange ? <OperatorRange range={currentRange} /> : <p className="text-sm text-muted-foreground">No range data available.</p>}
                </div>
            </InView>

            {/* Chibi Viewer Placeholder */}
            <InView
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                transition={{ duration: 0.4, delay: 0.2 }}
            >
                <ChibiViewer operator={operator} />
            </InView>

            {/* Module Details */}
            {operator.modules && operator.modules.length > 0 && (
                <InView
                    variants={{
                        hidden: { opacity: 0, y: 20 },
                        visible: { opacity: 1, y: 0 },
                    }}
                    transition={{ duration: 0.4, delay: 0.25 }}
                >
                    <Disclosure open={isModuleDetailsOpen} onOpenChange={setIsModuleDetailsOpen} className="rounded-xl border bg-card/50 backdrop-blur-sm">
                        <DisclosureTrigger>
                            <div className="flex cursor-pointer items-center justify-between p-6 transition-colors hover:bg-muted/50">
                                <h3 className="text-lg font-semibold">Module Details</h3>
                                <motion.div animate={{ rotate: isModuleDetailsOpen ? 180 : 0 }} transition={{ duration: 0.2 }}>
                                    <ChevronDown className="h-5 w-5 text-muted-foreground" />
                                </motion.div>
                            </div>
                        </DisclosureTrigger>
                        <DisclosureContent>
                            <div className="border-t px-6 pb-6">
                                <OperatorModuleDetails modules={operator.modules} currentModule={currentModule} />
                            </div>
                        </DisclosureContent>
                    </Disclosure>
                </InView>
            )}

            {/* Handbook */}
            <InView
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                transition={{ duration: 0.4, delay: 0.3 }}
            >
                <Disclosure open={isHandbookOpen} onOpenChange={setIsHandbookOpen} className="rounded-xl border bg-card/50 backdrop-blur-sm">
                    <DisclosureTrigger>
                        <div className="flex cursor-pointer items-center justify-between p-6 transition-colors hover:bg-muted/50">
                            <h3 className="text-lg font-semibold">Operator Files</h3>
                            <motion.div animate={{ rotate: isHandbookOpen ? 180 : 0 }} transition={{ duration: 0.2 }}>
                                <ChevronDown className="h-5 w-5 text-muted-foreground" />
                            </motion.div>
                        </div>
                    </DisclosureTrigger>
                    <DisclosureContent>
                        <div className="border-t px-6 pb-6">
                            <OperatorHandbook operator={operator} />
                        </div>
                    </DisclosureContent>
                </Disclosure>
            </InView>
        </div>
    );
}
