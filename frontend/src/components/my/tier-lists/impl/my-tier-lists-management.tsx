"use client";

import { Edit, Eye, LayoutList, MoreHorizontal, Plus, RefreshCw, Trash2 } from "lucide-react";
import Link from "next/link";
import { useCallback, useState } from "react";
import { toast } from "sonner";
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from "~/components/ui/shadcn/alert-dialog";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "~/components/ui/shadcn/dropdown-menu";
import type { TierListType } from "~/types/api/impl/tier-list";
import { CreateCommunityTierListDialog } from "./create-community-tier-list-dialog";

const MAX_COMMUNITY_TIER_LISTS = 10;

interface CommunityTierList {
    id: string;
    name: string;
    slug: string;
    description: string | null;
    is_active: boolean;
    tier_list_type: TierListType;
    created_by: string | null;
    created_at: string;
    updated_at: string;
}

interface MyTierListsManagementProps {
    tierLists: CommunityTierList[];
    loading?: boolean;
    onRefresh?: () => void;
}

export function MyTierListsManagement({ tierLists, loading = false, onRefresh }: MyTierListsManagementProps) {
    const [createDialogOpen, setCreateDialogOpen] = useState(false);
    const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
    const [tierListToDelete, setTierListToDelete] = useState<CommunityTierList | null>(null);
    const [deleting, setDeleting] = useState(false);

    const canCreateMore = tierLists.length < MAX_COMMUNITY_TIER_LISTS;

    const handleDelete = useCallback((tierList: CommunityTierList) => {
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
                const data = await response.json().catch(() => ({}));
                throw new Error(data.error || "Failed to delete tier list");
            }

            toast.success(`Deleted tier list: ${tierListToDelete.name}`);
            onRefresh?.();
        } catch (error) {
            console.error("Failed to delete tier list:", error);
            toast.error(error instanceof Error ? error.message : "Failed to delete tier list");
        } finally {
            setDeleting(false);
            setDeleteDialogOpen(false);
            setTierListToDelete(null);
        }
    }, [tierListToDelete, onRefresh]);

    const handleCreate = useCallback(
        async (data: { name: string; slug: string; description: string }) => {
            const response = await fetch("/api/tier-lists", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    name: data.name,
                    slug: data.slug,
                    description: data.description || null,
                    tier_list_type: "community",
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

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div className="space-y-1">
                    <h1 className="font-bold text-2xl tracking-tight">My Tier Lists</h1>
                    <p className="text-muted-foreground text-sm">Create and manage your community tier lists</p>
                </div>
                <div className="flex items-center gap-2">
                    <Badge className="bg-muted text-muted-foreground" variant="secondary">
                        {tierLists.length} / {MAX_COMMUNITY_TIER_LISTS} used
                    </Badge>
                    <Button disabled={!canCreateMore} onClick={() => setCreateDialogOpen(true)} size="sm">
                        <Plus className="mr-1.5 h-4 w-4" />
                        Create New
                    </Button>
                    {onRefresh && (
                        <Button disabled={loading} onClick={onRefresh} size="sm" variant="outline">
                            <RefreshCw className={`h-4 w-4 ${loading ? "animate-spin" : ""}`} />
                        </Button>
                    )}
                </div>
            </div>

            {/* Tier Lists Grid */}
            {tierLists.length > 0 ? (
                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                    {tierLists.map((tierList) => (
                        <TierListCard key={tierList.id} onDelete={() => handleDelete(tierList)} tierList={tierList} />
                    ))}
                </div>
            ) : (
                <Card className="border-dashed">
                    <CardContent className="flex flex-col items-center justify-center py-12 text-center">
                        <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-muted">
                            <LayoutList className="h-6 w-6 text-muted-foreground" />
                        </div>
                        <CardTitle className="mb-2 text-lg">No tier lists yet</CardTitle>
                        <CardDescription className="mb-4 max-w-sm">Create your first community tier list to share your operator rankings with others.</CardDescription>
                        <Button disabled={!canCreateMore} onClick={() => setCreateDialogOpen(true)}>
                            <Plus className="mr-1.5 h-4 w-4" />
                            Create Your First Tier List
                        </Button>
                    </CardContent>
                </Card>
            )}

            {/* Create Dialog */}
            <CreateCommunityTierListDialog onCreate={handleCreate} onOpenChange={setCreateDialogOpen} open={createDialogOpen} />

            {/* Delete Confirmation Dialog */}
            <AlertDialog onOpenChange={setDeleteDialogOpen} open={deleteDialogOpen}>
                <AlertDialogContent>
                    <AlertDialogHeader>
                        <AlertDialogTitle>Delete Tier List</AlertDialogTitle>
                        <AlertDialogDescription>
                            Are you sure you want to delete <strong>{tierListToDelete?.name}</strong>? This action cannot be undone. All tiers and placements will be permanently deleted.
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
        </div>
    );
}

function TierListCard({ tierList, onDelete }: { tierList: CommunityTierList; onDelete: () => void }) {
    const formattedDate = new Date(tierList.updated_at).toLocaleDateString("en-US", {
        year: "numeric",
        month: "short",
        day: "numeric",
    });

    return (
        <Card className="group relative transition-all hover:border-primary/50">
            <CardHeader className="pb-3">
                <div className="flex items-start justify-between gap-2">
                    <div className="min-w-0 flex-1">
                        <CardTitle className="truncate text-base">{tierList.name}</CardTitle>
                        <p className="mt-0.5 font-mono text-muted-foreground text-xs">{tierList.slug}</p>
                    </div>
                    <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                            <Button className="h-8 w-8 shrink-0" size="icon" variant="ghost">
                                <MoreHorizontal className="h-4 w-4" />
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                            <DropdownMenuItem asChild>
                                <Link href={`/operators/tier-list?slug=${tierList.slug}`}>
                                    <Eye className="mr-2 h-4 w-4" />
                                    View
                                </Link>
                            </DropdownMenuItem>
                            <DropdownMenuItem asChild>
                                <Link href={`/my/tier-lists/${tierList.slug}`}>
                                    <Edit className="mr-2 h-4 w-4" />
                                    Edit
                                </Link>
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem className="text-destructive focus:text-destructive" onClick={onDelete}>
                                <Trash2 className="mr-2 h-4 w-4" />
                                Delete
                            </DropdownMenuItem>
                        </DropdownMenuContent>
                    </DropdownMenu>
                </div>
            </CardHeader>
            <CardContent className="pt-0">
                {tierList.description && <p className="mb-3 line-clamp-2 text-muted-foreground text-sm">{tierList.description}</p>}
                <div className="flex items-center justify-between">
                    <Badge className={tierList.is_active ? "bg-green-500/10 text-green-500" : "bg-zinc-500/10 text-zinc-400"} variant="outline">
                        {tierList.is_active ? "Active" : "Inactive"}
                    </Badge>
                    <span className="text-muted-foreground text-xs">Updated {formattedDate}</span>
                </div>
            </CardContent>
        </Card>
    );
}
