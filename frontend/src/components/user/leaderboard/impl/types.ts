import type { ILeaderboardEntry } from "#/lib/api/user";

export type LeaderboardEntry = ILeaderboardEntry & {
    isSelf?: boolean;
};
