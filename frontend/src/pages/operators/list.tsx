import type { NextPage } from "next";
import { OperatorsList } from "~/components/operators/list/operators-list";
import { SEO } from "~/components/seo";
import { env } from "~/env";
import type { Operator, OperatorFromList } from "~/types/api";

const Operators: NextPage<Props> = ({ data }) => {
    return (
        <>
            <SEO description="Browse all Arknights operators with detailed stats, skills, talents, and information. Filter by class, rarity, and more." keywords={["Arknights operators", "operator list", "operator database", "operator stats"]} path="/operators/list" title="Operators" />
            <OperatorsList data={data} />
        </>
    );
};

export const getServerSideProps = async () => {
    const backendURL = env.BACKEND_URL;

    const base = `${backendURL}/static/operators`;

    const params = new URLSearchParams({
        limit: "1000",
        fields: ["id", "name", "nationId", "groupId", "teamId", "position", "isSpChar", "rarity", "profession", "subProfessionId", "profile", "artists", "portrait", "phases", "handbook"].join(","),
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
