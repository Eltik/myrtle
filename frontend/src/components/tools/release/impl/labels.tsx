import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { humanizeTag } from "./helpers";
import type { messages } from "./labels.messages";

type LabelsT = TypedT<typeof messages>;

/**
 * Turns a raw gamedata tag into something a player recognises.
 *
 * The planner shows three different vocabularies through one `<Tag>`: an
 * event's `activityType`, a banner's `ruleType`, and the calendar's own literal
 * tags. All three went through `humanizeTag`, which lowercases and swaps
 * underscores and nothing else, so the strip read "TYPE ACTIVITYSIDESTORY",
 * "TYPE ACT9D0" and "MULTIPLAY V3".
 *
 * Event types are the hard half: the per-event codes (`TYPE_ACT46SIDE`,
 * `TYPE_ACT9D0`) name a content TEMPLATE, one per event, so no table can
 * enumerate them. They are classified by the suffix the game reuses across
 * them instead, which is why the matching below is ordered and pattern-based
 * rather than a lookup.
 *
 * Anything unrecognised falls through to `humanizeTag`, so this can only ever
 * improve a label, never lose one.
 */
export function useReleaseTagLabel(): (raw: string | null | undefined) => string {
    const t: LabelsT = useT("tools");

    // Literal call sites, not a table of keys: the extractor matches literal
    // t() arguments and cannot follow an index into a constant.
    const L = {
        sideStory: t("release.tag.sideStory"),
        miniStory: t("release.tag.miniStory"),
        intermezzi: t("release.tag.intermezzi"),
        vignette: t("release.tag.vignette"),
        mainStory: t("release.tag.mainStory"),
        rerun: t("release.tag.rerun"),
        rogueLike: t("release.tag.rogueLike"),
        sandbox: t("release.tag.sandbox"),
        contingency: t("release.tag.contingency"),
        vectorBreak: t("release.tag.vectorBreak"),
        bossRush: t("release.tag.bossRush"),
        enemyDuel: t("release.tag.enemyDuel"),
        autoChess: t("release.tag.autoChess"),
        arcade: t("release.tag.arcade"),
        minigame: t("release.tag.minigame"),
        collection: t("release.tag.collection"),
        signIn: t("release.tag.signIn"),
        campaign: t("release.tag.campaign"),
        newSkins: t("release.tag.newSkins"),
        fashionReview: t("release.tag.fashionReview"),
        bannerLimited: t("release.tag.banner.limited"),
        bannerSingle: t("release.tag.banner.single"),
        bannerDouble: t("release.tag.banner.double"),
        bannerNormal: t("release.tag.banner.normal"),
        bannerLinkage: t("release.tag.banner.linkage"),
        bannerClassic: t("release.tag.banner.classic"),
        bannerAttain: t("release.tag.banner.attain"),
        bannerClassicAttain: t("release.tag.banner.classicAttain"),
        bannerFesClassic: t("release.tag.banner.fesClassic"),
    };

    return (raw) => {
        if (!raw) return "";
        // `TYPE_` is a gamedata prefix on every activity type and carries no
        // information: it is the single largest contributor to the noise.
        const key = raw
            .trim()
            .toUpperCase()
            .replace(/\s+/g, "_")
            .replace(/^TYPE_/, "");

        const exact: Record<string, string> = {
            // Banner rule types.
            LIMITED: L.bannerLimited,
            SINGLE: L.bannerSingle,
            DOUBLE: L.bannerDouble,
            NORMAL: L.bannerNormal,
            LINKAGE: L.bannerLinkage,
            CLASSIC: L.bannerClassic,
            ATTAIN: L.bannerAttain,
            CLASSIC_ATTAIN: L.bannerClassicAttain,
            FESCLASSIC: L.bannerFesClassic,
            SPECIAL: L.bannerAttain,
            // Calendar's own literal tags.
            NEW_SKINS: L.newSkins,
            FASHION_REVIEW: L.fashionReview,
            RERUN: L.rerun,
            // Activity categories that are their own name.
            ACTIVITY_SIDESTORY: L.sideStory,
            ACTIVITYSIDESTORY: L.sideStory,
            ACTIVITY_MINISTORY: L.miniStory,
            ACTIVITYMINISTORY: L.miniStory,
            MAINLINE: L.mainStory,
            MAINLINE_BP: L.mainStory,
            MINISTORY: L.miniStory,
            SIDESTORY: L.sideStory,
            BOSS_RUSH: L.bossRush,
            ENEMY_DUEL: L.enemyDuel,
            ARCADE: L.arcade,
            COLLECTION: L.collection,
            CHECKIN_ONLY: L.signIn,
            CHECKIN_ALL_PLAYER: L.signIn,
            LOGIN_ONLY: L.signIn,
            SWITCH_ONLY: L.campaign,
            GRID_GACHA: L.campaign,
            GRID_GACHA_V2: L.campaign,
            FLOAT_PARADE: L.minigame,
            TEAM_QUEST: L.minigame,
            INTERLOCK: L.minigame,
            FIREWORK: L.minigame,
            PRAY_ONLY: L.campaign,
            APRIL_FOOL: L.minigame,
        };
        const hit = exact[key];
        if (hit) return hit;

        // Ordered prefix rules for the families. `MULTIPLAY*` is Contingency
        // Contract / its multiplayer successors; `VEC_BREAK*` and the rest keep
        // their own names.
        if (key.startsWith("MULTIPLAY")) return L.contingency;
        if (key.startsWith("VEC_BREAK")) return L.vectorBreak;
        if (key.startsWith("AUTOCHESS")) return L.autoChess;
        if (key.startsWith("HALFIDLE")) return L.minigame;
        if (key.startsWith("ROGUELIKE")) return L.rogueLike;
        if (key.startsWith("SANDBOX") || key.startsWith("RECLAMATION")) return L.sandbox;
        if (key.startsWith("MAINLINE")) return L.mainStory;
        if (key.startsWith("CHECKIN") || key.startsWith("LOGIN") || key.startsWith("SIGNIN")) return L.signIn;

        // The per-event template codes. `ACT<n><suffix>`, where the suffix is
        // reused across events even though the number is not: SIDE and D0 are
        // side stories, D5 and REP are the reruns of them, MINI is a mini
        // story, VN a vignette, IME the Intermezzi line.
        const act = /^ACT\d+([A-Z0-9]*)$/.exec(key);
        if (act) {
            const suffix = act[1] ?? "";
            if (suffix.includes("REP") || suffix.startsWith("D5")) return L.rerun;
            if (suffix.includes("MINI")) return L.miniStory;
            if (suffix.includes("VN")) return L.vignette;
            if (suffix.includes("IME")) return L.intermezzi;
            if (suffix.includes("SIDE") || suffix.startsWith("D0") || suffix.startsWith("D1")) return L.sideStory;
            if (suffix.includes("FUN")) return L.minigame;
            // An unrecognised suffix is NOT guessed at. Calling it a side story
            // would be a category claim the code cannot support; the caller
            // gets the de-prefixed code instead, which is still strictly less
            // noise than before.
        }

        // `key`, not `raw`: the `TYPE_` prefix is dropped even when nothing
        // else matched, so the fallback is never worse than what it replaced.
        return humanizeTag(key);
    };
}
