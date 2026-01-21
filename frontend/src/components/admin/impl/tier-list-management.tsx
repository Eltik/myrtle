"use client";

import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import type { AdminRole } from "~/lib/permissions";
import { canCreateTierList, canDeleteTierList, canToggleTierListActive } from "~/lib/permissions";
import type { TierListResponse } from "~/types/api/impl/tier-list";
import type { OperatorFromList } from "~/types/api/operators";
import type { TierListSummary } from "~/types/frontend/admin";
import { TransitionPanel } from "../../ui/motion-primitives/transition-panel";
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from "../../ui/shadcn/alert-dialog";
import { CreateTierListDialog } from "./create-tier-list-dialog";
import { PublishVersionDialog } from "./publish-version-dialog";
import { TierListEditor } from "./tier-list-editor";
import { TierListsTable } from "./tier-lists-table";

interface TierListManagementProps {
    tierLists: TierListSummary[];
    loading?: boolean;
    onRefresh?: () => void;
    role: AdminRole;
}

type ViewMode = "list" | "editor";

export function TierListManagement({ tierLists, loading = false, onRefresh, role }: TierListManagementProps) {
    const canCreate = canCreateTierList(role);
    const canDelete = canDeleteTierList(role);
    const canToggleActive = canToggleTierListActive(role);
    const [viewMode, setViewMode] = useState<ViewMode>("list");
    const [, setSelectedTierList] = useState<TierListSummary | null>(null);
    const [tierListData, setTierListData] = useState<TierListResponse | null>(null);
    const [operatorsData, setOperatorsData] = useState<Record<string, OperatorFromList>>({});
    const [allOperators, setAllOperators] = useState<OperatorFromList[]>([]);
    const [editorLoading, setEditorLoading] = useState(false);
    const [createDialogOpen, setCreateDialogOpen] = useState(false);
    const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
    const [publishDialogOpen, setPublishDialogOpen] = useState(false);
    const [tierListToDelete, setTierListToDelete] = useState<TierListSummary | null>(null);
    const [deleting, setDeleting] = useState(false);
    const [operatorsLoading, setOperatorsLoading] = useState(true);

    // Fetch all operators once on mount
    useEffect(() => {
        async function fetchOperators() {
            setOperatorsLoading(true);
            try {
                // Fetch all operators with required fields for tier list editor
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
                // Handle both { data: operators } and { operators } response formats
                const operators = json.data || json.operators;
                if (operators && Array.isArray(operators)) {
                    console.log(`Loaded ${operators.length} operators for tier list editor`);
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

    const fetchTierListData = useCallback(async (slug: string) => {
        setEditorLoading(true);
        try {
            // Add cache-busting timestamp for admin fetches
            const response = await fetch(`/api/tier-lists/${slug}?_t=${Date.now()}`, {
                cache: "no-store",
            });
            if (!response.ok) {
                throw new Error("Failed to fetch tier list");
            }
            const data = await response.json();
            setTierListData(data);
        } catch (error) {
            console.error("Failed to fetch tier list:", error);
            toast.error("Failed to load tier list");
        } finally {
            setEditorLoading(false);
        }
    }, []);

    const handleEdit = useCallback(
        async (tierList: TierListSummary) => {
            setSelectedTierList(tierList);
            await fetchTierListData(tierList.slug);
            setViewMode("editor");
        },
        [fetchTierListData],
    );

    const handleView = useCallback((tierList: TierListSummary) => {
        window.open(`/operators/tier-list?slug=${tierList.slug}`, "_blank");
    }, []);

    const handleDelete = useCallback((tierList: TierListSummary) => {
        setTierListToDelete(tierList);
        setDeleteDialogOpen(true);
    }, []);

    const confirmDelete = useCallback(async () => {
        if (!tierListToDelete) return;

        setDeleting(true);
        try {
            const response = await fetch(`/api/tier-lists/${tierListToDelete.slug}`, {
                method: "DELETE",
            });

            if (!response.ok) {
                throw new Error("Failed to delete tier list");
            }

            toast.success(`Deleted tier list: ${tierListToDelete.name}`);
            onRefresh?.();
        } catch (error) {
            console.error("Failed to delete tier list:", error);
            toast.error("Failed to delete tier list");
        } finally {
            setDeleting(false);
            setDeleteDialogOpen(false);
            setTierListToDelete(null);
        }
    }, [tierListToDelete, onRefresh]);

    const handleCreate = useCallback(
        async (data: { name: string; slug: string; description: string; isActive: boolean }) => {
            const response = await fetch("/api/tier-lists", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    name: data.name,
                    slug: data.slug,
                    description: data.description || null,
                    is_active: data.isActive,
                }),
            });

            if (!response.ok) {
                const errorData = await response.json().catch(() => ({}));
                throw new Error(errorData.error || "Failed to create tier list");
            }

            toast.success(`Created tier list: ${data.name}`);
            onRefresh?.();
        },
        [onRefresh],
    );

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

                // Update tiers and placements
                // For simplicity, we'll send the full tier structure
                // The backend should handle creating/updating/deleting as needed
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

                console.log("Syncing tier list with payload:", JSON.stringify(syncPayload, null, 2));

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

                // Refetch the tier list to get fresh data from backend and verify save worked
                await fetchTierListData(data.tier_list.slug);

                onRefresh?.();
            } catch (error) {
                console.error("Failed to save tier list:", error);
                toast.error("Failed to save tier list");
                throw error;
            }
        },
        [onRefresh, fetchTierListData],
    );

    const handleBack = useCallback(() => {
        setViewMode("list");
        setSelectedTierList(null);
        setTierListData(null);
    }, []);

    const handleOpenPublishDialog = useCallback(() => {
        setPublishDialogOpen(true);
    }, []);

    const handlePublish = useCallback(
        async (data: { changelog: string; change_summary: string | null }) => {
            if (!tierListData) return;

            const response = await fetch(`/api/tier-lists/${tierListData.tier_list.slug}/publish`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(data),
            });

            if (!response.ok) {
                const errorData = await response.json().catch(() => ({}));
                throw new Error(errorData.error || "Failed to publish version");
            }

            const result = await response.json();
            toast.success(`Published version ${result.version.version} successfully`);
            onRefresh?.();
        },
        [tierListData, onRefresh],
    );

    const variants = {
        enter: { opacity: 0, x: viewMode === "editor" ? 20 : -20 },
        center: { opacity: 1, x: 0 },
        exit: { opacity: 0, x: viewMode === "editor" ? -20 : 20 },
    };

    return (
        <>
            <TransitionPanel activeIndex={viewMode === "list" ? 0 : 1} animateHeight transition={{ duration: 0.3, ease: "easeInOut" }} variants={variants}>
                {/* List View */}
                <TierListsTable canCreate={canCreate} canDelete={canDelete} loading={loading} onCreate={() => setCreateDialogOpen(true)} onDelete={handleDelete} onEdit={handleEdit} onRefresh={onRefresh} onView={handleView} tierLists={tierLists} />

                {/* Editor View */}
                {editorLoading ? (
                    <div className="flex min-h-100 items-center justify-center">
                        <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                    </div>
                ) : tierListData ? (
                    <TierListEditor allOperators={allOperators} canToggleActive={canToggleActive} onBack={handleBack} onPublish={handleOpenPublishDialog} onSave={handleSave} operatorsData={operatorsData} operatorsLoading={operatorsLoading} tierListData={tierListData} />
                ) : null}
            </TransitionPanel>

            {/* Create Dialog */}
            <CreateTierListDialog onCreate={handleCreate} onOpenChange={setCreateDialogOpen} open={createDialogOpen} />

            {/* Delete Confirmation Dialog */}
            <AlertDialog onOpenChange={setDeleteDialogOpen} open={deleteDialogOpen}>
                <AlertDialogContent>
                    <AlertDialogHeader>
                        <AlertDialogTitle>Delete Tier List</AlertDialogTitle>
                        <AlertDialogDescription>
                            Are you sure you want to delete <strong>{tierListToDelete?.name}</strong>? This action cannot be undone. All tiers, placements, and version history will be permanently deleted.
                        </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                        <AlertDialogCancel disabled={deleting}>Cancel</AlertDialogCancel>
                        <AlertDialogAction className="bg-destructive text-destructive-foreground hover:bg-destructive/90" disabled={deleting} onClick={confirmDelete}>
                            {deleting ? "Deleting..." : "Delete"}
                        </AlertDialogAction>
                    </AlertDialogFooter>
                </AlertDialogContent>
            </AlertDialog>

            {/* Publish Version Dialog */}
            <PublishVersionDialog onOpenChange={setPublishDialogOpen} onPublish={handlePublish} open={publishDialogOpen} tierListName={tierListData?.tier_list.name ?? "Tier List"} />
        </>
    );
}
