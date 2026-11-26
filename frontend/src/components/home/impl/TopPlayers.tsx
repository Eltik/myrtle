import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import type * as React from "react";
import { Button } from "#/components/ui/button";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { toPct } from "#/components/user/leaderboard/impl/constants";
import { playerIdentity } from "#/components/user/leaderboard/impl/identity";
import { leaderboardQueryOptions } from "#/lib/api/user";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { DEFAULT_AVATAR_ID } from "#/lib/utils";
import styles from "./Digest.module.css";
import shared from "./shared.module.css";
import type { messages } from "./TopPlayers.messages";

/** The home page's leaderboard slice. Exported for the route loader, so SSR and the client share one cache entry. */
export const TOP_PLAYERS_QUERY = { sort: "total_score", limit: 5 } as const;

/** Widened to `string` like the header's nav links: the route's search params all have defaults, so no search is passed. */
const LEADERBOARD_HREF: string = "/user/leaderboard";

/** One placeholder row per player while the leaderboard loads. */
const SKELETON_KEYS = Array.from({ length: TOP_PLAYERS_QUERY.limit }, (_, i) => `skeleton-${i}`);

type LeadersT = TypedT<typeof messages>;

export default function TopPlayers() {
    const t: LeadersT = useT("home");
    const { data, isPending, isError, isFetching, refetch } = useQuery(leaderboardQueryOptions(TOP_PLAYERS_QUERY));
    const entries = data?.entries ?? [];

    return (
        <div className={styles.panel}>
            <div className={styles.head}>
                <div className={styles.headText}>
                    <h2 className={styles.title}>{t("leaders.title")}</h2>
                </div>
                <Link to={LEADERBOARD_HREF} className={shared.moreLink}>
                    {t("leaders.seeAll")}
                </Link>
            </div>
            {/* Mounted from the first render and only its text changes, so a screen reader announces the load. */}
            <output className="sr-only">{isPending ? t("leaders.loading") : ""}</output>
            <div className={styles.list} aria-busy={isPending}>
                {isPending && SKELETON_KEYS.map((key) => <Skeleton key={key} className={styles.leaderSkeleton} />)}
                {isError && (
                    <div className={styles.noteRow}>
                        <p className={styles.note}>{t("leaders.error")}</p>
                        <Button variant="outline" size="sm" loading={isFetching} onClick={() => void refetch()}>
                            {t("leaders.retry")}
                        </Button>
                    </div>
                )}
                {!isPending && !isError && entries.length === 0 && <p className={styles.note}>{t("leaders.empty")}</p>}
                {entries.map((entry) => {
                    const { nickname } = playerIdentity(entry);
                    return (
                        <Link key={`${entry.server}:${entry.uid}`} to="/user/$id" params={{ id: entry.uid }} className={styles.leader}>
                            <span className={styles.rank}>{entry.rank_global ?? <Missing t={t} />}</span>
                            <span className={styles.avatar}>
                                <OperatorAvatar charId={entry.avatar_id ?? DEFAULT_AVATAR_ID} name={nickname} />
                            </span>
                            <span className={styles.who}>
                                <span className={styles.nameLine}>
                                    <span className={styles.nickname}>{nickname}</span>
                                    {entry.nick_number && <span className={styles.nickNumber}>#{entry.nick_number}</span>}
                                </span>
                                {/* With no level, the server code alone: "Lv" and a placeholder say nothing more. */}
                                <span className={styles.meta}>{entry.level == null ? entry.server.toUpperCase() : t("leaders.meta", { server: entry.server.toUpperCase(), level: entry.level })}</span>
                            </span>
                            <span className={styles.score}>
                                <span className={styles.scoreValue}>{entry.total_score == null ? <Missing t={t} /> : toPct(entry.total_score).toFixed(1)}</span>
                                <span className={styles.grade}>{entry.grade ?? <Missing t={t} />}</span>
                            </span>
                        </Link>
                    );
                })}
            </div>
        </div>
    );
}

/** A rank, score or grade the player does not have: a mark on screen, words for a screen reader. */
function Missing({ t }: { t: LeadersT }): React.ReactElement {
    return (
        <>
            <span aria-hidden="true">{t("leaders.missing")}</span>
            <span className="sr-only">{t("leaders.missingLabel")}</span>
        </>
    );
}
