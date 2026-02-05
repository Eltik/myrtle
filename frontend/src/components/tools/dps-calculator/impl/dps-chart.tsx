"use client";

import { forwardRef, useEffect, useImperativeHandle, useMemo, useRef, useState } from "react";
import { Area, Bar, CartesianGrid, ComposedChart, Legend, Line, ReferenceLine, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { ChartContainer } from "~/components/ui/shadcn/chart";
import { Skeleton } from "~/components/ui/shadcn/skeleton";
import { useDpsChartSettings } from "~/context/dps-chart-settings-context";
import { calculateDpsRange, isDpsRangeResult } from "~/lib/dps-calculator";
import { formatChartNumber, getGridStrokeDasharray } from "~/lib/dps-chart-settings";
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

    // Targets (only show if more than 1)
    const targets = op.params.targets ?? 1;
    if (targets > 1) {
        parts.push(`${targets}T`);
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

export interface DpsChartHandle {
    getChartElement: () => HTMLDivElement | null;
    getChartData: () => ChartDataPoint[];
    getOperators: () => OperatorConfiguration[];
}

interface DpsChartProps {
    operators: OperatorConfiguration[];
    mode: "defense" | "resistance";
    /** Ref passed as prop for dynamic import compatibility */
    chartRef?: React.RefObject<DpsChartHandle | null>;
}

// Custom tooltip component
function CustomTooltip({ active, payload, label, mode, formatLabel }: { active?: boolean; payload?: Array<{ dataKey: string; value: number; color: string; name: string }>; label?: number; mode: string; formatLabel: (v: number) => string }) {
    if (!active || !payload || payload.length === 0) return null;

    // Deduplicate entries by dataKey (prevents duplicates when area fill + line are both present)
    const uniqueEntries = payload.filter((entry, index, self) => self.findIndex((e) => e.dataKey === entry.dataKey) === index);

    return (
        <div className="rounded-lg border bg-background p-2 shadow-md">
            <p className="mb-1 font-medium text-sm">
                {mode === "defense" ? "DEF" : "RES"}: {label}
            </p>
            {uniqueEntries.map((entry) => (
                <p className="text-sm" key={entry.dataKey} style={{ color: entry.color }}>
                    {entry.name}: {formatLabel(entry.value)}
                </p>
            ))}
        </div>
    );
}

// Custom label component for data points
function CustomLabel({ x, y, value, formatLabel, index, interval, color }: { x?: number; y?: number; value?: number; formatLabel: (v: number) => string; index?: number; interval: number; color?: string }) {
    if (index === undefined || index % interval !== 0 || value === undefined) return null;

    return (
        <text fill={color || "#888888"} fontSize={10} textAnchor="middle" x={x} y={(y ?? 0) - 10}>
            {formatLabel(value)}
        </text>
    );
}

export const DpsChart = forwardRef<DpsChartHandle, DpsChartProps>(function DpsChart({ operators, mode, chartRef }, ref) {
    const containerRef = useRef<HTMLDivElement>(null);
    const [chartData, setChartData] = useState<ChartDataPoint[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const { settings } = useDpsChartSettings();

    // Expose methods via ref for export functionality
    useImperativeHandle(ref, () => ({
        getChartElement: () => containerRef.current,
        getChartData: () => chartData,
        getOperators: () => operators,
    }));

    // Also support chartRef prop for dynamic import compatibility
    // This is needed because Next.js dynamic() doesn't forward refs
    useImperativeHandle(chartRef, () => ({
        getChartElement: () => containerRef.current,
        getChartData: () => chartData,
        getOperators: () => operators,
    }));

    useEffect(() => {
        const fetchDpsData = async () => {
            if (operators.length === 0) {
                setChartData([]);
                return;
            }

            setLoading(true);
            setError(null);

            try {
                // Fetch DPS data for all operators in parallel using settings.range
                const results = await Promise.all(
                    operators.map(async (op) => {
                        try {
                            const result = await calculateDpsRange(op.operatorId, op.params, {
                                minDef: settings.range.minDef,
                                maxDef: settings.range.maxDef,
                                defStep: settings.range.defStep,
                                minRes: settings.range.minRes,
                                maxRes: settings.range.maxRes,
                                resStep: settings.range.resStep,
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
    }, [operators, mode, settings.range]);

    // Build chart config from operators with detailed labels
    const chartConfig = useMemo(
        () =>
            operators.reduce(
                (config, op) => {
                    config[op.id] = {
                        label: buildOperatorLabel(op),
                        color: op.color,
                    };
                    return config;
                },
                {} as Record<string, { label: string; color: string }>,
            ),
        [operators],
    );

    // Custom label formatter for data labels
    const formatLabel = useMemo(() => {
        return (value: number) => formatChartNumber(value, settings.dataDisplay.numberFormat, settings.dataDisplay.decimalPlaces);
    }, [settings.dataDisplay.numberFormat, settings.dataDisplay.decimalPlaces]);

    // Determine legend layout props based on position
    const legendProps = useMemo(() => {
        const legendPosition = settings.visual.legendPosition;
        if (legendPosition === "top") {
            return { verticalAlign: "top" as const, align: "center" as const, wrapperStyle: { paddingBottom: "20px" } };
        }
        if (legendPosition === "left") {
            return { verticalAlign: "middle" as const, align: "left" as const, layout: "vertical" as const, wrapperStyle: { paddingRight: "20px" } };
        }
        if (legendPosition === "right") {
            return { verticalAlign: "middle" as const, align: "right" as const, layout: "vertical" as const, wrapperStyle: { paddingLeft: "20px" } };
        }
        // "bottom" and default
        return { verticalAlign: "bottom" as const, align: "center" as const, wrapperStyle: { paddingTop: "20px" } };
    }, [settings.visual]);

    if (loading) {
        return (
            <div className="space-y-3">
                <Skeleton className="h-8 w-full" />
                <Skeleton className="w-full" style={{ height: settings.visual.chartHeight }} />
            </div>
        );
    }

    if (error) {
        return (
            <div className="flex items-center justify-center rounded-lg border border-destructive/50 bg-destructive/10 text-destructive" style={{ height: settings.visual.chartHeight }}>
                <p className="text-sm">{error}</p>
            </div>
        );
    }

    if (chartData.length === 0) {
        return (
            <div className="flex items-center justify-center text-muted-foreground text-sm" style={{ height: settings.visual.chartHeight }}>
                No data to display
            </div>
        );
    }

    const { visual, dataDisplay } = settings;

    // Render chart elements based on chart type
    const renderChartElements = () => {
        const elements: React.ReactNode[] = [];

        operators.forEach((op) => {
            const commonProps = {
                key: op.id,
                dataKey: op.id,
                name: chartConfig[op.id]?.label || op.operatorName,
                isAnimationActive: visual.enableAnimation,
            };

            if (visual.chartType === "bar") {
                elements.push(<Bar {...commonProps} fill={op.color} fillOpacity={0.8} />);
            } else if (visual.chartType === "area") {
                elements.push(
                    <Area
                        {...commonProps}
                        activeDot={visual.showDots ? { r: visual.dotSize + 2 } : { r: 4 }}
                        dot={visual.showDots ? { r: visual.dotSize, fill: op.color } : false}
                        fill={op.color}
                        fillOpacity={visual.areaFillOpacity}
                        label={dataDisplay.showDataLabels ? (props: { x?: number; y?: number; value?: number; index?: number }) => <CustomLabel color={op.color} formatLabel={formatLabel} index={props.index} interval={dataDisplay.dataLabelInterval} value={props.value} x={props.x} y={props.y} /> : undefined}
                        stroke={op.color}
                        strokeWidth={visual.strokeWidth}
                        type={visual.lineType}
                    />,
                );
            } else {
                // "line" chart type (default)
                // Optionally add area fill underneath (legendType="none" prevents duplicate legend entries)
                if (visual.showAreaFill) {
                    elements.push(<Area dataKey={op.id} fill={op.color} fillOpacity={visual.areaFillOpacity} isAnimationActive={visual.enableAnimation} key={`area-${op.id}`} legendType="none" name={chartConfig[op.id]?.label || op.operatorName} stroke="none" type={visual.lineType} />);
                }
                elements.push(
                    <Line
                        {...commonProps}
                        activeDot={visual.showDots ? { r: visual.dotSize + 2 } : { r: 4 }}
                        dot={visual.showDots ? { r: visual.dotSize, fill: op.color } : false}
                        label={dataDisplay.showDataLabels ? (props: { x?: number; y?: number; value?: number; index?: number }) => <CustomLabel color={op.color} formatLabel={formatLabel} index={props.index} interval={dataDisplay.dataLabelInterval} value={props.value} x={props.x} y={props.y} /> : undefined}
                        stroke={op.color}
                        strokeWidth={visual.strokeWidth}
                        type={visual.lineType}
                    />,
                );
            }
        });

        return elements;
    };

    return (
        <div ref={containerRef}>
            <ChartContainer className="w-full" config={chartConfig} style={{ height: visual.chartHeight }}>
                <ResponsiveContainer height="100%" width="100%">
                    <ComposedChart data={chartData} margin={{ top: 20, right: 30, left: 20, bottom: 20 }}>
                        {/* Grid */}
                        {visual.showGrid && <CartesianGrid className="stroke-border/50" strokeDasharray={getGridStrokeDasharray(visual.gridStyle)} />}

                        {/* Axes */}
                        <XAxis
                            className="text-muted-foreground"
                            dataKey="value"
                            domain={["auto", "auto"]}
                            label={
                                dataDisplay.showAxisLabels
                                    ? {
                                          value: mode === "defense" ? "Defense" : "Resistance",
                                          position: "insideBottom",
                                          offset: -10,
                                          className: "fill-muted-foreground",
                                      }
                                    : undefined
                            }
                            scale={dataDisplay.yAxisScale === "log" ? "log" : "auto"}
                            tickFormatter={(value) => String(value)}
                        />
                        <YAxis
                            className="text-muted-foreground"
                            domain={["auto", "auto"]}
                            label={
                                dataDisplay.showAxisLabels
                                    ? {
                                          value: "DPS",
                                          angle: -90,
                                          position: "insideLeft",
                                          className: "fill-muted-foreground",
                                      }
                                    : undefined
                            }
                            scale={dataDisplay.yAxisScale === "log" ? "log" : "auto"}
                            tickFormatter={formatLabel}
                        />

                        {/* Reference Lines - using orange color for visibility on both light/dark themes and exports */}
                        {dataDisplay.showReferenceLines &&
                            dataDisplay.referenceLineValues.map((value) => (
                                <ReferenceLine
                                    key={`ref-${value}`}
                                    label={{
                                        value: formatLabel(value),
                                        position: "right",
                                        fill: "#f97316",
                                        fontSize: 10,
                                    }}
                                    stroke="#f97316"
                                    strokeDasharray="5 5"
                                    strokeWidth={1.5}
                                    y={value}
                                />
                            ))}

                        {/* Tooltip */}
                        {dataDisplay.showTooltip && <Tooltip content={<CustomTooltip formatLabel={formatLabel} mode={mode} />} />}

                        {/* Legend */}
                        {visual.legendPosition !== "none" && <Legend {...legendProps} />}

                        {/* Chart elements (lines, areas, or bars) */}
                        {renderChartElements()}
                    </ComposedChart>
                </ResponsiveContainer>
            </ChartContainer>
        </div>
    );
});
