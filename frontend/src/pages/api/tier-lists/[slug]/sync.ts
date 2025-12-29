import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "~/env";
import { getSiteToken } from "~/lib/auth";

interface TierData {
    id: string | null;
    name: string;
    display_order: number;
    color: string | null;
    description: string | null;
    placements: PlacementData[];
}

interface PlacementData {
    id: string | null;
    operator_id: string;
    sub_order: number;
    notes: string | null;
}

interface SyncRequest {
    tiers: TierData[];
}

interface ApiSuccessResponse {
    success: true;
    message: string;
}

interface ApiErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = ApiSuccessResponse | ApiErrorResponse;

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    const { slug } = req.query;
    const backendUrl = env.BACKEND_URL;

    if (!slug || typeof slug !== "string") {
        return res.status(400).json({
            success: false,
            error: "Slug is required",
        });
    }

    if (req.method !== "POST") {
        res.setHeader("Allow", ["POST"]);
        return res.status(405).json({
            success: false,
            error: `Method ${req.method} not allowed`,
        });
    }

    const siteToken = getSiteToken(req);

    if (!siteToken) {
        return res.status(401).json({
            success: false,
            error: "Not authenticated",
        });
    }

    try {
        const { tiers } = req.body as SyncRequest;

        if (!tiers || !Array.isArray(tiers)) {
            return res.status(400).json({
                success: false,
                error: "Tiers array is required",
            });
        }

        // First, get the current state of the tier list
        const currentResponse = await fetch(`${backendUrl}/tier-lists/${slug}`, {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!currentResponse.ok) {
            return res.status(currentResponse.status).json({
                success: false,
                error: "Failed to fetch current tier list state",
            });
        }

        const currentData = await currentResponse.json();
        const currentTiers = currentData.tiers || [];

        // Track what needs to be created, updated, or deleted
        const tiersToCreate: TierData[] = [];
        const tiersToUpdate: TierData[] = [];
        const tierIdsToKeep = new Set<string>();

        for (const tier of tiers) {
            if (!tier.id || tier.id.startsWith("new-")) {
                tiersToCreate.push(tier);
            } else {
                tiersToUpdate.push(tier);
                tierIdsToKeep.add(tier.id);
            }
        }

        // Delete tiers that are no longer present
        for (const currentTier of currentTiers) {
            if (!tierIdsToKeep.has(currentTier.id)) {
                await fetch(`${backendUrl}/tier-lists/${slug}/tiers/${currentTier.id}`, {
                    method: "DELETE",
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: `Bearer ${siteToken}`,
                    },
                });
            }
        }

        // Create new tiers
        const tierIdMap = new Map<string, string>(); // old temp id -> new real id
        for (const tier of tiersToCreate) {
            const createResponse = await fetch(`${backendUrl}/tier-lists/${slug}/tiers`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${siteToken}`,
                },
                body: JSON.stringify({
                    name: tier.name,
                    display_order: tier.display_order,
                    color: tier.color,
                    description: tier.description,
                }),
            });

            if (createResponse.ok) {
                const createData = await createResponse.json();
                if (tier.id) {
                    tierIdMap.set(tier.id, createData.tier.id);
                }

                // Add placements to the new tier
                for (const placement of tier.placements) {
                    await fetch(`${backendUrl}/tier-lists/${slug}/placements`, {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json",
                            Authorization: `Bearer ${siteToken}`,
                        },
                        body: JSON.stringify({
                            tier_id: createData.tier.id,
                            operator_id: placement.operator_id,
                            sub_order: placement.sub_order,
                            notes: placement.notes,
                        }),
                    });
                }
            }
        }

        // Update existing tiers
        for (const tier of tiersToUpdate) {
            // Update tier metadata
            await fetch(`${backendUrl}/tier-lists/${slug}/tiers/${tier.id}`, {
                method: "PUT",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${siteToken}`,
                },
                body: JSON.stringify({
                    name: tier.name,
                    display_order: tier.display_order,
                    color: tier.color,
                    description: tier.description,
                }),
            });

            // Get current placements for this tier
            const currentTier = currentTiers.find((t: { id: string }) => t.id === tier.id);
            const currentPlacements = currentTier?.operators || [];

            // Track which placements to keep
            const placementIdsToKeep = new Set<string>();

            for (const placement of tier.placements) {
                if (!placement.id || placement.id.startsWith("new-")) {
                    // Create new placement
                    await fetch(`${backendUrl}/tier-lists/${slug}/placements`, {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json",
                            Authorization: `Bearer ${siteToken}`,
                        },
                        body: JSON.stringify({
                            tier_id: tier.id,
                            operator_id: placement.operator_id,
                            sub_order: placement.sub_order,
                            notes: placement.notes,
                        }),
                    });
                } else {
                    placementIdsToKeep.add(placement.id);
                    // Update existing placement (sub_order, notes)
                    await fetch(`${backendUrl}/tier-lists/${slug}/placements/${placement.id}`, {
                        method: "PUT",
                        headers: {
                            "Content-Type": "application/json",
                            Authorization: `Bearer ${siteToken}`,
                        },
                        body: JSON.stringify({
                            sub_order: placement.sub_order,
                            notes: placement.notes,
                        }),
                    });
                }
            }

            // Delete placements that are no longer present
            for (const currentPlacement of currentPlacements) {
                if (!placementIdsToKeep.has(currentPlacement.id)) {
                    await fetch(`${backendUrl}/tier-lists/${slug}/placements/${currentPlacement.id}`, {
                        method: "DELETE",
                        headers: {
                            "Content-Type": "application/json",
                            Authorization: `Bearer ${siteToken}`,
                        },
                    });
                }
            }
        }

        // Reorder all tiers to ensure correct display_order
        const reorderPayload = tiers.map((tier, index) => ({
            tier_id: tier.id?.startsWith("new-") ? tierIdMap.get(tier.id) || tier.id : tier.id,
            display_order: index,
        }));

        await fetch(`${backendUrl}/tier-lists/${slug}/tiers/reorder`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                Authorization: `Bearer ${siteToken}`,
            },
            body: JSON.stringify({ order: reorderPayload }),
        });

        return res.status(200).json({
            success: true,
            message: "Tier list synced successfully",
        });
    } catch (error) {
        console.error("Tier list sync error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
