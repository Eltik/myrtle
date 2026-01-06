"use client";

import { BarChart3, Plus, RotateCcw } from "lucide-react";
import { useCallback, useState } from "react";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Button } from "~/components/ui/shadcn/button";
import { Card } from "~/components/ui/shadcn/card";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "~/components/ui/shadcn/dialog";
import { Tabs, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import type { DpsOperatorListEntry } from "~/types/api/impl/dps-calculator";
import { DpsChart } from "./impl/dps-chart";
import { OperatorConfigurator } from "./impl/operator-configurator";
import { OperatorSelector } from "./impl/operator-selector";
import type { OperatorConfiguration } from "./impl/types";

interface DpsCalculatorProps {
    operators: DpsOperatorListEntry[];
}

// Color palette for chart lines
const CHART_COLORS = [
    "#3b82f6", // blue
    "#ef4444", // red
    "#10b981", // green
    "#f59e0b", // amber
    "#8b5cf6", // purple
    "#ec4899", // pink
    "#06b6d4", // cyan
    "#f97316", // orange
];

// Helper to get default max level based on rarity and promotion
function getDefaultMaxLevel(rarity: number, promotion: number, phaseLevels?: number[]): number {
    if (phaseLevels?.[promotion]) {
        return phaseLevels[promotion];
    }
    // Fallback based on rarity (standard max levels)
    const levels: Record<number, number[]> = {
        6: [50, 80, 90],
        5: [50, 70, 80],
        4: [45, 60, 70],
        3: [40, 55],
        2: [30],
        1: [30],
    };
    return levels[rarity]?.[promotion] ?? 1;
}

export function DpsCalculator({ operators }: DpsCalculatorProps) {
    const [selectedOperators, setSelectedOperators] = useState<OperatorConfiguration[]>([]);
    const [chartMode, setChartMode] = useState<"defense" | "resistance">("defense");
    const [isDialogOpen, setIsDialogOpen] = useState(false);

    const handleSelectOperator = useCallback(
        (operator: DpsOperatorListEntry) => {
            const colorIndex = selectedOperators.length % CHART_COLORS.length;

            // Use operator's default values from backend metadata
            const defaultSkill = operator.defaultSkillIndex || (operator.availableSkills[0] ?? 1);
            const defaultModule = operator.defaultModuleIndex || 0;
            const defaultPotential = operator.defaultPotential || 1;
            const defaultPromotion = operator.maxPromotion;
            const defaultLevel = getDefaultMaxLevel(operator.rarity, defaultPromotion, operator.phaseLevels);

            const newOperator: OperatorConfiguration = {
                id: `${operator.id}-${Date.now()}`,
                operatorId: operator.id,
                operatorName: operator.name,
                rarity: operator.rarity,
                color: CHART_COLORS[colorIndex] ?? "#3b82f6",
                params: {
                    potential: defaultPotential,
                    trust: 100,
                    promotion: defaultPromotion,
                    level: defaultLevel,
                    skillIndex: defaultSkill,
                    masteryLevel: 3,
                    moduleIndex: defaultModule,
                    moduleLevel: 3,
                },
                availableSkills: operator.availableSkills,
                availableModules: operator.availableModules,
                maxPromotion: operator.maxPromotion,
                // Pass enriched data if available
                skillData: operator.skillData,
                moduleData: operator.moduleData,
                phaseLevels: operator.phaseLevels,
                potentialRanks: operator.potentialRanks,
            };

            setSelectedOperators((prev) => [...prev, newOperator]);
            setIsDialogOpen(false);
        },
        [selectedOperators.length],
    );

    const handleUpdateOperator = useCallback((id: string, updates: Partial<OperatorConfiguration>) => {
        setSelectedOperators((prev) => prev.map((op) => (op.id === id ? { ...op, ...updates } : op)));
    }, []);

    const handleRemoveOperator = useCallback((id: string) => {
        setSelectedOperators((prev) => prev.filter((op) => op.id !== id));
    }, []);

    const handleClearAll = useCallback(() => {
        setSelectedOperators([]);
    }, []);

    return (
        <div className="space-y-8">
            {/* Header */}
            <InView
                transition={{ duration: 0.5 }}
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
            >
                <div className="space-y-2">
                    <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                            <BarChart3 className="h-5 w-5 text-primary" />
                        </div>
                        <div>
                            <h1 className="font-bold text-2xl text-foreground tracking-tight sm:text-3xl">DPS Calculator</h1>
                        </div>
                    </div>
                    <p className="max-w-2xl text-muted-foreground">Calculate and compare operator damage output against varying enemy defense and resistance values. Add multiple operators to create comparison charts.</p>
                </div>
            </InView>

            {/* Chart Section */}
            <InView
                transition={{ duration: 0.5, delay: 0.1 }}
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
            >
                <Card className="border-border bg-card/30 p-4 backdrop-blur-sm sm:p-6">
                    <div className="space-y-4">
                        <div className="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
                            <div className="flex items-center gap-2">
                                <h2 className="font-semibold text-foreground">DPS Graph</h2>
                                {selectedOperators.length > 0 && (
                                    <span className="rounded-full bg-muted px-2 py-0.5 font-medium text-muted-foreground text-xs">
                                        {selectedOperators.length} operator{selectedOperators.length !== 1 ? "s" : ""}
                                    </span>
                                )}
                            </div>

                            <Tabs onValueChange={(value) => setChartMode(value as "defense" | "resistance")} value={chartMode}>
                                <TabsList>
                                    <TabsTrigger value="defense">Defense</TabsTrigger>
                                    <TabsTrigger value="resistance">Resistance</TabsTrigger>
                                </TabsList>
                            </Tabs>
                        </div>

                        {selectedOperators.length === 0 ? (
                            <div className="flex h-[400px] flex-col items-center justify-center gap-4 text-center">
                                <p className="text-muted-foreground">Add operators below to generate DPS comparison charts</p>
                            </div>
                        ) : (
                            <DpsChart mode={chartMode} operators={selectedOperators} />
                        )}
                    </div>
                </Card>
            </InView>

            {/* Operator Configuration Section */}
            <InView
                transition={{ duration: 0.5, delay: 0.2 }}
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
            >
                <Card className="border-border bg-card/30 p-4 backdrop-blur-sm sm:p-6">
                    <div className="space-y-4">
                        <div className="flex items-center justify-between">
                            <h2 className="font-semibold text-foreground">Operators</h2>
                            <div className="flex items-center gap-2">
                                {selectedOperators.length > 0 && (
                                    <Button className="gap-2" onClick={handleClearAll} size="sm" variant="ghost">
                                        <RotateCcw className="h-3.5 w-3.5" />
                                        Clear All
                                    </Button>
                                )}
                                <Dialog onOpenChange={setIsDialogOpen} open={isDialogOpen}>
                                    <DialogTrigger asChild>
                                        <Button className="gap-2" size="sm">
                                            <Plus className="h-4 w-4" />
                                            Add Operator
                                        </Button>
                                    </DialogTrigger>
                                    <DialogContent className="max-w-3xl">
                                        <DialogHeader>
                                            <DialogTitle>Select Operator</DialogTitle>
                                            <DialogDescription>Choose an operator to add to the comparison</DialogDescription>
                                        </DialogHeader>
                                        <OperatorSelector onSelectOperator={handleSelectOperator} operators={operators} />
                                    </DialogContent>
                                </Dialog>
                            </div>
                        </div>

                        {selectedOperators.length === 0 ? (
                            <div className="flex h-32 items-center justify-center text-muted-foreground text-sm">No operators selected. Click "Add Operator" to begin.</div>
                        ) : (
                            <div className="space-y-3">
                                {selectedOperators.map((operator) => (
                                    <OperatorConfigurator key={operator.id} onRemove={handleRemoveOperator} onUpdate={handleUpdateOperator} operator={operator} />
                                ))}
                            </div>
                        )}
                    </div>
                </Card>
            </InView>

            {/* Footer note */}
            <p className="text-center text-muted-foreground text-xs">Note: DPS calculations are based on operator stats at specified configurations. Results may vary based on actual game conditions and enemy mechanics.</p>
        </div>
    );
}
