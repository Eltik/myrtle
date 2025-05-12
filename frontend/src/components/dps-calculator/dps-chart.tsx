import { useState, useEffect, useMemo, useRef } from "react";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, AreaChart, Area, BarChart, Bar, type TooltipProps } from "recharts";
import type { Operator } from "~/types/impl/api/static/operator";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Skeleton } from "~/components/ui/skeleton";
import { type NameType, type ValueType } from "recharts/types/component/DefaultTooltipContent";

// Define SelectedOperator type matching the one in dps-calculator.tsx
interface SelectedOperator extends Operator {
    instanceId: string;
    displayName: string;
}

interface DefenseChartPoint {
    defense: number;
    [operatorName: string]: number;
}

interface ResistanceChartPoint {
    resistance: number;
    [operatorName: string]: number;
}

type ChartPoint = DefenseChartPoint | ResistanceChartPoint;

type ChartDisplayMode = "line" | "area" | "bar";

interface ChartSettings {
    targets: number;
    showAverage: boolean;
    showTotal: boolean;
    maxValue: number;
    stepSize: number;
    displayMode: ChartDisplayMode;
    showDots: boolean;
    smoothCurves: boolean;
}

export interface DPSChartProps {
    operators: SelectedOperator[];
    generateChartData: () => Promise<ChartPoint[]>;
    xAxisLabel: string;
    chartType: "defense" | "resistance";
    chartSettings: ChartSettings;
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

export function DPSChart({ operators, generateChartData, xAxisLabel, chartType, chartSettings }: DPSChartProps) {
    const [data, setData] = useState<ChartPoint[]>([]);
    const [loading, setLoading] = useState(false);
    // Add a ref to track the latest request and chart type
    const requestIdRef = useRef(0);
    const lastChartTypeRef = useRef<string>(chartType);

    // Reset requestId when chart type changes to prioritize the new chart type's data
    if (lastChartTypeRef.current !== chartType) {
        requestIdRef.current = 0;
        lastChartTypeRef.current = chartType;
    }

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
            if (operator.displayName) {
                result[operator.displayName] = colors[index % colors.length] ?? "#8884d8";
            }
        });

        return result;
    }, [operators]);

    useEffect(() => {
        // Increment request ID to track the current request
        const currentRequestId = ++requestIdRef.current;
        let isMounted = true;

        const fetchData = async () => {
            if (operators.length === 0) {
                setData([]);
                return;
            }

            setLoading(true);
            try {
                const chartData = await generateChartData();
                // Only update state if this is still the latest request for the current chart type and component is mounted
                if (currentRequestId === requestIdRef.current && isMounted && lastChartTypeRef.current === chartType) {
                    setData(chartData);
                    setLoading(false);
                }
            } catch (error) {
                console.error("Error generating chart data:", error);
                if (currentRequestId === requestIdRef.current && isMounted) {
                    setLoading(false);
                }
            }
        };

        void fetchData();

        // Cleanup function to handle component unmount or dependencies changing
        return () => {
            isMounted = false;
        };
    }, [
        operators,
        generateChartData,
        chartType,
        // Only include data-affecting settings, not display-only settings
        chartSettings.targets,
        chartSettings.maxValue,
        chartSettings.stepSize,
    ]);

    const xDataKey = chartType === "defense" ? "defense" : "resistance";

    // Format numbers with k suffix for thousands
    const formatValue = (value: number) => {
        if (value >= 1000) {
            return `${(value / 1000).toFixed(1)}k`;
        }
        return value.toString();
    };

    // Function to render the appropriate chart type based on settings
    const renderChart = () => {
        const chartProps = {
            data,
            margin: {
                top: 5,
                right: 30,
                left: 20,
                bottom: 25,
            },
        };

        const axesProps = (
            <>
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
                <Legend layout="horizontal" verticalAlign="bottom" wrapperStyle={{ paddingTop: 20 }} formatter={(value: string) => <span className="text-sm font-medium">{value}</span>} />
            </>
        );

        // Determine curve type based on settings
        const curveType = chartSettings.smoothCurves ? "monotone" : "linear";

        // Common props for visualizations
        const dotProps = chartSettings.showDots ? { dot: { r: 3 } } : { dot: false };
        const activeDotProps = { activeDot: { r: 6 } };

        switch (chartSettings.displayMode) {
            case "area":
                return (
                    <AreaChart {...chartProps}>
                        {axesProps}
                        {operators.map((operator) => {
                            if (!operator.displayName) return null;
                            const color = operatorColors[operator.displayName];

                            return <Area key={operator.instanceId} type={curveType} dataKey={operator.displayName} stroke={color} fill={color} fillOpacity={0.3} strokeWidth={2} {...dotProps} {...activeDotProps} animationDuration={300} />;
                        })}
                    </AreaChart>
                );

            case "bar":
                return (
                    <BarChart {...chartProps} barGap={0} barCategoryGap={5}>
                        {axesProps}
                        {operators.map((operator) => {
                            if (!operator.displayName) return null;
                            return <Bar key={operator.instanceId} dataKey={operator.displayName} fill={operatorColors[operator.displayName]} animationDuration={300} />;
                        })}
                    </BarChart>
                );

            case "line":
            default:
                return (
                    <LineChart {...chartProps}>
                        {axesProps}
                        {operators.map((operator) => {
                            if (!operator.displayName) return null;
                            return <Line key={operator.instanceId} type={curveType} dataKey={operator.displayName} stroke={operatorColors[operator.displayName]} strokeWidth={2} {...dotProps} {...activeDotProps} animationDuration={300} />;
                        })}
                    </LineChart>
                );
        }
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
                            {renderChart()}
                        </ResponsiveContainer>
                    </div>
                ) : (
                    <div className="flex h-80 items-center justify-center text-muted-foreground">Select operators to generate DPS chart</div>
                )}
            </CardContent>
        </Card>
    );
}
