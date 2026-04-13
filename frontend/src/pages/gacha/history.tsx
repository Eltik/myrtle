"use client";

import { AlertCircle, History, RefreshCw } from "lucide-react";
import Image from "next/image";
import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import ProtectedPageLayout from "~/components/layout/protected-page-layout";
import { SEO } from "~/components/seo";
import { Alert, AlertDescription, AlertTitle } from "~/components/ui/shadcn/alert";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Separator } from "~/components/ui/shadcn/separator";
import { useAuth } from "~/hooks/use-auth";
import { useGacha } from "~/hooks/use-gacha";
import { cn } from "~/lib/utils";
import type { GachaRecord, GachaStats } from "~/types/api/impl/gacha";

const DEFAULT_PAGE_SIZE = 25;
const RARITY_COLORS: Record<number, string> = {
    6: "#ff6e40",
    5: "#ffd740",
    4: "#b388ff",
    3: "#40c4ff",
    2: "#b0bec5",
    1: "#b0bec5",
};

function GachaHistoryPageContent() {
    const { user, loading: authLoading } = useAuth();
    const { records, loading, loadingHistory, error, triggerFetch, fetchHistory, fetchUserStats, userStats } = useGacha();

    const [pageSize, setPageSize] = useState(DEFAULT_PAGE_SIZE);
    const [currentPage, setCurrentPage] = useState(1);
    const [rarityFilter, setRarityFilter] = useState<string>("all");
    const [isRefreshing, setIsRefreshing] = useState(false);
    const initialFetchDone = useRef(false);

    useEffect(() => {
        if (user?.uid && !authLoading && !initialFetchDone.current) {
            initialFetchDone.current = true;
            fetchHistory({ limit: pageSize, offset: 0 });
            fetchUserStats();
        }
    }, [user?.uid, authLoading, fetchHistory, fetchUserStats, pageSize]);

    const handleRefresh = useCallback(async () => {
        setIsRefreshing(true);
        try {
            await triggerFetch();
            await fetchHistory({ limit: pageSize, offset: 0 });
            await fetchUserStats();
            setCurrentPage(1);
            toast.success("Gacha data refreshed");
        } catch (error) {
            console.error("Error refreshing gacha data:", error);
            toast.error("Failed to refresh gacha data");
        } finally {
            setIsRefreshing(false);
        }
    }, [triggerFetch, fetchHistory, fetchUserStats, pageSize]);

    const handlePageChange = useCallback(
        (page: number) => {
            setCurrentPage(page);
            const rarity = rarityFilter !== "all" ? Number(rarityFilter) : undefined;
            fetchHistory({
                limit: pageSize,
                offset: (page - 1) * pageSize,
                rarity,
            });
        },
        [fetchHistory, pageSize, rarityFilter],
    );

    const handleRarityFilter = useCallback(
        (value: string) => {
            setRarityFilter(value);
            setCurrentPage(1);
            const rarity = value !== "all" ? Number(value) : undefined;
            fetchHistory({
                limit: pageSize,
                offset: 0,
                rarity,
            });
        },
        [fetchHistory, pageSize],
    );

    return (
        <>
            <SEO description="View your Arknights gacha pull history and statistics." noIndex path="/gacha/history" title="Gacha History" />
            <div className="mx-auto max-w-7xl space-y-6">
                {/* Header */}
                <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                    <div className="space-y-1">
                        <div className="flex items-center gap-3">
                            <History className="h-8 w-8 text-primary" />
                            <h1 className="font-bold text-3xl">Gacha History</h1>
                        </div>
                        <p className="text-muted-foreground">Track your pull statistics and history</p>
                    </div>
                    <div className="flex gap-2">
                        <Button disabled={isRefreshing || loading} onClick={handleRefresh} size="sm" variant="outline">
                            <RefreshCw className={`mr-2 h-4 w-4 ${isRefreshing ? "animate-spin" : ""}`} />
                            Refresh
                        </Button>
                    </div>
                </div>

                {/* Error Display */}
                {error && (
                    <Alert variant="destructive">
                        <AlertCircle className="h-4 w-4" />
                        <AlertTitle>Error</AlertTitle>
                        <AlertDescription>{error}</AlertDescription>
                    </Alert>
                )}

                {/* Loading State */}
                {(loading || loadingHistory) && records.length === 0 && !error && (
                    <div className="flex min-h-[30vh] items-center justify-center">
                        <div className="text-center">
                            <div className="mx-auto h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                            <p className="mt-4 text-muted-foreground text-sm">Loading gacha data...</p>
                        </div>
                    </div>
                )}

                {/* Statistics Overview */}
                {userStats && <StatsCards stats={userStats} />}

                {/* Main Content */}
                {records.length > 0 && (
                    <>
                        <Separator />

                        {/* Filters */}
                        <div className="flex items-center gap-4">
                            <Select onValueChange={handleRarityFilter} value={rarityFilter}>
                                <SelectTrigger className="w-40">
                                    <SelectValue placeholder="Filter by rarity" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="all">All Rarities</SelectItem>
                                    <SelectItem value="6">6-Star</SelectItem>
                                    <SelectItem value="5">5-Star</SelectItem>
                                    <SelectItem value="4">4-Star</SelectItem>
                                    <SelectItem value="3">3-Star</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>

                        {/* Pull History */}
                        <Card>
                            <CardHeader>
                                <CardTitle>Pull History</CardTitle>
                                <CardDescription>{records.length} records loaded</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div className="space-y-1">
                                    {records.map((record) => (
                                        <PullRow key={record.id} record={record} />
                                    ))}
                                </div>

                                {/* Pagination */}
                                <div className="mt-4 flex items-center justify-center gap-2">
                                    <Button disabled={currentPage === 1 || loadingHistory} onClick={() => handlePageChange(currentPage - 1)} size="sm" variant="outline">
                                        Previous
                                    </Button>
                                    <span className="text-muted-foreground text-sm">Page {currentPage}</span>
                                    <Button disabled={records.length < pageSize || loadingHistory} onClick={() => handlePageChange(currentPage + 1)} size="sm" variant="outline">
                                        Next
                                    </Button>
                                </div>
                            </CardContent>
                        </Card>
                    </>
                )}

                {/* Empty State */}
                {!loading && !loadingHistory && records.length === 0 && !error && (
                    <Card>
                        <CardContent className="flex min-h-75 flex-col items-center justify-center py-12 text-center">
                            <History className="mb-4 h-12 w-12 text-muted-foreground" />
                            <h3 className="mb-2 font-semibold text-lg">No Pull History</h3>
                            <p className="mb-4 max-w-sm text-muted-foreground text-sm">Your pull history will appear here once you sync your gacha data from the game.</p>
                            <Button onClick={handleRefresh} variant="outline">
                                <RefreshCw className="mr-2 h-4 w-4" />
                                Sync Gacha Data
                            </Button>
                        </CardContent>
                    </Card>
                )}
            </div>
        </>
    );
}

function StatsCards({ stats }: { stats: GachaStats }) {
    return (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <Card>
                <CardContent className="p-4">
                    <p className="text-muted-foreground text-sm">Total Pulls</p>
                    <p className="font-bold text-2xl">{(stats.total_pulls ?? 0).toLocaleString()}</p>
                </CardContent>
            </Card>
            <Card>
                <CardContent className="p-4">
                    <p className="text-muted-foreground text-sm">6-Star Pulls</p>
                    <p className="font-bold text-2xl" style={{ color: RARITY_COLORS[6] }}>
                        {(stats.six_star_count ?? 0).toLocaleString()}
                    </p>
                </CardContent>
            </Card>
            <Card>
                <CardContent className="p-4">
                    <p className="text-muted-foreground text-sm">5-Star Pulls</p>
                    <p className="font-bold text-2xl" style={{ color: RARITY_COLORS[5] }}>
                        {(stats.five_star_count ?? 0).toLocaleString()}
                    </p>
                </CardContent>
            </Card>
            <Card>
                <CardContent className="p-4">
                    <p className="text-muted-foreground text-sm">4-Star Pulls</p>
                    <p className="font-bold text-2xl" style={{ color: RARITY_COLORS[4] }}>
                        {(stats.four_star_count ?? 0).toLocaleString()}
                    </p>
                </CardContent>
            </Card>
        </div>
    );
}

function PullRow({ record }: { record: GachaRecord }) {
    const rarityColor = RARITY_COLORS[record.rarity] ?? "#b0bec5";
    const date = new Date(record.pull_timestamp * 1000);
    const dateStr = date.toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" });

    return (
        <div className="group flex items-center gap-3 rounded-md px-2 py-1.5 hover:bg-card">
            <div className="h-6 w-1 rounded-full" style={{ backgroundColor: rarityColor }} />
            <div className="relative h-8 w-8 shrink-0 overflow-hidden rounded">
                <Image alt={record.char_id} className="object-cover" fill sizes="32px" src={`/api/cdn/avatar/${record.char_id}`} />
            </div>
            <div className="min-w-0 flex-1">
                <span className="truncate text-sm font-medium">{record.char_id.replace("char_", "").replace(/_/g, " ")}</span>
            </div>
            <span className="shrink-0 text-xs font-mono" style={{ color: rarityColor }}>
                {record.rarity}-Star
            </span>
            {record.pool_name && <span className="hidden shrink-0 text-muted-foreground text-xs sm:inline">{record.pool_name}</span>}
            <span className="shrink-0 text-muted-foreground text-xs">{dateStr}</span>
        </div>
    );
}

export default function GachaPage() {
    return (
        <ProtectedPageLayout>
            <GachaHistoryPageContent />
        </ProtectedPageLayout>
    );
}
