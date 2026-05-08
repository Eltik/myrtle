import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { gachaEnhancedStatsQueryOptions } from "#/lib/api/gacha";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import type { IOperatorIndexEntry } from "#/types/operators";
import styles from "./impl/CommunityPage.module.css";
import { KpiStrip } from "./impl/KpiStrip";
import { Leaderboard } from "./impl/Leaderboard";
import { PageHeader } from "./impl/PageHeader";
import { RarityPanel } from "./impl/RarityPanel";
import { TimingPanel } from "./impl/TimingPanel";

export function CommunityPage() {
    const enhanced = useQuery(gachaEnhancedStatsQueryOptions({ topN: 20, includeTiming: true }));
    const operators = useQuery(operatorsIndexQueryOptions());

    const operatorsById = useMemo(() => {
        const map = new Map<string, IOperatorIndexEntry>();
        for (const entry of operators.data ?? []) map.set(entry.id, entry);
        return map;
    }, [operators.data]);

    const data = enhanced.data ?? null;
    const isLoading = enhanced.isLoading;

    return (
        <>
            <div className={styles.pageAmbient} aria-hidden />
            <section className="mx-auto flex w-full max-w-330 flex-col gap-6 px-8 pb-15 pt-7 max-[760px]:px-4 max-[760px]:pt-5 max-[760px]:pb-10">
                <PageHeader data={data} isLoading={isLoading} />

                {enhanced.isError ? (
                    <div className="rounded-[14px] border border-destructive/30 bg-destructive/8 px-5 py-4 font-sans text-sm text-foreground/90">
                        <strong className="font-semibold text-foreground">Couldn&rsquo;t load community stats.</strong> {(enhanced.error as Error)?.message ?? "Unknown error."}
                    </div>
                ) : null}

                <KpiStrip data={data} />

                <div className="grid gap-4 grid-cols-1 lg:grid-cols-[1.3fr_1fr]">
                    <Leaderboard ops={data?.mostCommonOperators ?? []} operatorsById={operatorsById} isLoading={isLoading} />
                    <RarityPanel data={data} />
                </div>

                <TimingPanel timing={data?.pullTiming} />
            </section>
        </>
    );
}
