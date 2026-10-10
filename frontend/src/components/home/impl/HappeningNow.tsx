import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import type * as React from "react";
import { useGamedataServerName } from "#/components/GamedataServerSwitcher";
import { itemIcon } from "#/components/operators/detail/impl/assets";
import { useArt } from "#/components/tools/release/impl/components/shared";
import { fromUnix } from "#/components/tools/release/impl/helpers";
import { useReleaseTagLabel } from "#/components/tools/release/impl/labels";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { DEFAULT_GAMEDATA_SERVER, resolveGamedataServer } from "#/lib/api/gamedata";
import { liveFeedQueryOptions } from "#/lib/api/live";
import { useGamedataServer, useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { languageOf } from "#/lib/story/book/book";
import type { messages } from "./HappeningNow.messages";
import styles from "./HappeningNow.module.css";
import { countdownParts, elapsedPercent, LIVE_LIMIT, type LiveItem, namedFeatured, selectLive, showsBannerPlaceholder } from "./live";
import { useNow, useReveal } from "./motion";
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

/** Cards reveal in a 60 ms stagger that restarts every fourth card. */
const CARD_STAGGER_MS = 60;
const CARD_STAGGER_PERIOD = 4;

export default function HappeningNow({ id }: { id: string }) {
    const t: LiveT = useT("home");
    const server = useGamedataServer();
    const serverName = useGamedataServerName()(server);
    const feed = useQuery(liveFeedQueryOptions(server));
    const now = useNow();
    const revealHead = useReveal<HTMLDivElement>();

    const loading = feed.isPending;
    const failed = !loading && feed.isError && !feed.data;
    // Re-selected every tick: a window that closes while the page is open drops
    // out, and the next one in the feed takes its place.
    const items = selectLive(feed.data?.items ?? [], now / 1000);
    const names: FeaturedNames = feed.data?.names ?? {};
    const cards = loading ? SKELETON_KEYS.map((key) => <div key={key} className={styles.skeleton} aria-hidden="true" />) : items.map((item, i) => <LiveCard key={item.key} item={item} index={i} now={now} server={server} names={names} />);

    return (
        <section id={id} className={`${shared.container} ${styles.section}`}>
            <div {...revealHead} className={`${shared.reveal} ${styles.head}`}>
                <div className={styles.headText}>
                    <span className={shared.eyebrow}>{t("live.eyebrow", { server: serverName })}</span>
                    <h2 className={styles.title}>{t("live.title")}</h2>
                </div>
                {/* The community page's banner list opens on "Active" and reads the same server. */}
                <Link to="/gacha/community" hash="banner-runs" className={shared.moreLink}>
                    {t("live.seeAll")}
                </Link>
            </div>

            {failed && <div className={styles.empty}>{t("live.error", { server: serverName })}</div>}
            {!loading && !failed && items.length === 0 && (
                <div className={`${styles.empty} ${styles.emptyRow}`}>
                    <span>{t("live.empty", { server: serverName })}</span>
                    {/* The release planner forecasts the default server only. */}
                    {resolveGamedataServer(server) === DEFAULT_GAMEDATA_SERVER && (
                        <Link to="/tools/release" className={shared.moreLink}>
                            {t("live.upcoming")}
                        </Link>
                    )}
                </div>
            )}

            <div className={styles.grid}>{cards}</div>
        </section>
    );
}

interface ILiveCardProps {
    item: LiveItem;
    index: number;
    now: number;
    server: string;
    /** Featured operators' display names, resolved server-side; an id missing here shows as itself. */
    names: FeaturedNames;
}

function LiveCard({ item, index, now, server, names: featuredNames }: ILiveCardProps): React.ReactElement {
    const t: LiveT = useT("home");
    const locale = useLocale();
    const tagLabel = useReleaseTagLabel();
    const reveal = useReveal<HTMLAnchorElement>((index % CARD_STAGGER_PERIOD) * CARD_STAGGER_MS);
    // Each art path in turn, then the faces: a rerun's own art, then its side story's.
    const firstArt = useArt(item.artPaths[0]);
    const nextArt = useArt(item.artPaths[1]);
    const art = firstArt.src ? firstArt : nextArt;
    const placeholder = showsBannerPlaceholder(item, Boolean(art.src));

    let kind: string;
    let detail: string;
    let faces: { id: string; name: string }[] = [];

    if (item.kind === "event") {
        kind = tagLabel(item.type);
        detail = t("live.eventDetail", { date: fromUnix(item.end).toLocaleDateString(locale, { month: "short", day: "numeric" }) });
    } else {
        kind = t("live.bannerKind", { kind: tagLabel(item.type) });
        // A pool with no 6★ rate-up still has its 5★ ones worth naming.
        const featured = namedFeatured(item).map((charId) => ({ id: charId, name: featuredNames[charId] ?? charId }));
        const names = featured.map((f) => f.name);
        detail = placeholder ? t("live.bannerUnlisted") : names.length > MAX_LISTED_NAMES ? t("live.bannerMore", { names: names.slice(0, NAMES_BEFORE_MORE).join(", "), count: names.length - NAMES_BEFORE_MORE }) : names.join(", ");
        faces = featured.slice(0, MAX_FACES);
    }

    const left = countdownParts(item.end * 1000 - now);
    const pct = elapsedPercent(item.start, item.end, now / 1000);

    return (
        <Link to="/tools/release" {...reveal} className={`${shared.reveal} ${styles.card}`}>
            {art.src ? (
                <img
                    src={art.src}
                    alt={item.name}
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
                    <img src={PERMIT_ICON} alt="" loading="lazy" decoding="async" className={styles.permit} />
                </div>
            ) : (
                <div className={styles.faces} style={{ "--faces": faces.length } as React.CSSProperties}>
                    {faces.map((face) => (
                        <span key={face.id} className={styles.face} title={face.name}>
                            <OperatorAvatar charId={face.id} name={face.name} server={server} />
                        </span>
                    ))}
                </div>
            )}
            <div className={styles.body}>
                <div className={styles.text}>
                    <div>
                        <span className={shared.tag}>{kind}</span>
                    </div>
                    <span className={styles.name} lang={languageOf(server)}>
                        {item.name}
                    </span>
                    {detail && (
                        <span className={styles.detail} suppressHydrationWarning>
                            {detail}
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
        </Link>
    );
}
