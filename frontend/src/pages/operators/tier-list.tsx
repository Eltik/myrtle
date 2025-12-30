import type { GetServerSidePropsContext, NextPage } from "next";
import Head from "next/head";
import { TierListView } from "~/components/operators/tier-list";
import { env } from "~/env";
import type { Operator } from "~/types/api";
import type { TierListResponse, TierListVersionSummary } from "~/types/api/impl/tier-list";
import type { OperatorFromList } from "~/types/api/operators";

interface TierListPageProps {
    tierListData: TierListResponse;
    operatorsData: Record<string, OperatorFromList>;
    versions: TierListVersionSummary[];
}

const TierListPage: NextPage<TierListPageProps> = ({ tierListData, operatorsData, versions }) => {
    return (
        <>
            <Head>
                <title>{tierListData.tier_list.name} - Operator Tier List</title>
                <meta content={tierListData.tier_list.description ?? "View operator rankings and tier list"} name="description" />
            </Head>
            <TierListView operatorsData={operatorsData} tierListData={tierListData} versions={versions} />
        </>
    );
};

export const getServerSideProps = async (context: GetServerSidePropsContext) => {
    const backendURL = env.BACKEND_URL;

    try {
        // Get slug from query params, default to "main" if not provided
        const tierListSlug = (context.query.slug as string) || "main";

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

        const rawData = await tierListResponse.json();

        // Transform backend response to match frontend TierListResponse format
        // Backend returns tier.operators, frontend expects tier.placements
        // Note: Convert undefined to null for Next.js serialization
        const tierListData: TierListResponse = {
            tier_list: {
                id: rawData.id,
                name: rawData.name,
                slug: rawData.slug,
                description: rawData.description ?? null,
                is_active: rawData.is_active ?? false,
                created_by: rawData.created_by ?? null,
                created_at: rawData.created_at ?? null,
                updated_at: rawData.updated_at ?? null,
            },
            tiers: (rawData.tiers || []).map((tier: { id: string; name: string; display_order: number; color: string | null; description: string | null; operators?: Array<{ id: string; operator_id: string; sub_order: number; notes: string | null }> }) => ({
                id: tier.id,
                tier_list_id: rawData.id,
                name: tier.name,
                display_order: tier.display_order,
                color: tier.color ?? null,
                description: tier.description ?? null,
                placements: (tier.operators || []).map((op) => ({
                    id: op.id,
                    tier_id: tier.id,
                    operator_id: op.operator_id,
                    sub_order: op.sub_order,
                    notes: op.notes ?? null,
                    created_at: rawData.created_at ?? null,
                    updated_at: rawData.updated_at ?? null,
                })),
            })),
        };

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

        // Fetch versions for changelog (don't fail if this errors)
        let versions: TierListVersionSummary[] = [];
        try {
            const versionsResponse = await fetch(`${backendURL}/tier-lists/${tierListSlug}/versions?limit=20`, {
                method: "GET",
                headers: {
                    "Content-Type": "application/json",
                },
            });

            if (versionsResponse.ok) {
                const versionsData = (await versionsResponse.json()) as { versions: TierListVersionSummary[] };
                versions = versionsData.versions.map((v) => ({
                    ...v,
                    change_summary: v.change_summary ?? null,
                    published_by: v.published_by ?? null,
                }));
            }
        } catch (versionsError) {
            console.error("Error fetching versions:", versionsError);
            // Continue without versions
        }

        return {
            props: {
                tierListData,
                operatorsData,
                versions,
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
