import type { NextPage } from "next";
import Head from "next/head";
import OperatorsInfo from "~/components/operators/operators-info";
import { OperatorsWrapper } from "~/components/operators/operators-wrapper";
import { env } from "~/env";
import type { Operator } from "~/types/impl/api/static/operator";

const Operators: NextPage<Props> = ({ data, id }) => {
    return (
        <>
            <Head>
                <title>{id ? `${(data as unknown as Operator).name}` : "Operators List"}</title>
                <meta content="Elevate your Arknights experience to the next level." name="description" />
                <link href="/favicon.ico" rel="icon" />
            </Head>
            <div className="mx-auto md:container">{id ? <OperatorsInfo operator={data as unknown as Operator} /> : <OperatorsWrapper operators={data} />}</div>
        </>
    );
};

export const getServerSideProps = async ({
    query,
}: {
    query: {
        id?: string;
    };
}) => {
    const backendURL = env.BACKEND_URL;

    // Construct the full URL for the API endpoint
    const apiURL = `${backendURL}/static`;

    if (!query.id) {
        const data = (await (
            await fetch(apiURL, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "operators",
                    fields: ["id", "nationId", "teamId", "profession", "rarity", "name", "subProfessionId", "artists", "groupId", "skills", "handbook", "modules", "phases", "profile", "position"],
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
    } else {
        const data = (await (
            await fetch(apiURL, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "operators",
                    id: query.id,
                }),
            })
        ).json()) as {
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
                id: query.id,
            },
        };
    }
};

export default Operators;

interface Props {
    data: Operator[];
    id?: string;
}
