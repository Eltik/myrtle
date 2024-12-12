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

const DPSCalculator: NextPage<Props> = ({ data }) => {
    const [selectedOperators, setSelectedOperators] = useState<Operator[]>([]);
    const [dpsOperators, setDPSOperators] = useState<DPSOperator[]>([]);
    const [isOperatorSelectorOpen, setIsOperatorSelectorOpen] = useState(false);
    const [operatorParams, setOperatorParams] = useState<Record<string, OperatorParams>>({});

    const getDPSOperators = useCallback(async () => {
        for (const operator of selectedOperators) {
            const isInDPSOperators = dpsOperators.find((op) => operator.id === op.operatorData.data.id);
            if (!isInDPSOperators) {
                const data = (await (
                    await fetch("/api/getDPSOperator", {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json",
                        },
                        body: JSON.stringify({
                            id: operator.id,
                            params: operatorParams[operator.id ?? ""],
                        }),
                    })
                ).json()) as DPSOperatorResponse;

                setDPSOperators((prevDPSOperators) => [...prevDPSOperators, data.operator]);
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

    const calculateDPS = async (operator: Operator, defense: number, res: number) => {
        const params = operatorParams[operator.id ?? ""];

        const data = (await (
            await fetch("/api/dpsCalculator", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    id: operator.id,
                    params,
                    enemy: {
                        defense,
                        res,
                    },
                }),
            })
        ).json()) as DPSCalculatorResponse;

        return data.dps;
    };

    const generateChartData = async () => {
        const data = [];
        for (let defense = 0; defense <= 2000; defense += 100) {
            const point = { defense };
            for (const operator of selectedOperators) {
                const dps = await calculateDPS(operator, defense, 0);
                Object.assign(point, {
                    [operator.name]: dps,
                });
            }
            data.push(point);
        }
        return data;
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
                        <DPSChart operators={selectedOperators} generateChartData={generateChartData} />
                    </div>
                </div>
            </motion.div>
        </>
    );
};

export const getServerSideProps = async () => {
    const backendURL = env.BACKEND_URL;

    // Construct the full URL for the API endpoint
    const apiURL = `${backendURL}/static`;
    const data = (await (
        await fetch(apiURL, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                type: "operators",
            }),
        })
    ).json()) as {
        operators: Operator[];
    };

    return {
        props: {
            data: data.operators,
        },
    };
};

export default DPSCalculator;

interface Props {
    data: Operator[];
}
