"use client";

import { useEffect, useState } from "react";
import { CartesianGrid, Legend, Line, LineChart, ResponsiveContainer, XAxis, YAxis } from "recharts";
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "~/components/ui/shadcn/chart";
import { Skeleton } from "~/components/ui/shadcn/skeleton";
import { calculateDpsRange, isDpsRangeResult } from "~/lib/dps-calculator";
import type { ChartDataPoint, OperatorConfiguration } from "./types";

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
                        if (!dataMap.has(point.value)) {
                            dataMap.set(point.value, { value: point.value });
                        }
                        const dataPoint = dataMap.get(point.value)!;
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

    const chartConfig = operators.reduce(
        (config, op) => {
            config[op.id] = {
                label: `${op.operatorName} E${op.params.promotion ?? 2} P${op.params.potential || 1} S${op.params.skillIndex || 1}M${op.params.masteryLevel || 0}`,
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
                    <ChartTooltip content={<ChartTooltipContent />} />
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
