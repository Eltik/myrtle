import { useState, useEffect, useMemo } from "react";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, type TooltipProps } from "recharts";
import type { Operator } from "~/types/impl/api/static/operator";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Skeleton } from "~/components/ui/skeleton";
import { type NameType, type ValueType } from "recharts/types/component/DefaultTooltipContent";

interface DefenseChartPoint {
    defense: number;
    [operatorName: string]: number;
}

interface ResistanceChartPoint {
    resistance: number;
    [operatorName: string]: number;
}

type ChartPoint = DefenseChartPoint | ResistanceChartPoint;

export interface DPSChartProps {
    operators: Operator[];
    generateChartData: () => Promise<ChartPoint[]>;
    xAxisLabel: string;
    chartType: "defense" | "resistance";
}

// Custom tooltip component for better styling
const CustomTooltip = ({ active, payload, label, xAxisLabel }: TooltipProps<ValueType, NameType> & { xAxisLabel: string }) => {
    if (active && payload?.length) {
        return (
            <div className="max-w-[300px] rounded-md border border-border bg-background p-4 shadow-md">
                <p className="mb-2 border-b pb-1 text-sm font-medium">
                    {xAxisLabel}: {label}
                </p>
                <div className="space-y-1.5">
                    {payload.map((entry, index) => (
                        <div key={`item-${index}`} className="flex items-center justify-between gap-4">
                            <div className="flex items-center gap-2">
                                <div className="h-3 w-3 rounded-full" style={{ backgroundColor: entry.color }} />
                                <span className="text-xs font-medium">{entry.name}</span>
                            </div>
                            <span className="font-mono text-xs">{Number(entry.value).toLocaleString()} DPS</span>
                        </div>
                    ))}
                </div>
            </div>
        );
    }

    return null;
};

export function DPSChart({ operators, generateChartData, xAxisLabel, chartType }: DPSChartProps) {
    const [data, setData] = useState<ChartPoint[]>([]);
    const [loading, setLoading] = useState(false);

    // Generate a unique color for each operator based on their index
    const operatorColors = useMemo(() => {
        const colors = [
            "#FF6B6B", // Coral Red
            "#4ECDC4", // Medium Turquoise
            "#FFD166", // Mustard
            "#6A0572", // Purple
            "#7371FC", // Lavender
            "#36A2EB", // Blue
            "#22577A", // Dark Teal
            "#FF5722", // Deep Orange
            "#8338EC", // Purple
            "#FB5607", // Orange
            "#3A86FF", // Royal Blue
            "#06D6A0", // Green
            "#FCBF49", // Yellow
            "#EF476F", // Pink
            "#9D4EDD", // Purple
            "#2B9348", // Green
            "#0077B6", // Blue
            "#FF9500", // Orange
            "#C1121F", // Red
            "#3C096C", // Dark Purple
        ];

        // Create a mapping of operator names to colors
        const result: Record<string, string> = {};

        operators.forEach((operator, index) => {
            if (operator.name) {
                Object.assign(result, { [operator.name]: colors[index % colors.length] });
            }
        });

        return result;
    }, [operators]);

    useEffect(() => {
        const fetchData = async () => {
            if (operators.length === 0) {
                setData([]);
                return;
            }

            setLoading(true);
            try {
                const chartData = await generateChartData();
                setData(chartData);
            } catch (error) {
                console.error("Error generating chart data:", error);
            } finally {
                setLoading(false);
            }
        };

        void fetchData();
    }, [operators, generateChartData, chartType]);

    const xDataKey = chartType === "defense" ? "defense" : "resistance";

    // Format numbers with k suffix for thousands
    const formatValue = (value: number) => {
        if (value >= 1000) {
            return `${(value / 1000).toFixed(1)}k`;
        }
        return value.toString();
    };

    return (
        <Card className="h-full w-full">
            <CardHeader className="pb-2">
                <CardTitle className="text-lg">{operators.length > 0 ? `DPS vs. ${xAxisLabel}` : "Select operators to see DPS chart"}</CardTitle>
            </CardHeader>
            <CardContent>
                {loading ? (
                    <div className="flex flex-col space-y-3">
                        <Skeleton className="h-[300px] w-full rounded-md" />
                    </div>
                ) : operators.length > 0 ? (
                    <div className="h-[450px] w-full">
                        <ResponsiveContainer width="100%" height="100%">
                            <LineChart
                                data={data}
                                margin={{
                                    top: 5,
                                    right: 30,
                                    left: 20,
                                    bottom: 25,
                                }}
                            >
                                <CartesianGrid strokeDasharray="3 3" opacity={0.2} />
                                <XAxis
                                    dataKey={xDataKey}
                                    label={{
                                        value: xAxisLabel,
                                        position: "insideBottomRight",
                                        offset: -10,
                                    }}
                                    tickFormatter={formatValue}
                                />
                                <YAxis
                                    label={{
                                        value: "DPS",
                                        angle: -90,
                                        position: "insideLeft",
                                    }}
                                    tickFormatter={formatValue}
                                />
                                <Tooltip content={<CustomTooltip xAxisLabel={xAxisLabel} />} cursor={{ strokeDasharray: "3 3", stroke: "#9ca3af", strokeWidth: 1 }} />
                                <Legend layout="horizontal" verticalAlign="bottom" wrapperStyle={{ paddingTop: 20 }} formatter={(value) => <span className="text-sm font-medium">{value}</span>} />
                                {operators.map((operator) => {
                                    // Skip operators without a name
                                    if (!operator.name) return null;

                                    const operatorKey = operator.id ?? `op-${Math.random()}`;

                                    return <Line key={operatorKey} type="monotone" dataKey={operator.name} stroke={operatorColors[operator.name]} strokeWidth={2} dot={false} activeDot={{ r: 6 }} animationDuration={300} />;
                                })}
                            </LineChart>
                        </ResponsiveContainer>
                    </div>
                ) : (
                    <div className="flex h-80 items-center justify-center text-muted-foreground">Select operators to generate DPS chart</div>
                )}
            </CardContent>
        </Card>
    );
}
