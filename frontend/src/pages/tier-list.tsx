import type { GetServerSidePropsContext, NextPage } from "next";
import { SEO } from "~/components/seo";
import { TierListView } from "~/components/tier-list";
import { TierListIndex } from "~/components/tier-list/impl/tier-list-index";
import { backendFetch } from "~/lib/backend-fetch";
import type { Operator, OperatorFromList } from "~/types/api";
import type { TierListResponse, TierListVersionSummary } from "~/types/api/impl/tier-list";

interface TierListPreview {
    id: string;
    name: string;
    slug: string;
    description: string | null;
    is_active: boolean;
    tier_list_type: "official" | "community";
    created_at: string;
    updated_at: string;
    operatorCount: number;
    tierCount: number;
    topOperators: OperatorFromList[];
}

interface TierListDetailProps {
    mode: "detail";
    tierListData: TierListResponse;
    operatorsData: Record<string, OperatorFromList>;
    versions: TierListVersionSummary[];
}

interface TierListIndexProps {
    mode: "index";
    tierLists: TierListPreview[];
}

type TierListPageProps = TierListDetailProps | TierListIndexProps;

const TierListPage: NextPage<TierListPageProps> = (props) => {
    if (props.mode === "index") {
        return (
            <>
                <SEO description="Browse all Arknights operator tier lists and rankings. Find the best operators for your team with community-curated rankings." keywords={["tier list", "operator rankings", "best operators", "meta operators"]} path="/tier-list" title="Tier Lists - Arknights Operator Rankings" />
                <TierListIndex tierLists={props.tierLists} />
            </>
        );
    }

    return (
        <>
            <SEO
                description={props.tierListData.tier_list.description ?? "View operator rankings and tier list for Arknights."}
                keywords={["tier list", props.tierListData.tier_list.name, "operator rankings"]}
                path={`/tier-list?slug=${props.tierListData.tier_list.slug}`}
                title={`${props.tierListData.tier_list.name} - Operator Tier List`}
            />
            <TierListView operatorsData={props.operatorsData} tierListData={props.tierListData} versions={props.versions} />
        </>
    );
};

export const getServerSideProps = async (context: GetServerSidePropsContext) => {
    const tierListSlug = context.query.slug as string | undefined;

    try {
        if (!tierListSlug) {
            const tierListsResponse = await backendFetch("/tier-lists");

            if (!tierListsResponse.ok) {
                console.error("Failed to fetch tier lists");
                return {
                    props: {
                        mode: "index" as const,
                        tierLists: [],
                    },
                };
            }

            // v3: GET /tier-lists returns TierList[] directly (plain array)
            const tierListsData = (await tierListsResponse.json()) as Array<{
                id: string;
                name: string;
                slug: string;
                description: string | null;
                list_type: string;
                is_active: boolean;
                created_at: string;
                updated_at: string;
            }>;

            // v3: /static/operators returns Record<string, Operator>
            const operatorsResponse = await backendFetch("/static/operators");

            const operatorsJson = (await operatorsResponse.json()) as Record<string, Operator>;

            const operatorsMap: Record<string, OperatorFromList> = {};
            for (const op of Object.values(operatorsJson)) {
                if (op.id) {
                    operatorsMap[op.id] = op as OperatorFromList;
                }
            }

            const tierListsWithDetails: TierListPreview[] = await Promise.all(
                tierListsData.map(async (tierList) => {
                    try {
                        const detailResponse = await backendFetch(`/tier-lists/${tierList.slug}`);

                        if (!detailResponse.ok) {
                            return {
                                id: tierList.id,
                                name: tierList.name,
                                slug: tierList.slug,
                                description: tierList.description ?? null,
                                is_active: tierList.is_active,
                                tier_list_type: (tierList.list_type || "official") as "official" | "community",
                                created_at: tierList.created_at,
                                updated_at: tierList.updated_at,
                                operatorCount: 0,
                                tierCount: 0,
                                topOperators: [],
                            };
                        }

                        const detailData = await detailResponse.json();
                        const tiers = detailData.tiers || [];

                        let operatorCount = 0;
                        const topOperatorIds: string[] = [];

                        for (const tier of tiers) {
                            const tierOperators = tier.operators || [];
                            operatorCount += tierOperators.length;

                            if (topOperatorIds.length < 6) {
                                for (const op of tierOperators) {
                                    if (topOperatorIds.length < 6) {
                                        topOperatorIds.push(op.operator_id);
                                    }
                                }
                            }
                        }

                        const topOperators = topOperatorIds.map((id) => operatorsMap[id]).filter((op): op is OperatorFromList => op !== undefined);

                        return {
                            id: tierList.id,
                            name: tierList.name,
                            slug: tierList.slug,
                            description: tierList.description ?? null,
                            is_active: tierList.is_active,
                            tier_list_type: (tierList.list_type || "official") as "official" | "community",
                            created_at: tierList.created_at,
                            updated_at: tierList.updated_at,
                            operatorCount,
                            tierCount: tiers.length,
                            topOperators,
                        };
                    } catch {
                        return {
                            id: tierList.id,
                            name: tierList.name,
                            slug: tierList.slug,
                            description: tierList.description ?? null,
                            is_active: tierList.is_active,
                            tier_list_type: (tierList.list_type || "official") as "official" | "community",
                            created_at: tierList.created_at,
                            updated_at: tierList.updated_at,
                            operatorCount: 0,
                            tierCount: 0,
                            topOperators: [],
                        };
                    }
                }),
            );

            return {
                props: {
                    mode: "index" as const,
                    tierLists: tierListsWithDetails,
                },
            };
        }

        const tierListResponse = await backendFetch(`/tier-lists/${tierListSlug}`);

        if (!tierListResponse.ok) {
            return {
                notFound: true,
            };
        }

        const rawData = await tierListResponse.json();

        // Backend returns tier.operators; remap to tier.placements and coerce undefined to null for serialization
        const tierListData: TierListResponse = {
            tier_list: {
                id: rawData.id,
                name: rawData.name,
                slug: rawData.slug,
                description: rawData.description ?? null,
                is_active: rawData.is_active ?? false,
                tier_list_type: (rawData.list_type || "official") as "official" | "community",
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

        const operatorIds = new Set<string>();
        for (const tier of tierListData.tiers) {
            for (const placement of tier.placements) {
                operatorIds.add(placement.operator_id);
            }
        }

        const operatorsData: Record<string, OperatorFromList> = {};

        if (operatorIds.size > 0) {
            // v3: /static/operators returns Record<string, Operator>
            const operatorsResponse = await backendFetch("/static/operators");

            const operatorsJson = (await operatorsResponse.json()) as Record<string, Operator>;

            for (const [opId, operator] of Object.entries(operatorsJson)) {
                if (operatorIds.has(opId)) {
                    operatorsData[opId] = operator as OperatorFromList;
                }
            }
        }

        let versions: TierListVersionSummary[] = [];
        try {
            const versionsResponse = await backendFetch(`/tier-lists/${tierListSlug}/versions?limit=20`);

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
        }

        return {
            props: {
                mode: "detail" as const,
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
