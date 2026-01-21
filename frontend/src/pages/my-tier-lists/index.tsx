"use client";

import { useCallback, useEffect, useState } from "react";
import { SEO } from "~/components/seo";
import { MyTierListsManagement } from "~/components/tier-lists/my-tier-lists-management";
import { Card, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { useAuth } from "~/hooks/use-auth";
import type { TierListType } from "~/types/api/impl/tier-list";

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

export default function MyTierListsPage() {
    const { user, loading: authLoading } = useAuth();
    const [tierLists, setTierLists] = useState<CommunityTierList[]>([]);
    const [loading, setLoading] = useState(true);

    const fetchTierLists = useCallback(async () => {
        setLoading(true);
        try {
            const response = await fetch("/api/tier-lists/mine");
            const data = await response.json();

            if (data.success) {
                setTierLists(data.tier_lists);
            } else {
                console.error("Failed to fetch tier lists:", data.error);
            }
        } catch (error) {
            console.error("Error fetching tier lists:", error);
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        if (user?.status) {
            fetchTierLists();
        } else if (!authLoading) {
            setLoading(false);
        }
    }, [user, authLoading, fetchTierLists]);

    if (authLoading) {
        return (
            <>
                <SEO description="Create and manage your community tier lists for Arknights operators." noIndex path="/my-tier-lists" title="My Tier Lists" />
                <div className="flex min-h-[50vh] items-center justify-center">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                </div>
            </>
        );
    }

    if (!user?.status) {
        return (
            <>
                <SEO description="Create and manage your community tier lists for Arknights operators." noIndex path="/my-tier-lists" title="My Tier Lists" />
                <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                    <Card className="max-w-md">
                        <CardHeader>
                            <CardTitle>Authentication Required</CardTitle>
                            <CardDescription>Please log in to view and manage your tier lists.</CardDescription>
                        </CardHeader>
                    </Card>
                </div>
            </>
        );
    }

    return (
        <>
            <SEO description="Create and manage your community tier lists for Arknights operators." noIndex path="/my-tier-lists" title="My Tier Lists" />
            <div className="mx-auto max-w-6xl">
                <MyTierListsManagement loading={loading} onRefresh={fetchTierLists} tierLists={tierLists} />
            </div>
        </>
    );
}
