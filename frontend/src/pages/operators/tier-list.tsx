import type { GetServerSidePropsContext, NextPage } from "next";
import { TierListView } from "~/components/operators/tier-list";
import { TierListIndex } from "~/components/operators/tier-list/impl/tier-list-index";
import { SEO } from "~/components/seo";
import { env } from "~/env";
import type { Operator } from "~/types/api";
import type { TierListResponse, TierListVersionSummary } from "~/types/api/impl/tier-list";
import type { OperatorFromList } from "~/types/api/operators";

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
                <SEO
                    description="Browse all Arknights operator tier lists and rankings. Find the best operators for your team with community-curated rankings."
                    keywords={["tier list", "operator rankings", "best operators", "meta operators"]}
                    path="/operators/tier-list"
                    title="Tier Lists - Arknights Operator Rankings"
                />
                <TierListIndex tierLists={props.tierLists} />
            </>
        );
    }

    return (
        <>
            <SEO
                description={props.tierListData.tier_list.description ?? "View operator rankings and tier list for Arknights."}
                keywords={["tier list", props.tierListData.tier_list.name, "operator rankings"]}
                path={`/operators/tier-list?slug=${props.tierListData.tier_list.slug}`}
                title={`${props.tierListData.tier_list.name} - Operator Tier List`}
            />
            <TierListView operatorsData={props.operatorsData} tierListData={props.tierListData} versions={props.versions} />
        </>
    );
};

export const getServerSideProps = async (context: GetServerSidePropsContext) => {
    const backendURL = env.BACKEND_URL;
    const tierListSlug = context.query.slug as string | undefined;

    try {
        // If no slug provided, show the index page with all tier lists
        if (!tierListSlug) {
            // Fetch all tier lists
            const tierListsResponse = await fetch(`${backendURL}/tier-lists`, {
                method: "GET",
                headers: {
                    "Content-Type": "application/json",
                },
            });

            if (!tierListsResponse.ok) {
                console.error("Failed to fetch tier lists");
                return {
                    props: {
                        mode: "index" as const,
                        tierLists: [],
                    },
                };
            }

            const tierListsData = (await tierListsResponse.json()) as {
                tier_lists: Array<{
                    id: string;
                    name: string;
                    slug: string;
                    description: string | null;
                    is_active: boolean;
                    tier_list_type: "official" | "community";
                    created_at: string;
                    updated_at: string;
                }>;
            };

            // Fetch operator data for previews
            const operatorsBase = `${backendURL}/static/operators`;
            const operatorParams = new URLSearchParams({
                limit: "1000",
                fields: ["id", "name", "portrait", "rarity"].join(","),
            });

            const operatorsResponse = await fetch(`${operatorsBase}?${operatorParams.toString()}`, {
                method: "GET",
                headers: {
                    "Content-Type": "application/json",
                },
            });

            const operatorsJson = (await operatorsResponse.json()) as {
                operators: Operator[];
            };

            const operatorsMap: Record<string, OperatorFromList> = {};
            for (const op of operatorsJson.operators) {
                if (op.id) {
                    operatorsMap[op.id] = op as OperatorFromList;
                }
            }

            // Fetch details for each tier list to get operator counts and top operators
            const tierListsWithDetails: TierListPreview[] = await Promise.all(
                tierListsData.tier_lists.map(async (tierList) => {
                    try {
                        const detailResponse = await fetch(`${backendURL}/tier-lists/${tierList.slug}`, {
                            method: "GET",
                            headers: {
                                "Content-Type": "application/json",
                            },
                        });

                        if (!detailResponse.ok) {
                            return {
                                id: tierList.id,
                                name: tierList.name,
                                slug: tierList.slug,
                                description: tierList.description ?? null,
                                is_active: tierList.is_active,
                                tier_list_type: tierList.tier_list_type || "official",
                                created_at: tierList.created_at,
                                updated_at: tierList.updated_at,
                                operatorCount: 0,
                                tierCount: 0,
                                topOperators: [],
                            };
                        }

                        const detailData = await detailResponse.json();
                        const tiers = detailData.tiers || [];

                        // Count operators and tiers
                        let operatorCount = 0;
                        const topOperatorIds: string[] = [];

                        for (const tier of tiers) {
                            const tierOperators = tier.operators || [];
                            operatorCount += tierOperators.length;

                            // Get top operators from highest tiers (first few tiers by display_order)
                            if (topOperatorIds.length < 6) {
                                for (const op of tierOperators) {
                                    if (topOperatorIds.length < 6) {
                                        topOperatorIds.push(op.operator_id);
                                    }
                                }
                            }
                        }

                        // Get operator data for top operators
                        const topOperators = topOperatorIds.map((id) => operatorsMap[id]).filter((op): op is OperatorFromList => op !== undefined);

                        return {
                            id: tierList.id,
                            name: tierList.name,
                            slug: tierList.slug,
                            description: tierList.description ?? null,
                            is_active: tierList.is_active,
                            tier_list_type: tierList.tier_list_type || "official",
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
                            tier_list_type: tierList.tier_list_type || "official",
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

        // Slug provided - show the specific tier list detail view
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
                tier_list_type: rawData.tier_list_type || "official",
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
