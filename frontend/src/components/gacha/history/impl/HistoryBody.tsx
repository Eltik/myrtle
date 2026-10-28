import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { bannersQueryOptions, deriveClientGachaRecords, type IBanner, type IGachaRecords, pullsGamedataServer } from "#/lib/api/gacha";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { useGamedataServer } from "#/lib/i18n";
import type { IOperatorIndexEntry } from "#/types/operators";
import { BannerBreakdown } from "./BannerBreakdown";
import { BannerHistory } from "./BannerHistory";
import { HistoryKpiStrip } from "./HistoryKpiStrip";
import { PityPanel } from "./PityPanel";
import { TopOperators } from "./TopOperators";

/**
 * The banners `server`'s account pulled on, keyed by pool id. Pool ids only resolve in
 * the table of the server the pulls came from, so this follows the account, not the
 * reader's text pick.
 */
export function useBannersById(server: string | null | undefined): Map<string, IBanner> {
    const bannersQuery = useQuery(bannersQueryOptions(pullsGamedataServer(server)));
    return useMemo(() => {
        const map = new Map<string, IBanner>();
        for (const b of bannersQuery.data ?? []) map.set(b.gachaPoolId, b);
        return map;
    }, [bannersQuery.data]);
}

interface IHistoryBodyProps {
    /** The pulls to show, already limited to the chosen banners; `null` while loading. */
    records: IGachaRecords | null;
    bannersById: Map<string, IBanner>;
    isLoading: boolean;
    /**
     * A banner filter is applied. Pity is a running counter across every banner of a
     * group, so a subset of banners cannot give it and the panel is left out.
     */
    filtered: boolean;
}

/**
 * The history's panels: KPIs, pity, top operators, the per-banner breakdown and the
 * pull table. The owner's page (`HistoryPage`) and a profile's Pulls tab both render it.
 */
export function HistoryBody({ records, bannersById, isLoading, filtered }: IHistoryBodyProps) {
    // Operator names follow the reader's text pick.
    const server = useGamedataServer();
    const operatorsQuery = useQuery(operatorsIndexQueryOptions(server));
    const operatorsById = useMemo(() => {
        const map = new Map<string, IOperatorIndexEntry>();
        for (const entry of operatorsQuery.data ?? []) map.set(entry.id, entry);
        return map;
    }, [operatorsQuery.data]);

    const derived = useMemo(() => (records ? deriveClientGachaRecords(records) : null), [records]);

    return (
        <>
            <HistoryKpiStrip records={derived} isLoading={isLoading} />

            {filtered ? null : <PityPanel records={derived} bannersById={bannersById} isLoading={isLoading} />}

            <div className="grid grid-cols-1 gap-4 lg:grid-cols-[1.3fr_1fr]">
                <TopOperators records={derived} operatorsById={operatorsById} isLoading={isLoading} />
                <BannerBreakdown records={derived} bannersById={bannersById} operatorsById={operatorsById} isLoading={isLoading} />
            </div>

            <BannerHistory records={derived} operatorsById={operatorsById} bannersById={bannersById} isLoading={isLoading} />
        </>
    );
}

/** Whether `records` holds any pull at all. */
export function hasAnyPull(records: IGachaRecords | null): boolean {
    return records != null && records.limited.total + records.regular.total + records.special.total > 0;
}
