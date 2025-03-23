import Head from "next/head";
import { motion } from "framer-motion";
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

type ChartDataType = "defense" | "resistance";

const DPSCalculator: NextPage<Props> = ({ data }) => {
    const [selectedOperators, setSelectedOperators] = useState<Operator[]>([]);
    const [dpsOperators, setDPSOperators] = useState<DPSOperator[]>([]);
    const [isOperatorSelectorOpen, setIsOperatorSelectorOpen] = useState(false);
    const [operatorParams, setOperatorParams] = useState<Record<string, OperatorParams>>({});
    const [chartDataType, setChartDataType] = useState<ChartDataType>("defense");

    const getDPSOperators = useCallback(async () => {
        for (const operator of selectedOperators) {
            // Check if this operator is already in dpsOperators
            const isInDPSOperators = dpsOperators.find((op) => operator.id === op.operatorData.data.id);

            if (!isInDPSOperators) {
                try {
                    const data = (await (
                        await fetch("/api/dpsCalculator", {
                            method: "POST",
                            headers: {
                                "Content-Type": "application/json",
                            },
                            body: JSON.stringify({
                                method: "operator",
                                id: operator.id,
                                params: operatorParams[operator.id ?? ""],
                            }),
                        })
                    ).json()) as DPSOperatorResponse;

                    if (data.operator) {
                        setDPSOperators((prevDPSOperators) => [...prevDPSOperators, data.operator]);
                    }
                } catch (error) {
                    console.error("Error fetching DPS operator:", error);
                }
            }
        }
    }, [selectedOperators, dpsOperators, operatorParams]);

    useEffect(() => {
        void getDPSOperators();
    }, [getDPSOperators, selectedOperators]);

    const handleOperatorParamsChange = (operatorId: string, params: OperatorParams) => {
        setOperatorParams((prevParams) => ({
            ...prevParams,
            [operatorId]: params,
        }));
    };

    const calculateDPS = async (operator: Operator, minDef: number, maxDef: number, minRes: number, maxRes: number) => {
        const params = operatorParams[operator.id ?? ""];

        const data = (await (
            await fetch("/api/dpsCalculator", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    method: "dps",
                    id: operator.id,
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

        return data.dps;
    };

    const generateChartData = async () => {
        interface DefenseChartPoint {
            defense: number;
            [operatorName: string]: number;
        }

        interface ResistanceChartPoint {
            resistance: number;
            [operatorName: string]: number;
        }

        const maxValue = 2000;
        const step = 100;

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
                            [operator.name]: point.dps,
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
                            [operator.name]: point.dps,
                        };
                    }
                });
            }

            return chartData;
        }
    };

    return (
        <>
            <Head>
                <title>myrtle.moe</title>
                <meta name="title" content="myrtle.moe" />
                <meta name="description" content="Elevate your Arknights experience to the next level." />
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
                <h1 className="mb-4 mt-2 text-2xl font-bold">Arknights DPS Calculator</h1>
                <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
                    <div className="flex flex-col gap-2">
                        <div>
                            <Button variant={"outline"} onClick={() => setIsOperatorSelectorOpen(true)}>
                                Add Operators
                            </Button>
                        </div>
                        <OperatorSelector operators={data} selectedOperators={selectedOperators} isOpen={isOperatorSelectorOpen} onClose={() => setIsOperatorSelectorOpen(false)} onSelect={setSelectedOperators} />
                        <div className="space-y-4">
                            {dpsOperators.map((operator) => (
                                <OperatorListItem key={operator.operatorData.data.id} operator={operator} onParamsChange={(params) => handleOperatorParamsChange(operator.operatorData.data.id ?? "", params)} />
                            ))}
                        </div>
                    </div>
                    <div>
                        <Tabs defaultValue="defense" className="w-full" onValueChange={(value) => setChartDataType(value as ChartDataType)}>
                            <div className="mb-4 flex items-center justify-between">
                                <TabsList>
                                    <TabsTrigger value="defense">Defense</TabsTrigger>
                                    <TabsTrigger value="resistance">Resistance</TabsTrigger>
                                </TabsList>
                            </div>
                            <TabsContent value="defense" className="mt-0">
                                <DPSChart operators={selectedOperators} generateChartData={generateChartData} xAxisLabel="Defense" chartType="defense" />
                            </TabsContent>
                            <TabsContent value="resistance" className="mt-0">
                                <DPSChart operators={selectedOperators} generateChartData={generateChartData} xAxisLabel="Resistance" chartType="resistance" />
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
