import type { IncomingMessage } from "http";
import type { NextPage } from "next";
import Head from "next/head";
import OperatorsInfo from "~/components/operators/operators-info";
import { OperatorsWrapper } from "~/components/operators/operators-wrapper";
import type { Operator } from "~/types/impl/api/static/operator";

const Operators: NextPage<Props> = ({ data, id }) => {
    return (
        <>
            <Head>
                <title>{id ? `${(data as unknown as Operator).name}` : "Operators List"}</title>
                <meta name="description" content="Elevate your Arknights experience to the next level." />
                <link rel="icon" href="/favicon.ico" />
            </Head>
            <div className="container">{id ? <OperatorsInfo operator={data as unknown as Operator} /> : <OperatorsWrapper operators={data} />}</div>
        </>
    );
};

export const getServerSideProps = async ({
    req,
    query,
}: {
    req: IncomingMessage;
    query: {
        id?: string;
    };
}) => {
    const host = req.headers.host;
    const protocol = req.headers.host?.startsWith("localhost") ? "http" : "https";

    // Construct the full URL for the API endpoint
    const apiURL = `${protocol}://${host}/api/static`;

    if (!query.id) {
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
            data: Operator[];
        };

        return {
            props: {
                data: data.data,
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
            data: Operator[];
        };

        if (data.data.length === 0) {
            return {
                notFound: true,
            };
        }

        return {
            props: {
                data: data.data,
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
