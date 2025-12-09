import type { NextPage } from "next";
import Head from "next/head";
import { OperatorsList } from "~/components/operators/operators-list";
import { env } from "~/env";
import type { Operator } from "~/types/api";
import type { OperatorFromList } from "~/types/api/operators";

const Operators: NextPage<Props> = ({ data }) => {
    return (
        <>
            <Head>
                <title>Operators</title>
                <meta content="Browse all Arknights operators with detailed stats, skills, and information." name="description" />
            </Head>
            <OperatorsList data={data} />
        </>
    );
};

export const getServerSideProps = async () => {
    const backendURL = env.BACKEND_URL;

    const base = `${backendURL}/static/operators`;

    const params = new URLSearchParams({
        limit: "1000",
        fields: ["id", "name", "nationId", "groupId", "teamId", "position", "isSpChar", "rarity", "profession", "subProfessionId", "profile", "artists", "portrait"].join(","),
    });

    const data = (await (
        await fetch(`${base}?${params.toString()}`, {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        })
    ).json()) as {
        has_more: boolean;
        next_cursor: string;
        operators: Operator[];
    };

    if (data.operators?.length === 0) {
        return {
            notFound: true,
        };
    }

    return {
        props: {
            data: data.operators,
        },
    };
};

export default Operators;

interface Props {
    data: OperatorFromList[];
}
