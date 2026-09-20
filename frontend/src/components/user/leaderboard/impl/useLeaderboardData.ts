import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { type IMaterials, materialsQueryOptions } from "#/lib/api/materials";
import { type IItemStanding, type ILeaderboardMover, type IPlayerStanding, itemCatalogQueryOptions, itemLeaderboardQueryOptions, itemStandingQueryOptions, leaderboardMoversQueryOptions, leaderboardQueryOptions, playerStandingQueryOptions } from "#/lib/api/user";
import { type LeaderboardInterval, type LeaderboardSort, PAGE_SIZE, type Ranking } from "./constants";
import { resolveCatalogItem, toCatalogItem } from "./inventory.helpers";
import type { ICatalogItem } from "./inventory.types";
import { rowFromItem, rowFromScore } from "./rows";
import type { IRankedRow } from "./types";

export interface ILeaderboardDataInput {
    ranking: Ranking;
    /** The score column to rank by; read only when `ranking.kind === "score"`. */
    sort: LeaderboardSort;
    /** Server code, or `undefined` for all servers. */
    server: string | undefined;
    /** Nickname / uid filter, or `undefined` for none. */
    q: string | undefined;
    interval: LeaderboardInterval;
    movementOnly: boolean;
    offset: number;
    /** The signed-in player, for `isSelf` rows and the standing cards. */
    me: { uid: string; server: string } | null;
}

export interface ILeaderboardData {
    catalog: ICatalogItem[];
    materials: IMaterials | undefined;
    catalogLoading: boolean;
    /** The ranked item, or `null` for a score ranking. */
    currentItem: ICatalogItem | null;
    rows: IRankedRow[];
    /** Rows matching the current filters, the number pagination is over. */
    total: number;
    isLoading: boolean;
    /** Everyone ranked, ignoring the nickname filter, for the hero. */
    population: number | null;
    /** The top score (0..1) or top holding, for the hero and the row bars. */
    topValue: number | null;
    heroLoading: boolean;
    /** Score snapshot time; `null` for an item ranking, which has none. */
    updatedAt: string | null;
    movers: ILeaderboardMover[];
    moversLoading: boolean;
    scoreStanding: IPlayerStanding | null;
    itemStanding: IItemStanding | null;
    itemStandingLoading: boolean;
}

/**
 * Every query behind the leaderboard page. The catalog and item names are
 * fetched whichever ranking is active, because the picker lists items either
 * way; the score and item page queries are exclusive, so switching ranking
 * costs one request set, not both.
 */
export function useLeaderboardData({ ranking, sort, server, q, interval, movementOnly, offset, me }: ILeaderboardDataInput): ILeaderboardData {
    const byItem = ranking.kind === "item";
    const itemId = byItem ? ranking.item : "";
    const signedIn = Boolean(me?.uid && me?.server);

    const materialsQuery = useQuery(materialsQueryOptions());
    const catalogQuery = useQuery(itemCatalogQueryOptions(server));

    const scorePageQuery = useQuery({
        ...leaderboardQueryOptions({ server, sort, movement_interval: interval, movement_only: movementOnly, q, limit: PAGE_SIZE, offset }),
        enabled: !byItem,
    });
    const scoreTopQuery = useQuery({ ...leaderboardQueryOptions({ server, limit: 1, offset: 0 }), enabled: !byItem });
    const moversQuery = useQuery({ ...leaderboardMoversQueryOptions({ direction: "up", interval, limit: 3, server }), enabled: !byItem });
    const scoreStandingQuery = useQuery({
        ...playerStandingQueryOptions({ uid: me?.uid ?? "", server: me?.server ?? "", interval }),
        enabled: !byItem && signedIn,
    });

    const itemPageQuery = useQuery({ ...itemLeaderboardQueryOptions({ item: itemId, server, q, limit: PAGE_SIZE, offset }), enabled: byItem });
    // The top holding anchors every row's bar and the hero, so it is fetched
    // once per item rather than read off whichever page or search is open.
    const itemTopQuery = useQuery({ ...itemLeaderboardQueryOptions({ item: itemId, server, limit: 1, offset: 0 }), enabled: byItem });
    const itemStandingQuery = useQuery({
        ...itemStandingQueryOptions({ item: itemId, uid: me?.uid ?? "", server: me?.server ?? "" }),
        enabled: byItem && signedIn,
    });

    const catalog = useMemo<ICatalogItem[]>(() => (catalogQuery.data ?? []).map((row) => toCatalogItem(row, materialsQuery.data)), [catalogQuery.data, materialsQuery.data]);
    const currentItem = useMemo<ICatalogItem | null>(() => {
        if (!byItem) return null;
        const counts = { holders: itemTopQuery.data?.total ?? 0, top: itemTopQuery.data?.entries[0]?.quantity ?? 0 };
        return resolveCatalogItem(catalog, itemId, materialsQuery.data, counts);
    }, [byItem, catalog, itemId, itemTopQuery.data, materialsQuery.data]);

    const rows = useMemo<IRankedRow[]>(() => {
        const isSelf = (uid: string, srv: string) => Boolean(me && uid === me.uid && srv === me.server);
        return byItem ? (itemPageQuery.data?.entries ?? []).map((e) => rowFromItem(e, isSelf)) : (scorePageQuery.data?.entries ?? []).map((e) => rowFromScore(e, sort, isSelf));
    }, [byItem, itemPageQuery.data?.entries, scorePageQuery.data?.entries, sort, me]);

    const pageQuery = byItem ? itemPageQuery : scorePageQuery;
    const topQuery = byItem ? itemTopQuery : scoreTopQuery;

    return {
        catalog,
        materials: materialsQuery.data,
        catalogLoading: catalogQuery.isLoading || materialsQuery.isLoading,
        currentItem,
        rows,
        total: pageQuery.data?.total ?? 0,
        isLoading: pageQuery.isLoading || pageQuery.isFetching,
        population: topQuery.data?.total ?? null,
        topValue: byItem ? (itemTopQuery.data?.entries[0]?.quantity ?? null) : (scoreTopQuery.data?.entries[0]?.total_score ?? null),
        heroLoading: topQuery.isLoading,
        updatedAt: byItem ? null : (scorePageQuery.data?.updated_at ?? null),
        movers: moversQuery.data ?? [],
        moversLoading: moversQuery.isLoading,
        scoreStanding: scoreStandingQuery.data ?? null,
        itemStanding: itemStandingQuery.data ?? null,
        itemStandingLoading: itemStandingQuery.isLoading,
    };
}
