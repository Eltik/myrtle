import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Lock, RefreshCw } from "lucide-react";
import { useMemo, useState } from "react";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { fetchMyGachaRecordsFn, myGachaStoredRecordsQueryOptions } from "#/lib/api/gacha";
import { authActions } from "#/lib/auth/store";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./HistoryPage.messages";
import { BannerFilter } from "./impl/BannerFilter";
import { bannerOptions, filterRecordsByPools } from "./impl/banner-filter";
import { HistoryBody, hasAnyPull, useBannersById } from "./impl/HistoryBody";
import styles from "./impl/HistoryPage.module.css";
import { PublicHistorySwitch, ShareHistoryButton } from "./impl/ShareHistory";
import { gachaStorageOff } from "./impl/storage-gate";

function UnauthenticatedState() {
    const t: TypedT<typeof messages> = useT("gacha");

    return (
        <div className="flex flex-col items-center justify-center gap-6 rounded-[14px] border border-border bg-card px-8 py-16 text-center">
            <div className="flex flex-col items-center gap-3">
                <div className="flex h-14 w-14 items-center justify-center rounded-2xl border border-border bg-muted text-2xl">
                    <Lock />
                </div>
                <div>
                    <h2 className="font-sans font-semibold text-[20px] text-foreground tracking-[-0.02em]">{t("history.signIn.title")}</h2>
                    <p className="mt-1.5 max-w-[42ch] font-sans text-muted-foreground text-sm">{t("history.signIn.body")}</p>
                </div>
            </div>
            <button type="button" onClick={() => authActions.openLoginDialog()} className="inline-flex h-9 cursor-pointer items-center rounded-lg bg-primary px-5 font-medium font-sans text-primary-foreground text-sm transition-opacity hover:opacity-90">
                {t("history.signIn.action")}
            </button>
        </div>
    );
}

function NoDataState() {
    const t: TypedT<typeof messages> = useT("gacha");

    return (
        <div className="flex flex-col items-center justify-center gap-4 rounded-[14px] border border-border bg-card px-8 py-14 text-center">
            <div>
                <h2 className="font-sans font-semibold text-[18px] text-foreground tracking-[-0.02em]">{t("history.noData.title")}</h2>
                <p className="mt-1.5 max-w-[48ch] font-sans text-muted-foreground text-sm">{t("history.noData.body")}</p>
            </div>
            <Link to="/settings" className="inline-flex h-8 items-center rounded-lg border border-border bg-card px-4 font-medium font-sans text-foreground text-sm transition-colors hover:bg-muted">
                {t("history.noData.action")}
            </Link>
        </div>
    );
}

export function HistoryPage() {
    const t: TypedT<typeof messages> = useT("gacha");
    const { isAuthenticated, user } = useAuth();
    const queryClient = useQueryClient();
    // With storing off the backend refuses the refresh (409) before calling
    // Yostar, so the button is disabled and says why instead of failing.
    const storageOff = gachaStorageOff(user);

    const recordsQuery = useQuery(myGachaStoredRecordsQueryOptions(isAuthenticated));
    const bannersById = useBannersById(user?.server);
    // The banners the page is limited to; empty is every banner. Filtered here, from the
    // records already loaded, so picking a banner costs no request.
    const [pools, setPools] = useState<string[]>([]);

    const refreshMutation = useMutation({
        mutationFn: () => fetchMyGachaRecordsFn(),
        onSuccess: (result) => {
            toastManager.add({
                id: `gacha-refresh-${Date.now()}`,
                title: t("history.refresh.okTitle"),
                description: t("history.refresh.okBody", { count: result.totalFetched }),
                type: "success",
            });
            queryClient.invalidateQueries({ queryKey: ["gacha"] });
        },
        onError: (err: Error) => {
            toastManager.add({
                id: `gacha-refresh-error-${Date.now()}`,
                title: t("history.refresh.errTitle"),
                description: err.message || t("history.refresh.errBody"),
                type: "error",
            });
        },
    });

    const isLoading = recordsQuery.isLoading;
    const rawRecords = recordsQuery.data ?? null;
    const options = useMemo(() => bannerOptions(rawRecords, bannersById), [rawRecords, bannersById]);
    const records = useMemo(() => (rawRecords ? filterRecordsByPools(rawRecords, pools) : null), [rawRecords, pools]);

    const hasRecords = hasAnyPull(rawRecords);

    return (
        <>
            <div className={styles.pageAmbient} aria-hidden />
            <section className="mx-auto flex w-full max-w-330 flex-col gap-6 px-8 pt-7 pb-15 max-[760px]:px-4 max-[760px]:pt-5 max-[760px]:pb-10">
                <PageHeader>
                    {isAuthenticated ? (
                        <div className="flex flex-col items-start gap-1.5 sm:items-end">
                            <button
                                type="button"
                                onClick={() => refreshMutation.mutate()}
                                disabled={refreshMutation.isPending || storageOff}
                                aria-describedby={storageOff ? "gacha-storage-off" : undefined}
                                className="inline-flex h-9 cursor-pointer items-center gap-2 rounded-lg bg-primary px-4 font-medium font-sans text-primary-foreground text-sm shadow-[0_1px_2px_oklch(0_0_0/0.25)] transition-opacity hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
                            >
                                <RefreshCw className={`h-4 w-4 ${refreshMutation.isPending ? "animate-spin" : ""}`} aria-hidden />
                                {refreshMutation.isPending ? t("history.refreshing") : t("history.refresh")}
                            </button>
                            {storageOff ? (
                                <p id="gacha-storage-off" className="m-0 max-w-[40ch] font-sans text-muted-foreground text-xs sm:text-right">
                                    {t("history.refresh.storageOff")}{" "}
                                    <Link to="/settings" className="text-foreground underline underline-offset-2 hover:text-primary">
                                        {t("history.refresh.storageOffAction")}
                                    </Link>
                                </p>
                            ) : null}
                        </div>
                    ) : null}
                </PageHeader>

                {recordsQuery.isError ? (
                    <div className="rounded-[14px] border border-destructive/30 bg-destructive/8 px-5 py-4 font-sans text-foreground/90 text-sm">
                        <strong className="font-semibold text-foreground">{t("history.error.load")}</strong> {(recordsQuery.error as Error)?.message ?? t("history.error.unknown")}
                    </div>
                ) : null}

                {!isAuthenticated && !isLoading ? (
                    <UnauthenticatedState />
                ) : isAuthenticated && !isLoading && !hasRecords ? (
                    <NoDataState />
                ) : (
                    <>
                        {user ? (
                            <div className="flex flex-col gap-3 rounded-[14px] border border-border bg-card px-4 py-3 lg:flex-row lg:items-center lg:justify-between">
                                <BannerFilter options={options} value={pools} onChange={setPools} disabled={isLoading} />
                                <div className="flex flex-wrap items-center gap-3 sm:gap-4">
                                    <PublicHistorySwitch user={user} />
                                    <ShareHistoryButton user={user} pools={pools} />
                                </div>
                            </div>
                        ) : null}

                        <HistoryBody records={records} bannersById={bannersById} isLoading={isLoading} filtered={pools.length > 0} />
                    </>
                )}
            </section>
        </>
    );
}

function PageHeader({ children }: { children?: React.ReactNode }) {
    const t: TypedT<typeof messages> = useT("gacha");
    const rt: TypedRichT<typeof messages> = useRichT("gacha");

    return (
        <div className="flex flex-col gap-3.5 pt-1.5">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between sm:gap-6">
                <div className="flex max-w-180 flex-col items-start">
                    <span className="mb-2.5 inline-block font-bold text-[0.69rem] text-primary uppercase tracking-[0.22em]">{t("history.header.kicker")}</span>
                    <h1 className="m-0 mb-3 text-balance font-bold font-sans text-[32px] text-foreground leading-[1.05] tracking-[-0.03em] sm:text-[38px] sm:leading-[1.03] sm:tracking-[-0.035em] lg:text-[44px] lg:leading-[1.02]">
                        {rt("history.header.title", { emphasis: <em className="text-primary not-italic">{t("history.header.titleEmphasis")}</em> })}
                    </h1>
                    <p className="m-0 max-w-[60ch] font-sans text-muted-foreground">{t("history.header.blurb")}</p>
                </div>

                {children ? <div className="flex shrink-0 items-center gap-2">{children}</div> : null}
            </div>

            <div className="flex flex-wrap gap-2">
                <Link to="/gacha/community" className="inline-flex h-6 items-center rounded-md border border-border bg-card/80 px-2.5 font-sans text-[11.5px] text-muted-foreground transition-colors hover:text-foreground">
                    {t("history.header.communityLink")}
                </Link>
            </div>
        </div>
    );
}
