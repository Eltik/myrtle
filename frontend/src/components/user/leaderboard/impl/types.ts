import type { ILeaderboardEntry } from "#/lib/api/user";

export type LeaderboardEntry = ILeaderboardEntry & {
    isSelf?: boolean;
};

/**
 * One table row whichever metric ranks it. Score rows carry a 0..1 `value`
 * and a rank `delta`; item rows carry a quantity and no delta, because item
 * holdings are not snapshotted.
 */
export interface IRankedRow {
    uid: string;
    nickname: string | null;
    nick_number: string | null;
    level: number | null;
    avatar_id: string | null;
    server: string;
    grade: string | null;
    rank: number | null;
    delta: number | null;
    value: number | null;
    isSelf: boolean;
}
