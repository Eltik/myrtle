"use client";

import { useRouter } from "next/router";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import { TierListEditor } from "~/components/admin/impl/tier-list-editor";
import { SEO } from "~/components/seo";
import { Card, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { useAuth } from "~/hooks/use-auth";
import type { OperatorFromList } from "~/types/api";
import type { TierListResponse } from "~/types/api/impl/tier-list";

export default function EditMyTierListPage() {
    const router = useRouter();
    const { slug } = router.query;
    const { user, loading: authLoading } = useAuth();

    const [tierListData, setTierListData] = useState<TierListResponse | null>(null);
    const [operatorsData, setOperatorsData] = useState<Record<string, OperatorFromList>>({});
    const [allOperators, setAllOperators] = useState<OperatorFromList[]>([]);
    const [loading, setLoading] = useState(true);
    const [operatorsLoading, setOperatorsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    // Fetch tier list data
    const fetchTierList = useCallback(async () => {
        if (!slug || typeof slug !== "string") return;

        setLoading(true);
        setError(null);
        try {
            const response = await fetch(`/api/tier-lists/${slug}?_t=${Date.now()}`, {
                cache: "no-store",
            });

            if (!response.ok) {
                const data = await response.json().catch(() => ({}));
                throw new Error(data.error || "Failed to fetch tier list");
            }

            const data = await response.json();

            // Verify this is a community tier list
            if (data.tier_list.tier_list_type !== "community") {
                throw new Error("This is not a community tier list");
            }

            // Note: Ownership is enforced by the backend when saving.
            // We can't easily check ownership here because created_by is the internal
            // user UUID, not the game UID. The backend will return an error if the
            // user tries to save a tier list they don't own.

            setTierListData(data);
        } catch (err) {
            console.error("Failed to fetch tier list:", err);
            setError(err instanceof Error ? err.message : "Failed to load tier list");
        } finally {
            setLoading(false);
        }
    }, [slug]);

    // Fetch operators data
    useEffect(() => {
        async function fetchOperators() {
            setOperatorsLoading(true);
            try {
                const response = await fetch("/api/static", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({
                        type: "operators",
                        limit: 1000,
                        fields: ["id", "name", "rarity", "profession", "subProfessionId", "portrait"],
                    }),
                });
                const json = await response.json();
                const operators = json.data || json.operators;
                if (operators && Array.isArray(operators)) {
                    setAllOperators(operators);
                    const operatorsMap: Record<string, OperatorFromList> = {};
                    for (const op of operators) {
                        if (op.id) {
                            operatorsMap[op.id] = op;
                        }
                    }
                    setOperatorsData(operatorsMap);
                }
            } catch (error) {
                console.error("Failed to fetch operators:", error);
            } finally {
                setOperatorsLoading(false);
            }
        }
        fetchOperators();
    }, []);

    // Fetch tier list when user is authenticated and slug is available
    useEffect(() => {
        if (user?.status && slug) {
            fetchTierList();
        } else if (!authLoading && !user?.status) {
            setLoading(false);
        }
    }, [user, authLoading, slug, fetchTierList]);

    const handleBack = useCallback(() => {
        router.push("/my/tier-lists");
    }, [router]);

    const handleSave = useCallback(
        async (data: TierListResponse) => {
            try {
                // Update tier list metadata
                const metaResponse = await fetch(`/api/tier-lists/${data.tier_list.slug}`, {
                    method: "PUT",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({
                        name: data.tier_list.name,
                        description: data.tier_list.description,
                        is_active: data.tier_list.is_active,
                    }),
                });

                if (!metaResponse.ok) {
                    throw new Error("Failed to update tier list metadata");
                }

                // Sync tiers and placements
                const syncPayload = {
                    tiers: data.tiers.map((tier) => ({
                        id: tier.id.startsWith("new-") ? null : tier.id,
                        name: tier.name,
                        display_order: tier.display_order,
                        color: tier.color,
                        description: tier.description,
                        placements: tier.placements.map((p) => ({
                            id: p.id.startsWith("new-") ? null : p.id,
                            operator_id: p.operator_id,
                            sub_order: p.sub_order,
                            notes: p.notes,
                        })),
                    })),
                };

                const tiersResponse = await fetch(`/api/tier-lists/${data.tier_list.slug}/sync`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify(syncPayload),
                });

                if (!tiersResponse.ok) {
                    const errorData = await tiersResponse.json().catch(() => ({}));
                    console.error("Sync failed:", errorData);
                    throw new Error("Failed to update tiers");
                }

                toast.success("Tier list saved successfully");

                // Refetch the tier list to get fresh data
                await fetchTierList();
            } catch (error) {
                console.error("Failed to save tier list:", error);
                toast.error("Failed to save tier list");
                throw error;
            }
        },
        [fetchTierList],
    );

    if (authLoading || loading) {
        return (
            <>
                <SEO description="Edit your community tier list." noIndex path={`/my/tier-lists/${slug}`} title="Edit Tier List" />
                <div className="flex min-h-[50vh] items-center justify-center">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                </div>
            </>
        );
    }

    if (!user?.status) {
        return (
            <>
                <SEO description="Edit your community tier list." noIndex path={`/my/tier-lists/${slug}`} title="Edit Tier List" />
                <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                    <Card className="max-w-md">
                        <CardHeader>
                            <CardTitle>Authentication Required</CardTitle>
                            <CardDescription>Please log in to edit your tier lists.</CardDescription>
                        </CardHeader>
                    </Card>
                </div>
            </>
        );
    }

    if (error) {
        return (
            <>
                <SEO description="Edit your community tier list." noIndex path={`/my/tier-lists/${slug}`} title="Edit Tier List" />
                <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                    <Card className="max-w-md">
                        <CardHeader>
                            <CardTitle>Error</CardTitle>
                            <CardDescription>{error}</CardDescription>
                        </CardHeader>
                    </Card>
                </div>
            </>
        );
    }

    if (!tierListData) {
        return (
            <>
                <SEO description="Edit your community tier list." noIndex path={`/my/tier-lists/${slug}`} title="Edit Tier List" />
                <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                    <Card className="max-w-md">
                        <CardHeader>
                            <CardTitle>Not Found</CardTitle>
                            <CardDescription>The tier list you&apos;re looking for could not be found.</CardDescription>
                        </CardHeader>
                    </Card>
                </div>
            </>
        );
    }

    return (
        <>
            <SEO description={`Edit ${tierListData.tier_list.name} - your community tier list.`} noIndex path={`/my/tier-lists/${slug}`} title={`Edit ${tierListData.tier_list.name}`} />
            <div className="mx-auto max-w-7xl">
                <TierListEditor allOperators={allOperators} canToggleActive={false} onBack={handleBack} onSave={handleSave} operatorsData={operatorsData} operatorsLoading={operatorsLoading} tierListData={tierListData} />
            </div>
        </>
    );
}
