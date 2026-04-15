import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
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
    // Backend returns camelCase `userId`. Accept `user_id` too for forward-compat.
    userId?: string;
    user_id?: string;
}

async function verifyUserRole(token: string): Promise<{ valid: boolean; role: string | null; userId: string | null }> {
    try {
        // Backend `/auth/verify` is a GET route that requires a Bearer token header.
        const response = await backendFetch("/auth/verify", {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });
        if (!response.ok) return { valid: false, role: null, userId: null };
        const data: VerifyResponse = await response.json();
        return { valid: data.valid, role: data.role || null, userId: data.userId ?? data.user_id ?? null };
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

    const siteToken = getToken(req);

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
        // v3 backend emits `list_type`; v2 emitted `tier_list_type`. Accept either.
        const tierListType = (tierListData.list_type || tierListData.tier_list_type || "official") as TierListType;
        const createdBy = tierListData.created_by;

        if (tierListType === "community") {
            // Community tier lists: owner can edit, tier_list_admin+ can bypass
            const isAdmin = role === "tier_list_admin" || role === "super_admin";
            if (createdBy !== userId && !isAdmin) {
                return res.status(403).json({
                    success: false,
                    error: "You don't own this tier list",
                });
            }
        } else {
            // Official tier lists: any admin role can edit (backend handles fine-grained permissions)
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
                // v3 backend returns the bare Tier object.
                const createdTier = (await createResponse.json()) as { id: string };
                if (tier.id) {
                    tierIdMap.set(tier.id, createdTier.id);
                }

                for (const placement of tier.placements) {
                    await backendFetch(`/tier-lists/${slug}/placements`, {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json",
                            Authorization: `Bearer ${siteToken}`,
                        },
                        body: JSON.stringify({
                            tier_id: createdTier.id,
                            operator_id: placement.operator_id,
                            sub_order: placement.sub_order,
                            notes: placement.notes,
                        }),
                    });
                }
            } else {
                console.error(`Failed to create tier ${tier.name}:`, await createResponse.text());
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
            // v3 backend expects a full tier payload including `display_order`.
            const tierUpdateResponse = await backendFetch(`/tier-lists/${slug}/tiers/${tier.id}`, {
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

        // v3 backend keys placements by `operator_id` (not placement UUID).
        // Deletions first, then moves, then creates — avoids duplicate-operator
        // conflicts when the same operator is being relocated.
        //
        // Operator→placement_id mapping is needed because `placementsToDelete`
        // holds placement UUIDs from the old state. Resolve each back to its
        // operator_id via the map built from `currentTiers`.
        const placementIdToOperator = new Map<string, string>();
        for (const tier of currentTiers) {
            for (const op of tier.operators || []) {
                placementIdToOperator.set(op.id, op.operator_id);
            }
        }

        for (const placementId of placementsToDelete) {
            const operatorId = placementIdToOperator.get(placementId);
            if (!operatorId) continue;
            const deleteResponse = await backendFetch(`/tier-lists/${slug}/placements/${encodeURIComponent(operatorId)}`, {
                method: "DELETE",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${siteToken}`,
                },
            });
            if (!deleteResponse.ok) {
                console.error(`Failed to delete placement for ${operatorId}:`, await deleteResponse.text());
            }
        }

        for (const { newTierId, placement } of placementsToMove) {
            const moveResponse = await backendFetch(`/tier-lists/${slug}/placements/${encodeURIComponent(placement.operator_id)}/move`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${siteToken}`,
                },
                body: JSON.stringify({
                    new_tier_id: newTierId,
                    sub_order: placement.sub_order,
                }),
            });
            if (!moveResponse.ok) {
                console.error(`Failed to move placement for ${placement.operator_id}:`, await moveResponse.text());
            }
        }

        for (const { tierId, placement } of placementsToCreate) {
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
                console.error(`Failed to create placement for ${placement.operator_id}:`, await createResponse.text());
            }
        }

        // Sub-order updates for existing placements: re-invoke `move_to` on the
        // same tier to overwrite `sub_order`. (There is no dedicated placement
        // update endpoint in v3; notes can't be edited through sync.)
        for (const { placement } of placementsToUpdate) {
            const containingTier = tiersToUpdate.find((t) => t.placements.some((p) => p.operator_id === placement.operator_id));
            if (!containingTier?.id) continue;
            const moveResponse = await backendFetch(`/tier-lists/${slug}/placements/${encodeURIComponent(placement.operator_id)}/move`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${siteToken}`,
                },
                body: JSON.stringify({
                    new_tier_id: containingTier.id,
                    sub_order: placement.sub_order,
                }),
            });
            if (!moveResponse.ok) {
                console.error(`Failed to reorder placement for ${placement.operator_id}:`, await moveResponse.text());
            }
        }

        // v3 backend has no dedicated "reorder tiers" endpoint. Re-PUT each
        // tier with its new display_order instead.
        for (const [index, tier] of tiers.entries()) {
            const resolvedId = tier.id?.startsWith("new-") ? tierIdMap.get(tier.id) : tier.id;
            if (!resolvedId) continue;
            await backendFetch(`/tier-lists/${slug}/tiers/${resolvedId}`, {
                method: "PUT",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${siteToken}`,
                },
                body: JSON.stringify({
                    name: tier.name,
                    display_order: index,
                    color: tier.color,
                    description: tier.description,
                }),
            });
        }

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
