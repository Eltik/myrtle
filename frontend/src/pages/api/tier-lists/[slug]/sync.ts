import type { NextApiRequest, NextApiResponse } from "next";
import { getSiteToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import { isAdminRole } from "~/lib/permissions";
import type { TierListType } from "~/types/api/impl/tier-list";

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

interface VerifyResponse {
    valid: boolean;
    role?: string;
    user_id?: string;
}

async function verifyUserRole(token: string): Promise<{ valid: boolean; role: string | null; userId: string | null }> {
    try {
        const response = await backendFetch("/auth/verify", {
            method: "POST",
            body: JSON.stringify({ token }),
        });
        if (!response.ok) return { valid: false, role: null, userId: null };
        const data: VerifyResponse = await response.json();
        return { valid: data.valid, role: data.role || null, userId: data.user_id || null };
    } catch {
        return { valid: false, role: null, userId: null };
    }
}

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    const { slug } = req.query;

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

    const { valid, role, userId } = await verifyUserRole(siteToken);
    if (!valid) {
        return res.status(401).json({
            success: false,
            error: "Invalid or expired session",
        });
    }

    try {
        const tierListResponse = await backendFetch(`/tier-lists/${slug}`, {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!tierListResponse.ok) {
            return res.status(tierListResponse.status).json({
                success: false,
                error: "Tier list not found",
            });
        }

        const tierListData = await tierListResponse.json();
        const tierListType = (tierListData.tier_list_type || "official") as TierListType;
        const createdBy = tierListData.created_by;

        if (tierListType === "community") {
            if (createdBy !== userId) {
                return res.status(403).json({
                    success: false,
                    error: "You don't own this tier list",
                });
            }
        } else {
            if (!isAdminRole(role ?? undefined)) {
                return res.status(403).json({
                    success: false,
                    error: "You don't have permission to edit tier lists",
                });
            }
        }
        const { tiers } = req.body as SyncRequest;

        if (!tiers || !Array.isArray(tiers)) {
            return res.status(400).json({
                success: false,
                error: "Tiers array is required",
            });
        }

        const currentResponse = await backendFetch(`/tier-lists/${slug}`, {
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

        for (const currentTier of currentTiers) {
            if (!tierIdsToKeep.has(currentTier.id)) {
                await backendFetch(`/tier-lists/${slug}/tiers/${currentTier.id}`, {
                    method: "DELETE",
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: `Bearer ${siteToken}`,
                    },
                });
            }
        }

        const tierIdMap = new Map<string, string>();
        for (const tier of tiersToCreate) {
            const createResponse = await backendFetch(`/tier-lists/${slug}/tiers`, {
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

                for (const placement of tier.placements) {
                    await backendFetch(`/tier-lists/${slug}/placements`, {
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

        const currentPlacementsByOperator = new Map<string, { id: string; tier_id: string }>();
        for (const tier of currentTiers) {
            for (const op of tier.operators || []) {
                currentPlacementsByOperator.set(op.operator_id, { id: op.id, tier_id: tier.id });
            }
        }

        const placementsToCreate: Array<{ tierId: string; placement: PlacementData }> = [];
        const placementsToMove: Array<{ placementId: string; newTierId: string; placement: PlacementData }> = [];
        const placementsToUpdate: Array<{ placementId: string; placement: PlacementData }> = [];
        const placementIdsHandled = new Set<string>();

        for (const tier of tiersToUpdate) {
            const tierUpdateResponse = await backendFetch(`/tier-lists/${slug}/tiers/${tier.id}`, {
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
                    const existingPlacement = currentPlacementsByOperator.get(placement.operator_id);
                    if (existingPlacement && existingPlacement.tier_id !== tier.id) {
                        console.log(`Moving operator ${placement.operator_id} from tier ${existingPlacement.tier_id} to ${tier.id}`);
                        placementsToMove.push({
                            placementId: existingPlacement.id,
                            newTierId: tier.id ?? "",
                            placement,
                        });
                        placementIdsHandled.add(existingPlacement.id);
                    } else if (!existingPlacement) {
                        placementsToCreate.push({ tierId: tier.id ?? "", placement });
                    }
                } else {
                    placementIdsHandled.add(placement.id);
                    placementsToUpdate.push({ placementId: placement.id, placement });
                }
            }
        }

        const placementsToDelete: string[] = [];
        for (const tier of currentTiers) {
            for (const op of tier.operators || []) {
                if (!placementIdsHandled.has(op.id)) {
                    placementsToDelete.push(op.id);
                }
            }
        }

        // Moves must happen before creates to avoid duplicate operator_id conflicts
        for (const { placementId, newTierId, placement } of placementsToMove) {
            console.log(`Moving placement ${placementId} to tier ${newTierId}`);
            const moveResponse = await backendFetch(`/tier-lists/${slug}/placements/${placementId}/move`, {
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

        for (const { tierId, placement } of placementsToCreate) {
            console.log(`Creating placement for operator ${placement.operator_id} in tier ${tierId}`);
            const createResponse = await backendFetch(`/tier-lists/${slug}/placements`, {
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

        for (const { placementId, placement } of placementsToUpdate) {
            const updateResponse = await backendFetch(`/tier-lists/${slug}/placements/${placementId}`, {
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

        for (const placementId of placementsToDelete) {
            console.log(`Deleting placement ${placementId}`);
            const deleteResponse = await backendFetch(`/tier-lists/${slug}/placements/${placementId}`, {
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

        const reorderPayload = tiers.map((tier, index) => ({
            tier_id: tier.id?.startsWith("new-") ? tierIdMap.get(tier.id) || tier.id : tier.id,
            display_order: index,
        }));

        await backendFetch(`/tier-lists/${slug}/tiers/reorder`, {
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
