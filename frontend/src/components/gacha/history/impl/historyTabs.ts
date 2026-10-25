import type { ClientGachaGroup, IClientGachaRecords, IGachaItem } from "#/lib/api/gacha";

/** A Pull History tab: one banner group, or every group merged. */
export type HistoryTab = ClientGachaGroup | "all";

/**
 * Tab order. "All" leads because it is the superset; the page still OPENS on
 * Limited (`DEFAULT_HISTORY_TAB`), which is the landing it had before "All"
 * existed.
 */
export const HISTORY_TABS: readonly HistoryTab[] = ["all", "limited", "linkage", "regular", "special", "boot"];

/**
 * Where the page opens. Limited, as before "All" existed: a long-time
 * account's history is mostly Standard pulls (the bulk of every 10-pull
 * session), so an All landing would push the limited pulls most people open
 * this page to check below several pages of 3★ rows.
 */
export const DEFAULT_HISTORY_TAB: HistoryTab = "limited";

/** Every group, in the order `HISTORY_TABS` lists them, which is the order "all" merges them in. */
export const GROUPS: readonly ClientGachaGroup[] = ["limited", "linkage", "regular", "special", "boot"];

/**
 * Every pull across every group, newest first.
 *
 * The groups are disjoint (`deriveClientGachaRecords` puts each pull in
 * exactly one), so concatenating them double-counts nothing. A 10-pull shares
 * one timestamp across its ten rows, so ties are kept in their group's own
 * order by the stable sort instead of being reshuffled.
 */
export function mergeAllGroups(records: IClientGachaRecords): IGachaItem[] {
    return GROUPS.flatMap((group) => records[group].records).sort((a, b) => b.at - a.at);
}

/** The pulls one tab lists, unsorted for a group (the table sorts) and merged newest first for "all". */
export function recordsForTab(records: IClientGachaRecords, tab: HistoryTab): IGachaItem[] {
    return tab === "all" ? mergeAllGroups(records) : records[tab].records;
}

/** Pulls in one tab before any filter. "all" is the sum, since the groups are disjoint. */
export function totalForTab(records: IClientGachaRecords, tab: HistoryTab): number {
    return tab === "all" ? GROUPS.reduce((sum, group) => sum + records[group].total, 0) : records[tab].total;
}

/**
 * Whether a tab is offered. Starter Headhunting is a one-time pool most
 * accounts never pulled on, so its tab shows only when it holds pulls.
 */
export function isTabShown(records: IClientGachaRecords, tab: HistoryTab): boolean {
    return tab !== "boot" || records.boot.total > 0;
}
