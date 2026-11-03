import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import type * as React from "react";
import { useGamedataServerName } from "#/components/GamedataServerSwitcher";
import { itemIcon } from "#/components/operators/detail/impl/assets";
import { useArt } from "#/components/tools/release/impl/components/shared";
import { fromUnix } from "#/components/tools/release/impl/helpers";
import { useReleaseTagLabel } from "#/components/tools/release/impl/labels";
import { Button } from "#/components/ui/button";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { DEFAULT_GAMEDATA_SERVER, resolveGamedataServer } from "#/lib/api/gamedata";
import { liveFeedQueryOptions } from "#/lib/api/live";
import { useFormatters, useGamedataServer, useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { languageOf } from "#/lib/story/book/book";
import { useNow } from "./clock";
import type { messages } from "./HappeningNow.messages";
import styles from "./HappeningNow.module.css";
import { countdownParts, elapsedPercent, LIVE_LIMIT, type LiveItem, namedFeatured, selectLive, showsBannerPlaceholder } from "./live";
import shared from "./shared.module.css";

type LiveT = TypedT<typeof messages>;
type FeaturedNames = Readonly<Record<string, string>>;

/** One placeholder per card while the server's events and banners load. */
const SKELETON_KEYS = Array.from({ length: LIVE_LIMIT }, (_, i) => `skeleton-${i}`);

/** How many featured operators a banner without art shows as faces. */
const MAX_FACES = 6;

/** A banner lists up to this many featured names in full; past it, the first two plus "+N more". */
const MAX_LISTED_NAMES = 3;
const NAMES_BEFORE_MORE = 2;

/**
 * The Headhunting Permit (item 7003), drawn in a banner's art slot when the pool has
 * neither art nor rate-ups to show. Same icon on every server; the default route
 * serves it (200 on en, jp, kr and cn, 2026-10-10).
 */
const PERMIT_ICON = itemIcon("7003", "TKT_GACHA", null);

export default function HappeningNow({ id }: { id: string }) {
    const t: LiveT = useT("home");
    const server = useGamedataServer();
    const serverName = useGamedataServerName()(server);
    const feed = useQuery(liveFeedQueryOptions(server));
    const now = useNow();
    // The release planner forecasts the default server only, so only there does it cover what a card shows.
    const forecasts = resolveGamedataServer(server) === DEFAULT_GAMEDATA_SERVER;

    const loading = feed.isPending;
    const failed = !loading && feed.isError && !feed.data;
    // Re-selected every tick: a window that closes while the page is open drops
    // out, and the next one in the feed takes its place.
    const items = selectLive(feed.data?.items ?? [], now / 1000);
    const names: FeaturedNames = feed.data?.names ?? {};
    const cards = loading ? SKELETON_KEYS.map((key) => <Skeleton key={key} className={styles.skeleton} />) : items.map((item) => <LiveCard key={item.key} item={item} now={now} server={server} forecasts={forecasts} names={names} />);

    return (
        <section id={id} className={`${shared.container} ${styles.section}`}>
            <div className={styles.head}>
                <div className={styles.headText}>
                    <h2 className={styles.title}>{t("live.title")}</h2>
                </div>
                {/* The community page's banner list opens on "Active" and reads the same server. */}
                <Link to="/gacha/community" hash="banner-runs" className={shared.moreLink}>
                    {t("live.seeAll")}
                </Link>
            </div>

            {failed && (
                <div className={`${styles.empty} ${styles.emptyRow}`}>
                    <span>{t("live.error", { server: serverName })}</span>
                    <Button variant="outline" size="sm" loading={feed.isFetching} onClick={() => void feed.refetch()}>
                        {t("live.retry")}
                    </Button>
                </div>
            )}
            {!loading && !failed && items.length === 0 && (
                <div className={`${styles.empty} ${styles.emptyRow}`}>
                    <span>{t("live.empty", { server: serverName })}</span>
                    {forecasts && (
                        <Link to="/tools/release" className={shared.moreLink}>
                            {t("live.upcoming")}
                        </Link>
                    )}
                </div>
            )}

            {/* Mounted from the first render and only its text changes, so a screen reader announces the load. */}
            <output className="sr-only">{loading ? t("live.loading", { server: serverName }) : ""}</output>
            <div className={styles.grid} aria-busy={loading}>
                {cards}
            </div>
        </section>
    );
}

interface ILiveCardProps {
    item: LiveItem;
    now: number;
    server: string;
    /** Whether the release planner forecasts this server, and so covers the card. */
    forecasts: boolean;
    /** Featured operators' display names, resolved server-side; an id missing here shows as itself. */
    names: FeaturedNames;
}

function LiveCard({ item, now, server, forecasts, names: featuredNames }: ILiveCardProps): React.ReactElement {
    const t: LiveT = useT("home");
    const f = useFormatters();
    const locale = useLocale();
    const tagLabel = useReleaseTagLabel();
    // Each art path in turn, then the faces: a rerun's own art, then its side story's.
    const firstArt = useArt(item.artPaths[0]);
    const nextArt = useArt(item.artPaths[1]);
    const art = firstArt.src ? firstArt : nextArt;
    const placeholder = showsBannerPlaceholder(item, Boolean(art.src));

    let kind: string;
    let detail: string;
    let faces: { id: string; name: string }[] = [];
    // Set when `detail` cuts the featured list short: the whole list, for screen readers.
    let fullNames: string | null = null;

    if (item.kind === "event") {
        kind = tagLabel(item.type);
        // `f.date`, not `toLocaleDateString`: the home page renders on the server, and an end date
        // printed in the server's zone hydrated against the browser's.
        detail = t("live.eventDetail", { date: f.date(fromUnix(item.end), { month: "short", day: "numeric" }) });
    } else {
        kind = t("live.bannerKind", { kind: tagLabel(item.type) });
        // A pool with no 6★ rate-up still has its 5★ ones worth naming.
        const featured = namedFeatured(item).map((charId) => ({ id: charId, name: featuredNames[charId] ?? charId }));
        const names = featured.map((f) => f.name);
        const listed = new Intl.ListFormat(locale, { type: "unit", style: "short" });
        if (placeholder) {
            detail = t("live.bannerUnlisted");
        } else if (names.length > MAX_LISTED_NAMES) {
            detail = t("live.bannerMore", { names: listed.format(names.slice(0, NAMES_BEFORE_MORE)), count: names.length - NAMES_BEFORE_MORE });
            fullNames = new Intl.ListFormat(locale, { type: "conjunction" }).format(names);
        } else {
            detail = listed.format(names);
        }
        faces = featured.slice(0, MAX_FACES);
    }

    const left = countdownParts(item.end * 1000 - now);
    const pct = elapsedPercent(item.start, item.end, now / 1000);

    const content = (
        <>
            {art.src ? (
                <img
                    src={art.src}
                    // The card's text names the event or banner right below.
                    alt=""
                    loading="lazy"
                    className={styles.art}
                    onError={art.onError}
                    // An image that failed before hydration fires no error event React sees,
                    // and most pools 404 here (the path is built, not looked up), so check on mount.
                    ref={(img) => {
                        if (img?.complete && img.naturalWidth === 0 && img.getAttribute("src") === art.src) art.onError();
                    }}
                />
            ) : placeholder ? (
                <div className={styles.placeholder}>
                    <img src={PERMIT_ICON} alt="" width={184} height={184} loading="lazy" decoding="async" className={styles.permit} />
                </div>
            ) : (
                // The faces repeat the names in the detail line, which is what a screen reader gets.
                <div className={styles.faces} style={{ "--faces": faces.length } as React.CSSProperties} aria-hidden="true">
                    {faces.map((face) => (
                        <span key={face.id} className={styles.face}>
                            <OperatorAvatar charId={face.id} name={face.name} server={server} />
                        </span>
                    ))}
                </div>
            )}
            <div className={styles.body}>
                <div className={styles.text}>
                    <div>
                        <span className="kicker">{kind}</span>
                    </div>
                    <span className={styles.name} lang={languageOf(server)}>
                        {item.name}
                    </span>
                    {detail && (
                        <span className={styles.detail}>
                            <span aria-hidden={fullNames ? true : undefined} suppressHydrationWarning>
                                {detail}
                            </span>
                            {fullNames && <span className="sr-only">{fullNames}</span>}
                        </span>
                    )}
                </div>
                <div className={styles.progress}>
                    <div className={styles.track}>
                        <div className={styles.fill} style={{ width: `${pct.toFixed(1)}%` }} suppressHydrationWarning />
                    </div>
                    <span className={styles.countdown} suppressHydrationWarning>
                        {left ? t("live.countdown", { days: left.days, clock: left.clock }) : t("live.ended")}
                    </span>
                </div>
            </div>
        </>
    );

    // The planner covers every card on its server. Elsewhere a banner goes to the community
    // page's banner list, which reads the same server; an event there has no page to open.
    if (forecasts) {
        return (
            <Link to="/tools/release" className={`${styles.card} ${styles.cardLink}`}>
                {content}
            </Link>
        );
    }
    if (item.kind === "banner") {
        return (
            <Link to="/gacha/community" hash="banner-runs" className={`${styles.card} ${styles.cardLink}`}>
                {content}
            </Link>
        );
    }
    return <div className={styles.card}>{content}</div>;
}
