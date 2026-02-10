"use client";

import { AlertCircle, History, RefreshCw } from "lucide-react";
import Link from "next/link";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";
import { BannerTabs, GachaSettingsPopover, PullFilters, PullHistoryList, StatsOverview } from "~/components/gacha/history";
import ProtectedPageLayout from "~/components/layout/protected-page-layout";
import { SEO } from "~/components/seo";
import { Alert, AlertDescription, AlertTitle } from "~/components/ui/shadcn/alert";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Separator } from "~/components/ui/shadcn/separator";
import { useAuth } from "~/hooks/use-auth";
import { useGacha } from "~/hooks/use-gacha";
import { buildOperatorLookup, enrichGachaRecords, enrichRecordEntries, type OperatorLookupEntry } from "~/lib/gacha-utils";
import type { GachaHistoryParams, GachaType } from "~/types/api";

const DEFAULT_PAGE_SIZE = 25;

function GachaHistoryPageContent() {
    const { user, loading: authLoading } = useAuth();
    const { storedRecords, loading: loadingRecords, loadingStoredRecords, error: recordsError, fetchAllRecords, fetchStoredRecords, history, loadingHistory, fetchHistory, settings, fetchSettings } = useGacha();

    const [activeTab, setActiveTab] = useState<"all" | GachaType>("all");
    const [pageSize, setPageSize] = useState(DEFAULT_PAGE_SIZE);
    const [filters, setFilters] = useState<GachaHistoryParams>({
        limit: pageSize,
        offset: 0,
        order: "desc",
    });
    const [isRefreshing, setIsRefreshing] = useState(false);
    const [currentPage, setCurrentPage] = useState(1);
    const [isTabSwitching, setIsTabSwitching] = useState(false);
    const initialFetchDone = useRef(false);
    const [operatorMap, setOperatorMap] = useState<Map<string, OperatorLookupEntry> | null>(null);

    // Fetch operator data for enriching gacha records with correct rarity/name
    useEffect(() => {
        fetch("/api/static", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ type: "operators", fields: ["id", "name", "rarity", "profession"], limit: 1000 }),
        })
            .then((res) => res.json())
            .then((data) => {
                if (data.data && Array.isArray(data.data)) {
                    setOperatorMap(buildOperatorLookup(data.data));
                }
            })
            .catch((err) => console.error("Failed to fetch operator lookup data:", err));
    }, []);

    // Enrich history records with correct rarity/name from static game data
    const enrichedHistoryRecords = useMemo(() => {
        if (!history?.records) return [];
        if (!operatorMap) return history.records;
        return enrichRecordEntries(history.records, operatorMap);
    }, [history?.records, operatorMap]);

    // Enrich stored records (used for stats) with correct rarity/name
    const enrichedStoredRecords = useMemo(() => {
        if (!storedRecords) return null;
        if (!operatorMap) return storedRecords;
        return enrichGachaRecords(storedRecords, operatorMap);
    }, [storedRecords, operatorMap]);

    const handlePageSizeChange = useCallback(
        (newSize: number) => {
            setPageSize(newSize);
            setCurrentPage(1);
            const newFilters = {
                ...filters,
                limit: newSize,
                offset: 0,
                gachaType: activeTab === "all" ? undefined : activeTab,
            };
            setFilters(newFilters);
            fetchHistory(newFilters);
        },
        [filters, activeTab, fetchHistory],
    );

    useEffect(() => {
        if (user?.status && !authLoading && !initialFetchDone.current) {
            initialFetchDone.current = true;
            fetchAllRecords();
            fetchSettings();
            fetchHistory({ limit: pageSize, offset: 0, order: "desc" });
            fetchStoredRecords();
        }
    }, [user?.status, authLoading, fetchAllRecords, fetchHistory, fetchSettings, fetchStoredRecords, pageSize]);

    const handleTabChange = useCallback(
        async (tab: string) => {
            setIsTabSwitching(true);
            setCurrentPage(1);
            setActiveTab(tab as "all" | GachaType);
            const newFilters = {
                ...filters,
                offset: 0,
                gachaType: tab === "all" ? undefined : (tab as GachaType),
            };
            setFilters(newFilters);
            await fetchHistory(newFilters);
            setIsTabSwitching(false);
        },
        [filters, fetchHistory],
    );

    const handleApplyFilters = useCallback(() => {
        setCurrentPage(1);
        const newFilters = {
            ...filters,
            offset: 0,
            gachaType: activeTab === "all" ? undefined : activeTab,
        };
        setFilters(newFilters);
        fetchHistory(newFilters);
        toast.success("Filters applied");
    }, [filters, activeTab, fetchHistory]);

    const handleResetFilters = useCallback(() => {
        setCurrentPage(1);
        const newFilters: GachaHistoryParams = {
            limit: pageSize,
            offset: 0,
            order: "desc",
            gachaType: activeTab === "all" ? undefined : activeTab,
        };
        setFilters(newFilters);
        fetchHistory(newFilters);
        toast.success("Filters reset");
    }, [pageSize, activeTab, fetchHistory]);

    const handlePageChange = useCallback(
        (page: number) => {
            setCurrentPage(page);
            const newFilters = {
                ...filters,
                offset: (page - 1) * pageSize,
                gachaType: activeTab === "all" ? undefined : activeTab,
            };
            setFilters(newFilters);
            fetchHistory(newFilters);
        },
        [filters, pageSize, activeTab, fetchHistory],
    );

    const handleRefresh = useCallback(async () => {
        setIsRefreshing(true);
        try {
            await fetchAllRecords();
            await fetchStoredRecords();
            await fetchHistory({ ...filters, gachaType: activeTab === "all" ? undefined : activeTab });
            toast.success("Gacha data refreshed");
        } catch (error) {
            console.error("Error refreshing gacha data:", error);
            toast.error("Failed to refresh gacha data");
        } finally {
            setIsRefreshing(false);
        }
    }, [fetchAllRecords, fetchStoredRecords, fetchHistory, filters, activeTab]);

    const storageDisabled = settings && !settings.store_records;

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
                        <GachaSettingsPopover onPageSizeChange={handlePageSizeChange} pageSize={pageSize} />
                        <Button disabled={isRefreshing} onClick={handleRefresh} size="sm" variant="outline">
                            <RefreshCw className={`mr-2 h-4 w-4 ${isRefreshing ? "animate-spin" : ""}`} />
                            Refresh
                        </Button>
                    </div>
                </div>

                {/* Storage Disabled Warning */}
                {storageDisabled && (
                    <Alert>
                        <AlertCircle className="h-4 w-4" />
                        <AlertTitle>Gacha Storage Disabled</AlertTitle>
                        <AlertDescription>
                            You have disabled gacha record storage. Enable it in{" "}
                            <Link className="font-medium underline underline-offset-4" href="/my/settings">
                                settings
                            </Link>{" "}
                            to view your pull history.
                        </AlertDescription>
                    </Alert>
                )}

                {/* Error Display */}
                {recordsError && (
                    <Alert variant="destructive">
                        <AlertCircle className="h-4 w-4" />
                        <AlertTitle>Error</AlertTitle>
                        <AlertDescription>{recordsError}</AlertDescription>
                    </Alert>
                )}

                {/* Loading State */}
                {(loadingStoredRecords || loadingRecords) && !enrichedStoredRecords && (
                    <div className="flex min-h-[30vh] items-center justify-center">
                        <div className="text-center">
                            <div className="mx-auto h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                            <p className="mt-4 text-muted-foreground text-sm">Loading gacha data...</p>
                        </div>
                    </div>
                )}

                {/* Main Content */}
                {!loadingStoredRecords && enrichedStoredRecords && !storageDisabled && (
                    <>
                        {/* Statistics Overview */}
                        <StatsOverview loading={loadingStoredRecords} records={enrichedStoredRecords} />

                        <Separator />

                        {/* Banner Tabs and History */}
                        <BannerTabs activeTab={activeTab} isLoading={isTabSwitching} onTabChange={handleTabChange} records={enrichedStoredRecords}>
                            <div className="grid min-w-0 gap-6 lg:grid-cols-[1fr_300px]">
                                {/* Pull History List */}
                                <div className="min-w-0 space-y-4">
                                    <Card className="overflow-hidden">
                                        <CardHeader>
                                            <CardTitle>Pull History</CardTitle>
                                            <CardDescription>
                                                {(() => {
                                                    if (!enrichedStoredRecords) return "No pulls recorded";
                                                    const total =
                                                        activeTab === "all"
                                                            ? enrichedStoredRecords.limited.total + enrichedStoredRecords.regular.total + enrichedStoredRecords.special.total
                                                            : activeTab === "limited"
                                                              ? enrichedStoredRecords.limited.total
                                                              : activeTab === "regular"
                                                                ? enrichedStoredRecords.regular.total
                                                                : enrichedStoredRecords.special.total;
                                                    return total > 0 ? `${total.toLocaleString()} total pulls` : "No pulls recorded";
                                                })()}
                                            </CardDescription>
                                            <Separator className="mt-2" />
                                        </CardHeader>
                                        <CardContent>
                                            <PullHistoryList
                                                currentPage={currentPage}
                                                isPageLoading={loadingHistory && !!history}
                                                loading={loadingHistory && !history}
                                                onPageChange={handlePageChange}
                                                records={enrichedHistoryRecords}
                                                totalPages={history?.pagination.total ? Math.ceil(history.pagination.total / pageSize) : 1}
                                            />
                                        </CardContent>
                                    </Card>
                                </div>

                                {/* Filters Sidebar */}
                                <div className="lg:sticky lg:top-16 lg:self-start">
                                    <PullFilters filters={filters} onApply={handleApplyFilters} onFiltersChange={setFilters} onReset={handleResetFilters} />
                                </div>
                            </div>
                        </BannerTabs>
                    </>
                )}

                {/* Empty State */}
                {!loadingStoredRecords && enrichedStoredRecords && !storageDisabled && history?.pagination.total === 0 && (
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
export default function GachaPage() {
    return (
        <ProtectedPageLayout>
            <GachaHistoryPageContent />
        </ProtectedPageLayout>
    );
}
