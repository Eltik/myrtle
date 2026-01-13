import type { GetServerSideProps, NextPage } from "next";
import { OperatorDetail } from "~/components/operators/detail/operator-detail";
import { SEO } from "~/components/seo";
import { env } from "~/env.js";
import type { Operator } from "~/types/api";

interface Props {
    operator: Operator;
}

const OperatorPage: NextPage<Props> = ({ operator }) => {
    return (
        <>
            <SEO description={`View detailed information about ${operator.name} including stats, skills, talents, skins, and voice lines.`} keywords={[operator.name, "Arknights operator", "operator stats", "skills", "talents"]} path={`/operators/${operator.id}`} title={`${operator.name} - Operator Details`} />
            <OperatorDetail operator={operator} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async (context) => {
    const { id } = context.query;

    if (!id || typeof id !== "string") {
        return { notFound: true };
    }

    const backendURL = env.BACKEND_URL;
    const base = `${backendURL}/static/operators/${id}`;

    try {
        const response = await fetch(base, {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!response.ok) {
            return { notFound: true };
        }

        const data = (await response.json()) as { operator: Operator };

        if (!data.operator) {
            return { notFound: true };
        }

        return {
            props: {
                operator: data.operator,
            },
        };
    } catch (error) {
        console.error("Failed to fetch operator:", error);
        return { notFound: true };
    }
};

export default OperatorPage;
