import type { GetServerSideProps, NextPage } from "next";
import { OperatorDetail } from "~/components/collection/operators/detail/operator-detail";
import { OperatorsList } from "~/components/collection/operators/list/operators-list";
import { SEO } from "~/components/seo";
import { backendFetch } from "~/lib/backend-fetch";
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

    try {
        // v3: /static/operators returns Record<string, Operator>
        const response = await backendFetch("/static/operators");

        if (!response.ok) {
            console.error("Failed to fetch operators:", response.status);
            return { notFound: true };
        }

        const data = (await response.json()) as Record<string, Operator>;
        const operators = Object.values(data);

        if (operators.length === 0) {
            return { notFound: true };
        }

        // If no ID provided, show the list view
        if (!id || typeof id !== "string") {
            return {
                props: {
                    mode: "list" as const,
                    operators: operators as OperatorFromList[],
                },
            };
        }

        // ID provided, show the detail view - look up by ID in the map
        const operator = data[id];

        if (!operator) {
            return { notFound: true };
        }

        return {
            props: {
                mode: "detail" as const,
                operator,
            },
        };
    } catch (error) {
        console.error("Failed to fetch operators:", error);
        return { notFound: true };
    }
};

export default OperatorsPage;
