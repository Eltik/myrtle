import type { GetServerSideProps, NextPage } from "next";
import { SEO } from "~/components/seo";
import { DpsCalculator } from "~/components/tools/dps-calculator";
import { env } from "~/env";
import type { DpsOperatorListEntry } from "~/types/api/impl/dps-calculator";

interface Props {
    operators: DpsOperatorListEntry[];
}

const DpsCalculatorPage: NextPage<Props> = ({ operators }) => {
    return (
        <>
            <SEO
                description="Calculate and compare operator DPS in Arknights with 100% accuracy. Generate DPS curves against different enemy defense and resistance values. Supports 280+ operators."
                keywords={["DPS calculator", "damage calculator", "operator damage", "DPS comparison", "enemy defense"]}
                path="/tools/dps"
                title="DPS Calculator"
            />
            <DpsCalculator operators={operators} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async () => {
    const backendURL = env.BACKEND_URL;

    try {
        // Fetch list of operators with DPS calculators
        const response = await fetch(`${backendURL}/dps-calculator/operators`, {
            method: "GET",
            headers: { "Content-Type": "application/json" },
        });

        if (!response.ok) {
            console.error("Failed to fetch DPS operators:", response.status);
            return { notFound: true };
        }

        const data = (await response.json()) as {
            count: number;
            operators: DpsOperatorListEntry[];
        };

        if (!data.operators || !Array.isArray(data.operators)) {
            console.error("Invalid DPS operators data structure");
            return { notFound: true };
        }

        return {
            props: {
                operators: data.operators,
            },
        };
    } catch (error) {
        console.error("Failed to fetch DPS calculator data:", error);
        return { notFound: true };
    }
};

export default DpsCalculatorPage;
