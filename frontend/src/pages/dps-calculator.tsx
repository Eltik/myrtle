import Head from "next/head";
import { motion, AnimatePresence } from "framer-motion";
import { useCallback, useEffect, useState } from "react";
import type { Operator } from "~/types/impl/api/static/operator";
import { Button } from "~/components/ui/button";
import OperatorSelector from "~/components/dps-calculator/operator-selector";
import { env } from "~/env";
import type { NextPage } from "next";
import OperatorListItem from "~/components/dps-calculator/operator-list-item";
import type { DPSCalculatorResponse, DPSOperator, DPSOperatorResponse, OperatorParams } from "~/types/impl/api/impl/dps-calculator";
import { DPSChart } from "~/components/dps-calculator/dps-chart";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Separator } from "~/components/ui/separator";
import { Label } from "~/components/ui/label";
import { Switch } from "~/components/ui/switch";
import { Slider } from "~/components/ui/slider";
import { RadioGroup, RadioGroupItem } from "~/components/ui/radio-group";
import Link from "next/link";
import { ChevronDown } from "lucide-react";
import { Input } from "~/components/ui/input";

type ChartDataType = "defense" | "resistance";
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

interface SelectedOperator extends Operator {
    instanceId: string;
    displayName: string;
}

interface SelectedDPSOperator extends DPSOperator {
    instanceId: string;
    displayName: string;
}

interface OperatorDPSStats {
    averageDPS: number;
    totalDPS: number;
}

const DPSCalculator: NextPage<Props> = ({ data }) => {
    const [selectedOperators, setSelectedOperators] = useState<SelectedOperator[]>([]);
    const [dpsOperators, setDPSOperators] = useState<SelectedDPSOperator[]>([]);
    const [isOperatorSelectorOpen, setIsOperatorSelectorOpen] = useState(false);
    const [operatorParams, setOperatorParams] = useState<Record<string, OperatorParams>>({});
    const [chartDataType, setChartDataType] = useState<ChartDataType>("defense");
    const [chartSettings, setChartSettings] = useState<ChartSettings>({
        targets: 1,
        showAverage: false,
        showTotal: false,
        maxValue: 2000,
        stepSize: 100,
        displayMode: "line",
        showDots: false,
        smoothCurves: true,
    });
    const [operatorDPSStats, setOperatorDPSStats] = useState<Record<string, OperatorDPSStats>>({});
    const [isChartSettingsOpen, setIsChartSettingsOpen] = useState(true);

    const handleOperatorSelectionChange = (newSelectedBaseOps: Operator[]) => {
        const newBaseOpIds = new Set(newSelectedBaseOps.map((op) => op.id));

        setSelectedOperators((prevSelectedOperators) => {
            const nextSelectedOperators: SelectedOperator[] = [];
            const existingInstanceCounts: Record<string, number> = {};

            // Keep existing operators that are still selected in the modal
            prevSelectedOperators.forEach((op) => {
                if (newBaseOpIds.has(op.id)) {
                    nextSelectedOperators.push(op); // Keep existing instanceId and displayName
                    Object.assign(existingInstanceCounts, {
                        [op.id ?? ""]: (existingInstanceCounts[op.id ?? ""] ?? 0) + 1,
                    });
                }
            });

            // Add new operators that were not previously present (as single instances)
            newSelectedBaseOps.forEach((baseOp) => {
                if (!nextSelectedOperators.some((selOp) => selOp.id === baseOp.id)) {
                    const instanceId = `${baseOp.id}_${Date.now()}`;
                    // For operators added via selector, their initial displayName is just their name
                    // Duplicates will get suffixes later via a duplicate button.
                    const displayName = baseOp.name;
                    nextSelectedOperators.push({ ...baseOp, instanceId, displayName });
                }
            });
            return nextSelectedOperators;
        });

        // Update dpsOperators and operatorParams based on the new selectedOperators
        // This needs to be done more carefully, perhaps in an effect listening to selectedOperators
    };

    useEffect(() => {
        // Synchronize dpsOperators and operatorParams with selectedOperators
        setDPSOperators((prevDpsOps) => {
            const newDpsOps: SelectedDPSOperator[] = [];
            const currentSelectedInstanceIds = new Set(selectedOperators.map((op) => op.instanceId));
            prevDpsOps.forEach((dpsOp) => {
                if (currentSelectedInstanceIds.has(dpsOp.instanceId)) {
                    newDpsOps.push(dpsOp);
                }
            });
            return newDpsOps;
        });

        setOperatorParams((prevParams) => {
            const newOpParams: Record<string, OperatorParams> = {};
            const currentSelectedInstanceIds = new Set(selectedOperators.map((op) => op.instanceId));
            // Iterate over the keys of prevParams which are known to be strings
            Object.keys(prevParams).forEach((instanceId) => {
                if (currentSelectedInstanceIds.has(instanceId)) {
                    const params = prevParams[instanceId];
                    if (params !== undefined) {
                        newOpParams[instanceId] = params;
                    }
                }
            });
            return newOpParams;
        });
    }, [selectedOperators]);

    const getDPSOperators = useCallback(async () => {
        for (const operator of selectedOperators) {
            const isInDPSOperators = dpsOperators.find((op) => op.instanceId === operator.instanceId);

            if (!isInDPSOperators) {
                try {
                    const paramsForFetch = operatorParams[operator.instanceId];
                    const fetchedOperatorData = (await (
                        await fetch("/api/dpsCalculator", {
                            method: "POST",
                            headers: {
                                "Content-Type": "application/json",
                            },
                            body: JSON.stringify({
                                method: "operator",
                                id: operator.id, // Use base ID for fetching
                                params: paramsForFetch,
                            }),
                        })
                    ).json()) as DPSOperatorResponse;

                    if (fetchedOperatorData.operator) {
                        setDPSOperators((prevDPSOperators) => {
                            // Check if the instanceId already exists before adding
                            if (prevDPSOperators.some((op) => op.instanceId === operator.instanceId)) {
                                return prevDPSOperators; // Already exists, don't add again
                            }
                            return [...prevDPSOperators, { ...fetchedOperatorData.operator, instanceId: operator.instanceId, displayName: operator.displayName }];
                        });
                    }
                } catch (error) {
                    console.error("Error fetching DPS operator:", error);
                }
            }
        }
    }, [selectedOperators, dpsOperators, operatorParams]);

    useEffect(() => {
        void getDPSOperators();
    }, [selectedOperators, getDPSOperators]);

    const handleOperatorParamsChange = (instanceId: string, params: OperatorParams) => {
        setOperatorParams((prevParams) => ({
            ...prevParams,
            [instanceId]: params,
        }));
    };

    const handleChartSettingChange = <K extends keyof ChartSettings>(key: K, value: ChartSettings[K]) => {
        setChartSettings((prev) => ({
            ...prev,
            [key]: value,
        }));
    };

    const calculateDPS = useCallback(
        async (operator: SelectedOperator, minDef: number, maxDef: number, minRes: number, maxRes: number) => {
            const params = {
                ...(operatorParams[operator.instanceId] ?? {}),
                targets: chartSettings.targets,
            };

            const data = (await (
                await fetch("/api/dpsCalculator", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        method: "dps",
                        id: operator.id, // Use base ID for API
                        params,
                        range: {
                            minDef,
                            maxDef,
                            minRes,
                            maxRes,
                        },
                    }),
                })
            ).json()) as DPSCalculatorResponse;

            // Store DPS stats for this operator instance
            setOperatorDPSStats((prev) => ({
                ...prev,
                [operator.instanceId]: {
                    averageDPS: data.averageDPS,
                    totalDPS: data.totalDPS,
                },
            }));

            return data.dps;
        },
        [operatorParams, chartSettings.targets],
    );

    const generateChartData = useCallback(async () => {
        interface DefenseChartPoint {
            defense: number;
            [operatorName: string]: number;
        }

        interface ResistanceChartPoint {
            resistance: number;
            [operatorName: string]: number;
        }

        const maxValue = chartSettings.maxValue;
        const step = chartSettings.stepSize;

        if (chartDataType === "defense") {
            const chartData: DefenseChartPoint[] = [];

            // Create all defense data points
            for (let defense = 0; defense <= maxValue; defense += step) {
                chartData.push({ defense });
            }

            // Fetch DPS data for all operators with a single request per operator
            for (const operator of selectedOperators) {
                const dpsResult = await calculateDPS(operator, 0, maxValue, 0, 0);

                // Map the results to the appropriate data points
                dpsResult.def.forEach((point) => {
                    const index = Math.floor(point.def / step);
                    if (index >= 0 && index < chartData.length && chartData[index]) {
                        const currentPoint = chartData[index];
                        chartData[index] = {
                            ...currentPoint,
                            [operator.displayName]: point.dps,
                        };
                    }
                });
            }

            return chartData;
        } else {
            const chartData: ResistanceChartPoint[] = [];

            // Create all resistance data points
            for (let resistance = 0; resistance <= maxValue; resistance += step) {
                chartData.push({ resistance });
            }

            // Fetch DPS data for all operators with a single request per operator
            for (const operator of selectedOperators) {
                const dpsResult = await calculateDPS(operator, 0, 0, 0, maxValue);

                // Map the results to the appropriate data points
                dpsResult.res.forEach((point) => {
                    const index = Math.floor(point.res / step);
                    if (index >= 0 && index < chartData.length && chartData[index]) {
                        const currentPoint = chartData[index];
                        chartData[index] = {
                            ...currentPoint,
                            [operator.displayName]: point.dps,
                        };
                    }
                });
            }

            return chartData;
        }
    }, [chartDataType, chartSettings.maxValue, chartSettings.stepSize, selectedOperators, calculateDPS]);

    // Simplified chart type change handler
    const handleChartTypeChange = (value: string) => {
        setChartDataType(value as ChartDataType);
    };

    const handleRemoveOperator = (instanceIdToRemove: string) => {
        // Remove from selectedOperators first to trigger useEffect for dpsOperators and operatorParams
        setSelectedOperators((prevOperators) => prevOperators.filter((op) => op.instanceId !== instanceIdToRemove));

        // Remove from operatorDPSStats directly
        setOperatorDPSStats((prevStats) => {
            const newStats = { ...prevStats };
            delete newStats[instanceIdToRemove];
            return newStats;
        });
    };

    const handleDuplicateOperator = (instanceIdToDuplicate: string) => {
        const operatorToDuplicate = selectedOperators.find((op) => op.instanceId === instanceIdToDuplicate);
        if (!operatorToDuplicate) return;

        // Create a new base operator object by stripping instance-specific fields
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        const { instanceId: _i, displayName: _d, ...baseOperatorData } = operatorToDuplicate;
        const baseOp = baseOperatorData as Operator;

        const existingInstancesOfThisType = selectedOperators.filter((op) => op.id === baseOp.id);
        const newInstanceNumber = existingInstancesOfThisType.length + 1;

        const newInstanceId = `${baseOp.id}_${Date.now()}`;
        const newDisplayName = `${baseOp.name} (${newInstanceNumber})`;

        const duplicatedOp: SelectedOperator = {
            ...baseOp, // Spread the original base operator data
            instanceId: newInstanceId,
            displayName: newDisplayName,
        };

        setSelectedOperators((prev) => [...prev, duplicatedOp]);

        // Optionally, copy parameters from the duplicated operator
        const paramsToCopy = operatorParams[instanceIdToDuplicate];
        if (paramsToCopy) {
            setOperatorParams((prev) => ({
                ...prev,
                [newInstanceId]: { ...paramsToCopy },
            }));
        }
    };

    return (
        <>
            <Head>
                <title>DPS Calculator</title>
                <meta name="title" content="myrtle.moe" />
                <meta name="description" content="Calculate DPS for Arknights operators." />
                <link rel="icon" href="/favicon.ico" />
            </Head>
            <motion.div
                initial="hidden"
                animate="visible"
                exit={"hidden"}
                variants={{
                    hidden: { opacity: 0 },
                    visible: {
                        opacity: 1,
                        transition: { staggerChildren: 0.2 },
                    },
                }}
                className="container mx-auto p-4"
            >
                <h1 className="mt-2 text-2xl font-bold">Arknights DPS Calculator</h1>
                <p className="mb-4 text-xs md:text-sm">
                    Display DPS for a single operator or compare multiple operators. Please note that the calculator may not have all operators available.
                    <br />
                    <b>Note:</b> This DPS calculator uses calculations from{" "}
                    <Link href={"http://github.com/WhoAteMyCQQkie/ArknightsDpsCompare/"} className="text-blue-500 hover:underline">
                        WhoAteMyCQQkie/ArknightsDpsCompare
                    </Link>
                    . All credit for the DPS calculations goes to them.
                </p>
                <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
                    <div className="flex flex-col gap-2">
                        <div>
                            <Button variant={"outline"} onClick={() => setIsOperatorSelectorOpen(true)}>
                                Add Operators
                            </Button>
                        </div>
                        <OperatorSelector
                            operators={data}
                            selectedOperators={selectedOperators} // Pass SelectedOperator[]
                            isOpen={isOperatorSelectorOpen}
                            onClose={() => setIsOperatorSelectorOpen(false)}
                            onSelect={handleOperatorSelectionChange}
                        />

                        {/* Collapsible Chart settings card */}
                        <Card className={`mb-4 w-full ${isChartSettingsOpen ? "" : "transition-all duration-150 hover:bg-primary-foreground"}`}>
                            <button className="w-full text-left focus:outline-none" onClick={() => setIsChartSettingsOpen(!isChartSettingsOpen)} aria-expanded={isChartSettingsOpen}>
                                <CardHeader className="flex flex-row items-center justify-between">
                                    <CardTitle className="text-lg">Chart Settings</CardTitle>
                                    <motion.div animate={{ rotate: isChartSettingsOpen ? 180 : 0 }} transition={{ duration: 0.3 }}>
                                        <ChevronDown className="h-4 w-4" />
                                    </motion.div>
                                </CardHeader>
                            </button>
                            <AnimatePresence initial={false}>
                                {isChartSettingsOpen && (
                                    <motion.div
                                        initial="collapsed"
                                        animate="open"
                                        exit="collapsed"
                                        variants={{
                                            open: { opacity: 1, height: "auto" },
                                            collapsed: { opacity: 0, height: 0 },
                                        }}
                                        transition={{ duration: 0.3, ease: "easeInOut" }}
                                    >
                                        <CardContent className="space-y-4">
                                            <div className="space-y-2">
                                                <Label>Number of Enemy Targets</Label>
                                                <Input type="number" min={1} max={99} value={chartSettings.targets} onChange={(e) => handleChartSettingChange("targets", Math.max(1, Math.min(99, parseInt(e.target.value) || 1)))} className="w-24" />
                                            </div>

                                            <Separator />

                                            <div className="space-y-2">
                                                <Label>Display Options</Label>
                                                <div className="grid grid-cols-2 gap-4">
                                                    <div className="flex items-center space-x-2">
                                                        <Switch id="show-average" checked={chartSettings.showAverage} onCheckedChange={(checked) => handleChartSettingChange("showAverage", checked)} />
                                                        <Label htmlFor="show-average">Show Average DPS</Label>
                                                    </div>
                                                    <div className="flex items-center space-x-2">
                                                        <Switch id="show-total" checked={chartSettings.showTotal} onCheckedChange={(checked) => handleChartSettingChange("showTotal", checked)} />
                                                        <Label htmlFor="show-total">Show Total DPS</Label>
                                                    </div>
                                                </div>

                                                {(chartSettings.showAverage || chartSettings.showTotal) && Object.keys(operatorDPSStats).length > 0 ? (
                                                    <div className="mt-2 space-y-2 rounded-md bg-secondary p-2 text-sm">
                                                        {selectedOperators.map((operator) => {
                                                            const stats = operatorDPSStats[operator.instanceId];
                                                            if (!stats) return null;

                                                            return (
                                                                <div key={operator.instanceId} className="flex justify-between">
                                                                    <span className="font-medium">{operator.displayName}:</span>
                                                                    <div className="space-x-4">
                                                                        {chartSettings.showAverage && <span>Avg: {stats.averageDPS.toLocaleString()}</span>}
                                                                        {chartSettings.showTotal && <span>Total: {stats.totalDPS.toLocaleString()}</span>}
                                                                    </div>
                                                                </div>
                                                            );
                                                        })}
                                                    </div>
                                                ) : null}
                                            </div>

                                            <Separator />

                                            <div className="space-y-2">
                                                <Label>Value Range</Label>
                                                <div className="space-y-4">
                                                    <div className="space-y-2">
                                                        <div className="flex justify-between">
                                                            <Label htmlFor="max-value">
                                                                Maximum {chartDataType === "defense" ? "DEF" : "RES"} Value: {chartSettings.maxValue}
                                                            </Label>
                                                        </div>
                                                        <Slider id="max-value" min={500} max={5000} step={500} value={[chartSettings.maxValue]} onValueChange={(values) => handleChartSettingChange("maxValue", values[0] ?? 2000)} />
                                                    </div>
                                                    <div className="space-y-2">
                                                        <div className="flex justify-between">
                                                            <Label htmlFor="step-size">Step Size: {chartSettings.stepSize}</Label>
                                                        </div>
                                                        <Slider id="step-size" min={50} max={500} step={50} value={[chartSettings.stepSize]} onValueChange={(values) => handleChartSettingChange("stepSize", values[0] ?? 100)} />
                                                    </div>
                                                </div>
                                            </div>

                                            <Separator />

                                            <div className="space-y-2">
                                                <Label>Chart Style</Label>
                                                <RadioGroup defaultValue={chartSettings.displayMode} onValueChange={(value: string) => handleChartSettingChange("displayMode", value as ChartDisplayMode)} className="grid grid-cols-3 gap-4">
                                                    <div className="flex items-center space-x-2">
                                                        <RadioGroupItem value="line" id="chart-line" />
                                                        <Label htmlFor="chart-line">Line</Label>
                                                    </div>
                                                    <div className="flex items-center space-x-2">
                                                        <RadioGroupItem value="area" id="chart-area" />
                                                        <Label htmlFor="chart-area">Area</Label>
                                                    </div>
                                                    <div className="flex items-center space-x-2">
                                                        <RadioGroupItem value="bar" id="chart-bar" />
                                                        <Label htmlFor="chart-bar">Bar</Label>
                                                    </div>
                                                </RadioGroup>

                                                <div className="mt-2 grid grid-cols-2 gap-4">
                                                    <div className="flex items-center space-x-2">
                                                        <Switch id="show-dots" checked={chartSettings.showDots} onCheckedChange={(checked) => handleChartSettingChange("showDots", checked)} />
                                                        <Label htmlFor="show-dots">Show Data Points</Label>
                                                    </div>
                                                    <div className="flex items-center space-x-2">
                                                        <Switch id="smooth-curves" checked={chartSettings.smoothCurves} onCheckedChange={(checked) => handleChartSettingChange("smoothCurves", checked)} />
                                                        <Label htmlFor="smooth-curves">Smooth Curves</Label>
                                                    </div>
                                                </div>
                                            </div>
                                        </CardContent>
                                    </motion.div>
                                )}
                            </AnimatePresence>
                        </Card>

                        <div className="space-y-4">
                            {dpsOperators.map((operator) => (
                                <OperatorListItem key={operator.instanceId} operator={operator} onParamsChange={(params) => handleOperatorParamsChange(operator.instanceId, params)} onRemove={handleRemoveOperator} onDuplicate={handleDuplicateOperator} />
                            ))}
                        </div>
                    </div>
                    <div>
                        <Tabs defaultValue="defense" className="w-full" onValueChange={handleChartTypeChange}>
                            <div className="mb-4 flex items-center justify-between">
                                <TabsList>
                                    <TabsTrigger value="defense">Defense</TabsTrigger>
                                    <TabsTrigger value="resistance">Resistance</TabsTrigger>
                                </TabsList>
                            </div>
                            <TabsContent value="defense" className="mt-0">
                                <DPSChart
                                    operators={selectedOperators} // Pass SelectedOperator[]
                                    generateChartData={generateChartData}
                                    xAxisLabel="Defense"
                                    chartType="defense"
                                    chartSettings={chartSettings}
                                />
                            </TabsContent>
                            <TabsContent value="resistance" className="mt-0">
                                <DPSChart
                                    operators={selectedOperators} // Pass SelectedOperator[]
                                    generateChartData={generateChartData}
                                    xAxisLabel="Resistance"
                                    chartType="resistance"
                                    chartSettings={chartSettings}
                                />
                            </TabsContent>
                        </Tabs>
                    </div>
                </div>
            </motion.div>
        </>
    );
};

export const getServerSideProps = async () => {
    const backendURL = env.BACKEND_URL;

    // Construct the full URL for the API endpoint
    const apiURL = `${backendURL}/dps-calculator`;
    const response = (await (
        await fetch(apiURL, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                method: "operator",
            }),
        })
    ).json()) as { operators: DPSOperator[] };

    // The new API returns operators directly
    const operators = response.operators || [];

    return {
        props: {
            data: operators,
        },
    };
};

export default DPSCalculator;

interface Props {
    data: Operator[];
}
