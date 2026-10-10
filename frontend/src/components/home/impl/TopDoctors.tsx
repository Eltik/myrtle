import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { toPct } from "#/components/user/leaderboard/impl/constants";
import { playerIdentity } from "#/components/user/leaderboard/impl/identity";
import { leaderboardQueryOptions } from "#/lib/api/user";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { DEFAULT_AVATAR_ID } from "#/lib/utils";
import styles from "./Digest.module.css";
import { useReveal } from "./motion";
import shared from "./shared.module.css";
import type { messages } from "./TopDoctors.messages";

/** The home page's leaderboard slice. Exported for the route loader, so SSR and the client share one cache entry. */
export const TOP_DOCTORS_QUERY = { sort: "total_score", limit: 5 } as const;

/** Widened to `string` like the header's nav links: the route's search params all have defaults, so no search is passed. */
const LEADERBOARD_HREF: string = "/user/leaderboard";

export default function TopDoctors() {
    const t: TypedT<typeof messages> = useT("home");
    const reveal = useReveal<HTMLDivElement>();
    const { data, isPending, isError } = useQuery(leaderboardQueryOptions(TOP_DOCTORS_QUERY));
    const entries = data?.entries ?? [];

    return (
        <div {...reveal} className={`${shared.reveal} ${styles.panel}`}>
            <div className={styles.head}>
                <div className={styles.headText}>
                    <span className={shared.eyebrow}>{t("leaders.eyebrow")}</span>
                    <h2 className={styles.title}>{t("leaders.title")}</h2>
                </div>
                <Link to={LEADERBOARD_HREF} className={shared.moreLink}>
                    {t("leaders.seeAll")}
                </Link>
            </div>
            <div className={styles.list}>
                {isError && <p className={styles.note}>{t("leaders.error")}</p>}
                {!isPending && !isError && entries.length === 0 && <p className={styles.note}>{t("leaders.empty")}</p>}
                {entries.map((entry) => {
                    const { nickname } = playerIdentity(entry);
                    return (
                        <Link key={`${entry.server}:${entry.uid}`} to="/user/$id" params={{ id: entry.uid }} className={styles.leader} aria-label={t("leaders.viewProfile", { nickname })}>
                            <span className={styles.rank}>{entry.rank_global ?? "-"}</span>
                            <span className={styles.avatar}>
                                <OperatorAvatar charId={entry.avatar_id ?? DEFAULT_AVATAR_ID} name={nickname} />
                            </span>
                            <span className={styles.who}>
                                <span className={styles.nameLine}>
                                    <span className={styles.nickname}>{nickname}</span>
                                    {entry.nick_number && <span className={styles.nickNumber}>#{entry.nick_number}</span>}
                                </span>
                                <span className={styles.meta}>{t("leaders.meta", { server: entry.server.toUpperCase(), level: entry.level ?? "-" })}</span>
                            </span>
                            <span className={styles.score}>
                                <span className={styles.scoreValue}>{entry.total_score == null ? "-" : toPct(entry.total_score).toFixed(1)}</span>
                                <span className={styles.grade}>{entry.grade ?? "-"}</span>
                            </span>
                        </Link>
                    );
                })}
            </div>
        </div>
    );
}
