import Head from "next/head";
import { motion } from "framer-motion";
import { useState } from "react";
import type { Operator } from "~/types/impl/api/static/operator";
import { Button } from "~/components/ui/button";
import OperatorSelector from "~/components/dps-calculator/operator-selector";
import { env } from "~/env";
import type { NextPage } from "next";

const DPSCalculator: NextPage<Props> = ({ data }) => {
    const [selectedOperators, setSelectedOperators] = useState<Operator[]>([]);
    const [isOperatorSelectorOpen, setIsOperatorSelectorOpen] = useState(false);

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
                    <div>
                        <Button variant={"outline"} onClick={() => setIsOperatorSelectorOpen(true)}>
                            Add Operators
                        </Button>
                        <OperatorSelector operators={data} selectedOperators={selectedOperators} isOpen={isOperatorSelectorOpen} onClose={() => setIsOperatorSelectorOpen(false)} onSelect={setSelectedOperators} />
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
