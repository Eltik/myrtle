import type { IItemLeaderboardEntry, ILeaderboardEntry } from "#/lib/api/user";
import type { LeaderboardSort } from "./constants";
import type { IRankedRow } from "./types";

type IsSelf = (uid: string, server: string) => boolean;

/** The identity columns every ranking shares. */
function playerColumns(entry: ILeaderboardEntry | IItemLeaderboardEntry, isSelf: IsSelf) {
    return {
        uid: entry.uid,
        nickname: entry.nickname,
        nick_number: entry.nick_number,
        level: entry.level,
        avatar_id: entry.avatar_id,
        server: entry.server,
        grade: entry.grade,
        isSelf: isSelf(entry.uid, entry.server),
    };
}

/** A score row: the ranked value is the chosen score column, 0..1, and the
 * rank carries its movement against the snapshot baseline. */
export function rowFromScore(entry: ILeaderboardEntry, sort: LeaderboardSort, isSelf: IsSelf): IRankedRow {
    return { ...playerColumns(entry, isSelf), rank: entry.rank_global, delta: entry.rank_delta, value: entry[sort] };
}

/** An item row: the ranked value is a quantity; holdings are not
 * snapshotted, so there is no movement. */
export function rowFromItem(entry: IItemLeaderboardEntry, isSelf: IsSelf): IRankedRow {
    return { ...playerColumns(entry, isSelf), rank: entry.rank, delta: null, value: entry.quantity };
}
