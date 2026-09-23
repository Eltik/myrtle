import type { IUserProfile } from "#/types/user";

export type DisplayUser = Pick<IUserProfile, "uid" | "nickname" | "level" | "avatar_id" | "server" | "grade" | "total_score" | "operator_count" | "skin_count"> & {
    /** Value of the active sort for this row; null under the `score` sort. */
    metric: number | null;
};
