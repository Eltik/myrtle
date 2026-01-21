"use client";

import { useCallback, useEffect, useState } from "react";
import { SEO } from "~/components/seo";
import { Card, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { useAuth } from "~/hooks/use-auth";

// Types matching API response
interface GachaItem {
    charId: string;
    charName: string;
    star: string;
    color: string;
    poolId: string;
    poolName: string;
    typeName: string;
    at: number;
    atStr: string;
}

interface GachaTypeRecords {
    gacha_type: "limited" | "regular" | "special";
    records: GachaItem[];
    total: number;
}

interface GachaRecords {
    limited: GachaTypeRecords;
    regular: GachaTypeRecords;
    special: GachaTypeRecords;
}

export default function GachaPage() {
    const { user, loading: authLoading } = useAuth();
    const [gachaData, setGachaData] = useState<GachaRecords | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const fetchGachaRecords = useCallback(async () => {
        setLoading(true);
        setError(null);

        try {
            const response = await fetch("/api/gacha");
            const data = await response.json();

            if (data.success) {
                setGachaData(data.data);
            } else {
                setError(data.error || "Failed to fetch gacha records");
            }
        } catch (err) {
            console.error("Error fetching gacha records:", err);
            setError("An error occurred while fetching gacha records");
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        if (user?.status) {
            fetchGachaRecords();
        } else if (!authLoading) {
            setLoading(false);
        }
    }, [user, authLoading, fetchGachaRecords]);

    // Loading auth state
    if (authLoading) {
        return (
            <>
                <SEO description="View your Arknights gacha pull history." noIndex path="/gacha" title="Gacha History" />
                <div className="flex min-h-[50vh] items-center justify-center">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                </div>
            </>
        );
    }

    // Not authenticated
    if (!user?.status) {
        return (
            <>
                <SEO description="View your Arknights gacha pull history." noIndex path="/gacha" title="Gacha History" />
                <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                    <Card className="max-w-md">
                        <CardHeader>
                            <CardTitle>Authentication Required</CardTitle>
                            <CardDescription>Please log in to view your gacha history.</CardDescription>
                        </CardHeader>
                    </Card>
                </div>
            </>
        );
    }

    // Authenticated - render content
    return (
        <>
            <SEO description="View your Arknights gacha pull history." noIndex path="/gacha" title="Gacha History" />
            <div className="mx-auto max-w-6xl">
                <h1 className="mb-6 font-bold text-3xl">Gacha History</h1>

                {loading && (
                    <div className="flex min-h-[30vh] items-center justify-center">
                        <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                    </div>
                )}

                {error && (
                    <Card className="border-destructive">
                        <CardHeader>
                            <CardTitle className="text-destructive">Error</CardTitle>
                            <CardDescription>{error}</CardDescription>
                        </CardHeader>
                    </Card>
                )}

                {!loading && !error && gachaData && (
                    <div className="space-y-6">
                        {/* TODO: Implement gacha UI components */}
                        {/* For now, just display the counts */}
                        <div className="grid gap-4 md:grid-cols-3">
                            <Card>
                                <CardHeader>
                                    <CardTitle>Limited Headhunting</CardTitle>
                                    <CardDescription>
                                        {gachaData.limited.total} pulls ({gachaData.limited.records.length} loaded)
                                    </CardDescription>
                                </CardHeader>
                            </Card>
                            <Card>
                                <CardHeader>
                                    <CardTitle>Regular Headhunting</CardTitle>
                                    <CardDescription>
                                        {gachaData.regular.total} pulls ({gachaData.regular.records.length} loaded)
                                    </CardDescription>
                                </CardHeader>
                            </Card>
                            <Card>
                                <CardHeader>
                                    <CardTitle>Special Headhunting</CardTitle>
                                    <CardDescription>
                                        {gachaData.special.total} pulls ({gachaData.special.records.length} loaded)
                                    </CardDescription>
                                </CardHeader>
                            </Card>
                        </div>
                    </div>
                )}
            </div>
        </>
    );
}
