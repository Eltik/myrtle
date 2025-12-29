import type { NextPage } from "next";
import Head from "next/head";
import { TierListView } from "~/components/operators/tier-list";
import { env } from "~/env";
import type { Operator } from "~/types/api";
import type { TierListResponse } from "~/types/api/impl/tier-list";
import type { OperatorFromList } from "~/types/api/operators";

interface TierListPageProps {
    tierListData: TierListResponse;
    operatorsData: Record<string, OperatorFromList>;
}

const TierListPage: NextPage<TierListPageProps> = ({ tierListData, operatorsData }) => {
    return (
        <>
            <Head>
                <title>{tierListData.tier_list.name} - Operator Tier List</title>
                <meta content={tierListData.tier_list.description ?? "View operator rankings and tier list"} name="description" />
            </Head>
            <TierListView operatorsData={operatorsData} tierListData={tierListData} />
        </>
    );
};

export const getServerSideProps = async () => {
    const backendURL = env.BACKEND_URL;

    try {
        // Fetch tier list data from backend
        // TODO: Update this endpoint to match your actual backend API
        // For now, using a placeholder slug - adjust as needed
        const tierListSlug = "main"; // or get from query params

        const tierListResponse = await fetch(`${backendURL}/tier-lists/${tierListSlug}`, {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!tierListResponse.ok) {
            return {
                notFound: true,
            };
        }

        const tierListData = (await tierListResponse.json()) as TierListResponse;

        // Get all unique operator IDs from placements
        const operatorIds = new Set<string>();
        for (const tier of tierListData.tiers) {
            for (const placement of tier.placements) {
                operatorIds.add(placement.operator_id);
            }
        }

        // Fetch operator data for all operators in the tier list
        const operatorsData: Record<string, OperatorFromList> = {};

        if (operatorIds.size > 0) {
            const operatorsBase = `${backendURL}/static/operators`;
            const params = new URLSearchParams({
                limit: "1000",
                fields: ["id", "name", "nationId", "groupId", "teamId", "position", "isSpChar", "rarity", "profession", "subProfessionId", "profile", "artists", "portrait"].join(","),
            });

            const operatorsResponse = await fetch(`${operatorsBase}?${params.toString()}`, {
                method: "GET",
                headers: {
                    "Content-Type": "application/json",
                },
            });

            const operatorsJson = (await operatorsResponse.json()) as {
                has_more: boolean;
                next_cursor: string;
                operators: Operator[];
            };

            // Create a map of operator ID to operator data
            for (const operator of operatorsJson.operators) {
                if (operator.id && operatorIds.has(operator.id)) {
                    operatorsData[operator.id] = operator as OperatorFromList;
                }
            }
        }

        return {
            props: {
                tierListData,
                operatorsData,
            },
        };
    } catch (error) {
        console.error("Error fetching tier list data:", error);
        return {
            notFound: true,
        };
    }
};

export default TierListPage;
