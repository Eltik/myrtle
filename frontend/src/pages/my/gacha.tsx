"use client";

import { useEffect } from "react";
import { SEO } from "~/components/seo";
import { Card, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { useAuth } from "~/hooks/use-auth";
import { useGacha } from "~/hooks/use-gacha";

export default function GachaPage() {
    const { user, loading: authLoading } = useAuth();
    const { records, loading, error, fetchAllRecords } = useGacha();

    useEffect(() => {
        if (user?.status && !authLoading) {
            fetchAllRecords();
        }
    }, [user?.status, authLoading, fetchAllRecords]);

    // Loading auth state
    if (authLoading) {
        return (
            <>
                <SEO description="View your Arknights gacha pull history." noIndex path="/my/gacha" title="Gacha History" />
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
                <SEO description="View your Arknights gacha pull history." noIndex path="/my/gacha" title="Gacha History" />
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
            <SEO description="View your Arknights gacha pull history." noIndex path="/my/gacha" title="Gacha History" />
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

                {!loading && !error && records && (
                    <div className="space-y-6">
                        {/* TODO: Implement gacha UI components */}
                        {/* For now, just display the counts */}
                        <div className="grid gap-4 md:grid-cols-3">
                            <Card>
                                <CardHeader>
                                    <CardTitle>Limited Headhunting</CardTitle>
                                    <CardDescription>
                                        {records.limited.total} pulls ({records.limited.records.length} loaded)
                                    </CardDescription>
                                </CardHeader>
                            </Card>
                            <Card>
                                <CardHeader>
                                    <CardTitle>Regular Headhunting</CardTitle>
                                    <CardDescription>
                                        {records.regular.total} pulls ({records.regular.records.length} loaded)
                                    </CardDescription>
                                </CardHeader>
                            </Card>
                            <Card>
                                <CardHeader>
                                    <CardTitle>Special Headhunting</CardTitle>
                                    <CardDescription>
                                        {records.special.total} pulls ({records.special.records.length} loaded)
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
