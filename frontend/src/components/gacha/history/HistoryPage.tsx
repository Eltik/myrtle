import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Lock, RefreshCw } from "lucide-react";
import { useMemo } from "react";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { bannersQueryOptions, deriveClientGachaRecords, fetchMyGachaRecordsFn, type IBanner, myGachaStoredRecordsQueryOptions } from "#/lib/api/gacha";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { authActions } from "#/lib/auth/store";
import type { IOperatorIndexEntry } from "#/types/operators";
import { BannerBreakdown } from "./impl/BannerBreakdown";
import { BannerHistory } from "./impl/BannerHistory";
import { HistoryKpiStrip } from "./impl/HistoryKpiStrip";
import styles from "./impl/HistoryPage.module.css";
import { PityPanel } from "./impl/PityPanel";
import { TopOperators } from "./impl/TopOperators";

function UnauthenticatedState() {
    return (
        <div className="flex flex-col items-center justify-center gap-6 rounded-[14px] border border-border bg-card px-8 py-16 text-center">
            <div className="flex flex-col items-center gap-3">
                <div className="flex h-14 w-14 items-center justify-center rounded-2xl border border-border bg-muted text-2xl">
                    <Lock />
                </div>
                <div>
                    <h2 className="font-sans font-semibold text-[20px] text-foreground tracking-[-0.02em]">Sign in to view your history</h2>
                    <p className="mt-1.5 max-w-[42ch] font-sans text-muted-foreground text-sm">Your pull history, pity counters, and operator statistics are only available after linking your Yostar account.</p>
                </div>
            </div>
            <button type="button" onClick={() => authActions.openLoginDialog()} className="inline-flex h-9 cursor-pointer items-center rounded-lg bg-primary px-5 font-medium font-sans text-primary-foreground text-sm transition-opacity hover:opacity-90">
                Sign in
            </button>
        </div>
    );
}

function NoDataState() {
    return (
        <div className="flex flex-col items-center justify-center gap-4 rounded-[14px] border border-border bg-card px-8 py-14 text-center">
            <div>
                <h2 className="font-sans font-semibold text-[18px] text-foreground tracking-[-0.02em]">No pull records found</h2>
                <p className="mt-1.5 max-w-[48ch] font-sans text-muted-foreground text-sm">Sync your gacha records from the settings page to see your history, pity, and statistics here.</p>
            </div>
            <Link to="/settings" className="inline-flex h-8 items-center rounded-lg border border-border bg-card px-4 font-medium font-sans text-foreground text-sm transition-colors hover:bg-muted">
                Go to settings
            </Link>
        </div>
    );
}

export function HistoryPage() {
    const { isAuthenticated } = useAuth();
    const queryClient = useQueryClient();

    const recordsQuery = useQuery(myGachaStoredRecordsQueryOptions(isAuthenticated));
    const operatorsQuery = useQuery(operatorsIndexQueryOptions());
    const bannersQuery = useQuery(bannersQueryOptions());

    const refreshMutation = useMutation({
        mutationFn: () => fetchMyGachaRecordsFn(),
        onSuccess: (result) => {
            toastManager.add({
                id: `gacha-refresh-${Date.now()}`,
                title: "Up to date",
                description: `Synced ${result.totalFetched} records from Yostar.`,
                type: "success",
            });
            queryClient.invalidateQueries({ queryKey: ["gacha"] });
        },
        onError: (err: Error) => {
            toastManager.add({
                id: `gacha-refresh-error-${Date.now()}`,
                title: "Couldn't refresh",
                description: err.message || "Try signing in again.",
                type: "error",
            });
        },
    });

    const operatorsById = useMemo(() => {
        const map = new Map<string, IOperatorIndexEntry>();
        for (const entry of operatorsQuery.data ?? []) map.set(entry.id, entry);
        return map;
    }, [operatorsQuery.data]);

    const bannersById = useMemo(() => {
        const map = new Map<string, IBanner>();
        for (const b of bannersQuery.data ?? []) map.set(b.gachaPoolId, b);
        return map;
    }, [bannersQuery.data]);

    const isLoading = recordsQuery.isLoading;
    const rawRecords = recordsQuery.data ?? null;

    const records = useMemo(() => (rawRecords ? deriveClientGachaRecords(rawRecords) : null), [rawRecords]);

    const hasRecords = records != null && (records.limited.total > 0 || records.linkage.total > 0 || records.regular.total > 0 || records.special.total > 0);

    return (
        <>
            <div className={styles.pageAmbient} aria-hidden />
            <section className="mx-auto flex w-full max-w-330 flex-col gap-6 px-8 pt-7 pb-15 max-[760px]:px-4 max-[760px]:pt-5 max-[760px]:pb-10">
                <PageHeader>
                    {isAuthenticated ? (
                        <button
                            type="button"
                            onClick={() => refreshMutation.mutate()}
                            disabled={refreshMutation.isPending}
                            className="inline-flex h-9 cursor-pointer items-center gap-2 rounded-lg bg-primary px-4 font-medium font-sans text-primary-foreground text-sm shadow-[0_1px_2px_oklch(0_0_0/0.25)] transition-opacity hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
                        >
                            <RefreshCw className={`h-4 w-4 ${refreshMutation.isPending ? "animate-spin" : ""}`} aria-hidden />
                            {refreshMutation.isPending ? "Refreshing…" : "Refresh records"}
                        </button>
                    ) : null}
                </PageHeader>

                {recordsQuery.isError ? (
                    <div className="rounded-[14px] border border-destructive/30 bg-destructive/8 px-5 py-4 font-sans text-foreground/90 text-sm">
                        <strong className="font-semibold text-foreground">Couldn&rsquo;t load pull records.</strong> {(recordsQuery.error as Error)?.message ?? "Unknown error."}
                    </div>
                ) : null}

                {!isAuthenticated && !isLoading ? (
                    <UnauthenticatedState />
                ) : isAuthenticated && !isLoading && !hasRecords ? (
                    <NoDataState />
                ) : (
                    <>
                        <HistoryKpiStrip records={records} isLoading={isLoading} />

                        <PityPanel records={records} bannersById={bannersById} isLoading={isLoading} />

                        <div className="grid grid-cols-1 gap-4 lg:grid-cols-[1.3fr_1fr]">
                            <TopOperators records={records} operatorsById={operatorsById} isLoading={isLoading} />
                            <BannerBreakdown records={records} bannersById={bannersById} operatorsById={operatorsById} isLoading={isLoading} />
                        </div>

                        <BannerHistory records={records} operatorsById={operatorsById} bannersById={bannersById} isLoading={isLoading} />
                    </>
                )}
            </section>
        </>
    );
}

function PageHeader({ children }: { children?: React.ReactNode }) {
    return (
        <div className="flex flex-col gap-3.5 pt-1.5">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between sm:gap-6">
                <div className="flex max-w-180 flex-col items-start">
                    <span className="mb-2.5 inline-block font-bold text-[0.69rem] text-primary uppercase tracking-[0.22em]">Personal · pull history</span>
                    <h1 className="m-0 mb-3 text-balance font-bold font-sans text-[32px] text-foreground leading-[1.05] tracking-[-0.03em] sm:text-[38px] sm:leading-[1.03] sm:tracking-[-0.035em] lg:text-[44px] lg:leading-[1.02]">
                        Your <em className="text-primary not-italic">gacha</em> history.
                    </h1>
                    <p className="m-0 max-w-[60ch] font-sans text-muted-foreground">Pull counts, current pity, your most-pulled operators, and a full history across every banner type.</p>
                </div>

                {children ? <div className="flex shrink-0 items-center gap-2">{children}</div> : null}
            </div>

            <div className="flex flex-wrap gap-2">
                <Link to="/gacha/community" className="inline-flex h-6 items-center rounded-md border border-border bg-card/80 px-2.5 font-sans text-[11.5px] text-muted-foreground transition-colors hover:text-foreground">
                    View community stats →
                </Link>
            </div>
        </div>
    );
}
