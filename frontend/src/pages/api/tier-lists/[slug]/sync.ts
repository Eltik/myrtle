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
    const backendURL = env.BACKEND_URL;

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
        const currentResponse = await fetch(`${backendURL}/tier-lists/${slug}`, {
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
                await fetch(`${backendURL}/tier-lists/${slug}/tiers/${currentTier.id}`, {
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
            const createResponse = await fetch(`${backendURL}/tier-lists/${slug}/tiers`, {
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
                    await fetch(`${backendURL}/tier-lists/${slug}/placements`, {
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

        // Build a map of all current placements by operator_id to detect cross-tier moves
        const currentPlacementsByOperator = new Map<string, { id: string; tier_id: string }>();
        for (const tier of currentTiers) {
            for (const op of tier.operators || []) {
                currentPlacementsByOperator.set(op.operator_id, { id: op.id, tier_id: tier.id });
            }
        }

        // Collect placement operations
        const placementsToCreate: Array<{ tierId: string; placement: PlacementData }> = [];
        const placementsToMove: Array<{ placementId: string; newTierId: string; placement: PlacementData }> = [];
        const placementsToUpdate: Array<{ placementId: string; placement: PlacementData }> = [];
        const placementIdsHandled = new Set<string>(); // Track placements we're keeping or moving

        // Update existing tiers and collect placement operations
        for (const tier of tiersToUpdate) {
            // Update tier metadata (excluding display_order - handled by reorder endpoint)
            const tierUpdateResponse = await fetch(`${backendURL}/tier-lists/${slug}/tiers/${tier.id}`, {
                method: "PUT",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${siteToken}`,
                },
                body: JSON.stringify({
                    name: tier.name,
                    color: tier.color,
                    description: tier.description,
                }),
            });

            if (!tierUpdateResponse.ok) {
                console.error(`Failed to update tier ${tier.id}:`, await tierUpdateResponse.text());
            }

            for (const placement of tier.placements) {
                if (!placement.id || placement.id.startsWith("new-")) {
                    // Check if this operator already exists elsewhere in the tier list (cross-tier move)
                    const existingPlacement = currentPlacementsByOperator.get(placement.operator_id);
                    if (existingPlacement && existingPlacement.tier_id !== tier.id) {
                        // This is a cross-tier move - use move endpoint
                        console.log(`Moving operator ${placement.operator_id} from tier ${existingPlacement.tier_id} to ${tier.id}`);
                        placementsToMove.push({
                            placementId: existingPlacement.id,
                            newTierId: tier.id ?? "",
                            placement,
                        });
                        placementIdsHandled.add(existingPlacement.id);
                    } else if (!existingPlacement) {
                        // Genuinely new placement
                        placementsToCreate.push({ tierId: tier.id ?? "", placement });
                    }
                    // If existingPlacement.tier_id === tier.id, operator is already in this tier, skip
                } else {
                    placementIdsHandled.add(placement.id);
                    // Queue for update
                    placementsToUpdate.push({ placementId: placement.id, placement });
                }
            }
        }

        // Find placements to delete (not handled and not being moved)
        const placementsToDelete: string[] = [];
        for (const tier of currentTiers) {
            for (const op of tier.operators || []) {
                if (!placementIdsHandled.has(op.id)) {
                    placementsToDelete.push(op.id);
                }
            }
        }

        // Execute placement operations

        // 1. Move placements to different tiers (must happen first)
        for (const { placementId, newTierId, placement } of placementsToMove) {
            console.log(`Moving placement ${placementId} to tier ${newTierId}`);
            const moveResponse = await fetch(`${backendURL}/tier-lists/${slug}/placements/${placementId}/move`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${siteToken}`,
                },
                body: JSON.stringify({
                    new_tier_id: newTierId,
                    new_sub_order: placement.sub_order,
                }),
            });

            if (!moveResponse.ok) {
                const errorText = await moveResponse.text();
                console.error(`Failed to move placement ${placementId}:`, errorText);
            }
        }

        // 2. Create genuinely new placements
        for (const { tierId, placement } of placementsToCreate) {
            console.log(`Creating placement for operator ${placement.operator_id} in tier ${tierId}`);
            const createResponse = await fetch(`${backendURL}/tier-lists/${slug}/placements`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${siteToken}`,
                },
                body: JSON.stringify({
                    tier_id: tierId,
                    operator_id: placement.operator_id,
                    sub_order: placement.sub_order,
                    notes: placement.notes,
                }),
            });

            if (!createResponse.ok) {
                const errorText = await createResponse.text();
                console.error(`Failed to create placement for ${placement.operator_id}:`, errorText);
            }
        }

        // 3. Update existing placements (sub_order, notes)
        for (const { placementId, placement } of placementsToUpdate) {
            const updateResponse = await fetch(`${backendURL}/tier-lists/${slug}/placements/${placementId}`, {
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

            if (!updateResponse.ok) {
                console.error(`Failed to update placement ${placementId}:`, await updateResponse.text());
            }
        }

        // 4. Delete removed placements
        for (const placementId of placementsToDelete) {
            console.log(`Deleting placement ${placementId}`);
            const deleteResponse = await fetch(`${backendURL}/tier-lists/${slug}/placements/${placementId}`, {
                method: "DELETE",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${siteToken}`,
                },
            });

            if (!deleteResponse.ok) {
                console.error(`Failed to delete placement ${placementId}:`, await deleteResponse.text());
            }
        }

        // Reorder all tiers to ensure correct display_order
        const reorderPayload = tiers.map((tier, index) => ({
            tier_id: tier.id?.startsWith("new-") ? tierIdMap.get(tier.id) || tier.id : tier.id,
            display_order: index,
        }));

        await fetch(`${backendURL}/tier-lists/${slug}/tiers/reorder`, {
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
