"use client";

import { useEffect, useState } from "react";
import { CartesianGrid, Legend, Line, LineChart, ResponsiveContainer, XAxis, YAxis } from "recharts";
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "~/components/ui/shadcn/chart";
import { Skeleton } from "~/components/ui/shadcn/skeleton";
import { calculateDpsRange, isDpsRangeResult } from "~/lib/dps-calculator";
import type { DpsConditionalType } from "~/types/api/impl/dps-calculator";
import type { ChartDataPoint, OperatorConfiguration } from "./types";

// Map conditional type to DpsConditionals key
const CONDITIONAL_TYPE_TO_KEY: Record<DpsConditionalType, string> = {
    trait: "traitDamage",
    talent: "talentDamage",
    talent2: "talent2Damage",
    skill: "skillDamage",
    module: "moduleDamage",
};

/**
 * Build a detailed label for an operator configuration that includes
 * skill, mastery, potential, module, active conditionals, and buffs.
 */
function buildOperatorLabel(op: OperatorConfiguration): string {
    const parts: string[] = [];

    // Operator name
    parts.push(op.operatorName);

    // Potential (P1-P6)
    const potential = op.params.potential ?? 1;
    parts.push(`P${potential}`);

    // Skill and mastery
    const skillIdx = op.params.skillIndex ?? op.availableSkills[0] ?? 1;
    const mastery = op.params.masteryLevel ?? 3;
    const masteryLabel = op.maxPromotion >= 2 ? (mastery === 0 ? " Lv7" : ` M${mastery}`) : "";
    parts.push(`S${skillIdx}${masteryLabel}`);

    // Module (if selected)
    const moduleIdx = op.params.moduleIndex ?? 0;
    if (moduleIdx > 0) {
        const moduleData = op.moduleData?.find((m) => m.index === moduleIdx);
        const moduleType = moduleData?.typeName1 ?? (moduleIdx === 1 ? "X" : moduleIdx === 2 ? "Y" : "D");
        const moduleLevel = op.params.moduleLevel ?? 3;
        parts.push(`Mod${moduleType}${moduleLevel}`);
    }

    // Active conditionals (only if allCond is true or undefined)
    const allCond = op.params.allCond ?? true;
    if (allCond && op.conditionalData && op.conditionalData.length > 0) {
        const activeConditionals: string[] = [];
        for (const cond of op.conditionalData) {
            const paramKey = CONDITIONAL_TYPE_TO_KEY[cond.conditionalType];
            const isEnabled = op.params.conditionals?.[paramKey as keyof typeof op.params.conditionals] ?? true;

            // Check if this conditional is applicable to current config
            const currentSkill = op.params.skillIndex ?? op.availableSkills[0] ?? 1;
            const currentModule = op.params.moduleIndex ?? 0;
            const currentElite = op.params.promotion ?? op.maxPromotion;
            const currentModuleLevel = op.params.moduleLevel ?? 3;

            const isApplicable = (cond.applicableSkills.length === 0 || cond.applicableSkills.includes(currentSkill)) && (cond.applicableModules.length === 0 || cond.applicableModules.includes(currentModule)) && currentElite >= cond.minElite && (currentModule === 0 || currentModuleLevel >= cond.minModuleLevel);

            if (isApplicable && isEnabled && cond.name) {
                // For inverted conditionals, the name describes what happens when disabled
                // So we only add the name when enabled and it's NOT inverted
                if (!cond.inverted) {
                    activeConditionals.push(`+${cond.name}`);
                }
            }
        }
        if (activeConditionals.length > 0) {
            parts.push(activeConditionals.join(" "));
        }
    }

    // Buffs
    const buffs: string[] = [];
    const atkBuff = op.params.buffs?.atk;
    const flatAtkBuff = op.params.buffs?.flatAtk;
    const aspdBuff = op.params.buffs?.aspd;

    if (atkBuff && atkBuff > 0) {
        buffs.push(`atk+${Math.round(atkBuff * 100)}%`);
    }
    if (flatAtkBuff && flatAtkBuff > 0) {
        buffs.push(`atk+${flatAtkBuff}`);
    }
    if (aspdBuff && aspdBuff > 0) {
        buffs.push(`aspd+${aspdBuff}`);
    }

    if (buffs.length > 0) {
        parts.push(buffs.join(" "));
    }

    return parts.join(" ");
}

interface DpsChartProps {
    operators: OperatorConfiguration[];
    mode: "defense" | "resistance";
}

export function DpsChart({ operators, mode }: DpsChartProps) {
    const [chartData, setChartData] = useState<ChartDataPoint[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const fetchDpsData = async () => {
            if (operators.length === 0) {
                setChartData([]);
                return;
            }

            setLoading(true);
            setError(null);

            try {
                // Fetch DPS data for all operators in parallel
                const results = await Promise.all(
                    operators.map(async (op) => {
                        try {
                            const result = await calculateDpsRange(op.operatorId, op.params, {
                                minDef: 0,
                                maxDef: 3000,
                                defStep: 20,
                                minRes: 0,
                                maxRes: 120,
                                resStep: 10,
                            });

                            if (!isDpsRangeResult(result.dps)) {
                                return null;
                            }

                            return {
                                operator: op,
                                data: mode === "defense" ? result.dps.byDefense : result.dps.byResistance,
                            };
                        } catch (err) {
                            console.error(`Failed to fetch DPS for ${op.operatorName}:`, err);
                            return null;
                        }
                    }),
                );

                // Filter out failed requests
                const validResults = results.filter((r) => r !== null);

                if (validResults.length === 0) {
                    setError("Failed to calculate DPS for selected operators");
                    setChartData([]);
                    return;
                }

                // Merge data points by value (defense or resistance)
                const dataMap = new Map<number, ChartDataPoint>();

                for (const result of validResults) {
                    for (const point of result.data) {
                        let dataPoint = dataMap.get(point.value);
                        if (!dataPoint) {
                            dataPoint = { value: point.value };
                            dataMap.set(point.value, dataPoint);
                        }
                        dataPoint[result.operator.id] = point.dps;
                    }
                }

                const mergedData = Array.from(dataMap.values()).sort((a, b) => a.value - b.value);
                setChartData(mergedData);
            } catch (err) {
                console.error("Error calculating DPS:", err);
                setError("An error occurred while calculating DPS");
                setChartData([]);
            } finally {
                setLoading(false);
            }
        };

        void fetchDpsData();
    }, [operators, mode]);

    if (loading) {
        return (
            <div className="space-y-3">
                <Skeleton className="h-8 w-full" />
                <Skeleton className="h-[400px] w-full" />
            </div>
        );
    }

    if (error) {
        return (
            <div className="flex h-[400px] items-center justify-center rounded-lg border border-destructive/50 bg-destructive/10 text-destructive">
                <p className="text-sm">{error}</p>
            </div>
        );
    }

    if (chartData.length === 0) {
        return <div className="flex h-[400px] items-center justify-center text-muted-foreground text-sm">No data to display</div>;
    }

    // Build chart config from operators with detailed labels
    const chartConfig = operators.reduce(
        (config, op) => {
            config[op.id] = {
                label: buildOperatorLabel(op),
                color: op.color,
            };
            return config;
        },
        {} as Record<string, { label: string; color: string }>,
    );

    return (
        <ChartContainer className="h-[400px] w-full" config={chartConfig}>
            <ResponsiveContainer height="100%" width="100%">
                <LineChart data={chartData} margin={{ top: 20, right: 30, left: 20, bottom: 20 }}>
                    <CartesianGrid className="stroke-border/50" strokeDasharray="3 3" />
                    <XAxis
                        className="text-muted-foreground"
                        dataKey="value"
                        label={{
                            value: mode === "defense" ? "Defense" : "Resistance",
                            position: "insideBottom",
                            offset: -10,
                        }}
                    />
                    <YAxis
                        className="text-muted-foreground"
                        label={{
                            value: "DPS",
                            angle: -90,
                            position: "insideLeft",
                        }}
                    />
                    <ChartTooltip content={<ChartTooltipContent labelFormatter={(value) => `${mode === "defense" ? "Defense" : "Resistance"}: ${value}`} />} />
                    <Legend
                        formatter={(value) => {
                            const config = chartConfig[value as string];
                            return config?.label || value;
                        }}
                        wrapperStyle={{ paddingTop: "20px" }}
                    />
                    {operators.map((op) => (
                        <Line dataKey={op.id} dot={false} key={op.id} name={op.id} stroke={op.color} strokeWidth={2} type="monotone" />
                    ))}
                </LineChart>
            </ResponsiveContainer>
        </ChartContainer>
    );
}
