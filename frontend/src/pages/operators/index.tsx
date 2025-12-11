import type { GetServerSideProps, NextPage } from "next";
import Head from "next/head";
import { OperatorDetail } from "~/components/operators/detail/operator-detail";
import { env } from "~/env";
import type { Operator } from "~/types/api";

interface Props {
    operator: Operator;
}

const OperatorPage: NextPage<Props> = ({ operator }) => {
    return (
        <>
            <Head>
                <title>{operator.name} - Operator Details</title>
                <meta content={`View detailed information about ${operator.name} including stats, skills, talents, skins, and voice lines.`} name="description" />
            </Head>
            <OperatorDetail operator={operator} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async (context) => {
    const { id } = context.query;

    if (!id) {
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
