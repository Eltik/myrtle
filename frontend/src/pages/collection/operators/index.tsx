import type { GetServerSideProps, NextPage } from "next";
import { OperatorDetail } from "~/components/collection/operators/detail/operator-detail";
import { OperatorsList } from "~/components/collection/operators/list/operators-list";
import { SEO } from "~/components/seo";
import { env } from "~/env.js";
import type { Operator, OperatorFromList } from "~/types/api";

interface ListProps {
    mode: "list";
    operators: OperatorFromList[];
}

interface DetailProps {
    mode: "detail";
    operator: Operator;
}

type Props = ListProps | DetailProps;

const OperatorsPage: NextPage<Props> = (props) => {
    if (props.mode === "list") {
        return (
            <>
                <SEO description="Browse all Arknights operators with detailed stats, skills, talents, and information. Filter by class, rarity, and more." keywords={["Arknights operators", "operator list", "operator database", "operator stats"]} path="/collection/operators" title="Operators" />
                <OperatorsList data={props.operators} />
            </>
        );
    }

    return (
        <>
            <SEO
                description={`View detailed information about ${props.operator.name} including stats, skills, talents, skins, and voice lines.`}
                keywords={[props.operator.name, "Arknights operator", "operator stats", "skills", "talents"]}
                path={`/collection/operators/${props.operator.id}`}
                title={`${props.operator.name} - Operator Details`}
            />
            <OperatorDetail operator={props.operator} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async (context) => {
    const { id } = context.query;
    const backendURL = env.BACKEND_URL;

    // If no ID provided, show the list view
    if (!id || typeof id !== "string") {
        const base = `${backendURL}/static/operators`;
        const params = new URLSearchParams({
            limit: "1000",
            fields: ["id", "name", "nationId", "groupId", "teamId", "position", "isSpChar", "rarity", "profession", "subProfessionId", "profile", "artists", "portrait", "phases"].join(","),
        });

        try {
            const response = await fetch(`${base}?${params.toString()}`, {
                method: "GET",
                headers: {
                    "Content-Type": "application/json",
                },
            });

            const data = (await response.json()) as {
                has_more: boolean;
                next_cursor: string;
                operators: Operator[];
            };

            if (data.operators?.length === 0) {
                return { notFound: true };
            }

            return {
                props: {
                    mode: "list" as const,
                    operators: data.operators as OperatorFromList[],
                },
            };
        } catch (error) {
            console.error("Failed to fetch operators:", error);
            return { notFound: true };
        }
    }

    // ID provided, show the detail view
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
                mode: "detail" as const,
                operator: data.operator,
            },
        };
    } catch (error) {
        console.error("Failed to fetch operator:", error);
        return { notFound: true };
    }
};

export default OperatorsPage;
