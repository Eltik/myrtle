import { type IMetaSource, metaMessage } from "#/lib/meta";
import type { IDefaultOgData } from "./templates/Default";

/**
 * One default social card per page, as `meta` message keys rather than
 * literals.
 *
 * The text is resolved twice, in two places that cannot share a React context:
 * on the page, by `defaultOgURL()` inside a route's `head()`, to hash the card
 * into its `og:image` URL; and on the server, by the `default` OG handler,
 * when that URL is requested and the PNG is not cached. Both go through
 * `resolveDefaultOgPreset` so the two can never disagree about what the card
 * says. See `presets.messages.ts` for the English text.
 */
export interface IDefaultOgPreset {
    /** A `meta` key, e.g. `og.operators.title`. */
    titleKey: string;
    subtitleKey?: string;
    /**
     * The site section the card names in its kicker. Its label is the
     * `og.tag.<section>` message, and it picks the card's art (see
     * {@link DEFAULT_OG_SECTION_ART}), so every page of a section looks alike.
     */
    section?: DefaultOgSection;
}

/** The sections a default card can belong to. Ids, not labels: the label is `og.tag.<id>`. */
export type DefaultOgSection = "home" | "collection" | "players" | "community" | "gacha" | "tools" | "stats" | "account" | "legal";

/** The mascot art a default card draws, by file under `public/home/`. */
export type DefaultOgArt = "hero" | "happy" | "sleepy" | "fall";

/**
 * Which art each section draws, so a card's look is fixed by its slug. The
 * home and browsing sections get the full key visual; the tools the flag
 * waving pair; gacha the dropped pull; the account and legal pages the
 * sleeping one.
 */
export const DEFAULT_OG_SECTION_ART: Record<DefaultOgSection, DefaultOgArt> = {
    home: "hero",
    collection: "hero",
    community: "hero",
    stats: "hero",
    players: "happy",
    tools: "happy",
    gacha: "fall",
    account: "sleepy",
    legal: "sleepy",
};

export const DEFAULT_OG_PRESETS = {
    _root: {
        titleKey: "og.root.title",
        section: "home",
    },
    home: {
        titleKey: "og.home.title",
        subtitleKey: "og.home.subtitle",
        section: "home",
    },
    login: {
        titleKey: "og.login.title",
        subtitleKey: "og.login.subtitle",
        section: "account",
    },
    settings: {
        titleKey: "og.settings.title",
        subtitleKey: "og.settings.subtitle",
        section: "account",
    },
    operators: {
        titleKey: "og.operators.title",
        subtitleKey: "og.operators.subtitle",
        section: "collection",
    },
    enemies: {
        titleKey: "og.enemies.title",
        subtitleKey: "og.enemies.subtitle",
        section: "collection",
    },
    stages: {
        titleKey: "og.stages.title",
        subtitleKey: "og.stages.subtitle",
        section: "collection",
    },
    stats: {
        titleKey: "og.stats.title",
        subtitleKey: "og.stats.subtitle",
        section: "stats",
    },
    "user-search": {
        titleKey: "og.userSearch.title",
        subtitleKey: "og.userSearch.subtitle",
        section: "players",
    },
    "user-leaderboard": {
        titleKey: "og.userLeaderboard.title",
        subtitleKey: "og.userLeaderboard.subtitle",
        section: "players",
    },
    "gacha-community": {
        titleKey: "og.gachaCommunity.title",
        subtitleKey: "og.gachaCommunity.subtitle",
        section: "gacha",
    },
    "gacha-history": {
        titleKey: "og.gachaHistory.title",
        subtitleKey: "og.gachaHistory.subtitle",
        section: "gacha",
    },
    "tools-dps": {
        titleKey: "og.toolsDps.title",
        subtitleKey: "og.toolsDps.subtitle",
        section: "tools",
    },
    "tools-hps": {
        titleKey: "og.toolsHps.title",
        subtitleKey: "og.toolsHps.subtitle",
        section: "tools",
    },
    "tools-squad-builder": {
        titleKey: "og.toolsSquadBuilder.title",
        subtitleKey: "og.toolsSquadBuilder.subtitle",
        section: "tools",
    },
    "tools-recruitment": {
        titleKey: "og.toolsRecruitment.title",
        subtitleKey: "og.toolsRecruitment.subtitle",
        section: "tools",
    },
    "tools-planner": {
        titleKey: "og.toolsPlanner.title",
        subtitleKey: "og.toolsPlanner.subtitle",
        section: "tools",
    },
    "tools-randomizer": {
        titleKey: "og.toolsRandomizer.title",
        subtitleKey: "og.toolsRandomizer.subtitle",
        section: "tools",
    },
    "tools-birthdays": {
        titleKey: "og.toolsBirthdays.title",
        subtitleKey: "og.toolsBirthdays.subtitle",
        section: "tools",
    },
    "tools-release": {
        titleKey: "og.toolsRelease.title",
        subtitleKey: "og.toolsRelease.subtitle",
        section: "tools",
    },
    "tier-lists": {
        titleKey: "og.tierLists.title",
        subtitleKey: "og.tierLists.subtitle",
        section: "community",
    },
    grids: {
        titleKey: "og.grids.title",
        subtitleKey: "og.grids.subtitle",
        section: "community",
    },
    terms: {
        titleKey: "og.terms.title",
        subtitleKey: "og.terms.subtitle",
        section: "legal",
    },
    privacy: {
        titleKey: "og.privacy.title",
        subtitleKey: "og.privacy.subtitle",
        section: "legal",
    },
    changelog: {
        titleKey: "og.changelog.title",
        subtitleKey: "og.changelog.subtitle",
        section: "home",
    },
    music: {
        titleKey: "og.music.title",
        subtitleKey: "og.music.subtitle",
        section: "collection",
    },
    stories: {
        titleKey: "og.stories.title",
        subtitleKey: "og.stories.subtitle",
        section: "collection",
    },
    archives: {
        titleKey: "og.archives.title",
        subtitleKey: "og.archives.subtitle",
        section: "collection",
    },
} as const satisfies Record<string, IDefaultOgPreset>;

export type DefaultOgPresetSlug = keyof typeof DEFAULT_OG_PRESETS;

/** Turn a preset's keys into the text the default template renders. */
export function resolveDefaultOgPreset(preset: IDefaultOgPreset, source?: IMetaSource | null): IDefaultOgData {
    const section = preset.section ?? "home";
    return {
        title: metaMessage(preset.titleKey, source),
        subtitle: preset.subtitleKey === undefined ? undefined : metaMessage(preset.subtitleKey, source),
        section: metaMessage(`og.tag.${section}`, source),
        art: DEFAULT_OG_SECTION_ART[section],
    };
}

/** The card for a one-off page whose id is its own title: the home section's look. */
export function literalDefaultOgData(title: string, source?: IMetaSource | null): IDefaultOgData {
    return { title, section: metaMessage("og.tag.home", source), art: DEFAULT_OG_SECTION_ART.home };
}

/** The preset a slug names, or `null` for an id that is not a preset at all. */
export function defaultOgPreset(slug: string): IDefaultOgPreset | null {
    return (DEFAULT_OG_PRESETS as Record<string, IDefaultOgPreset>)[slug] ?? null;
}
