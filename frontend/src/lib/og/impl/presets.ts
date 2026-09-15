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
     * Which section chip the card highlights. NOT display text - it is matched
     * against {@link DEFAULT_OG_TAGS} by identity, and the visible label comes
     * from `og.tag.*`, so this stays English however the card is translated.
     *
     * Deliberately a plain string rather than the chip union: `stats` has
     * always carried `Stats`, which is not one of the five chips, so that card
     * highlights nothing. Narrowing the type here would silently change the
     * rendered card, which is a separate decision from this one.
     */
    activeTag?: string;
}

/**
 * The section chips every default card draws, in order. Ids, not labels: see
 * `IDefaultOgPreset.activeTag`.
 */
export const DEFAULT_OG_TAGS = ["Home", "Collection", "Players", "Gacha", "Tools"] as const;

export const DEFAULT_OG_PRESETS = {
    _root: {
        titleKey: "og.root.title",
    },
    home: {
        titleKey: "og.home.title",
        subtitleKey: "og.home.subtitle",
        activeTag: "Home",
    },
    login: {
        titleKey: "og.login.title",
        subtitleKey: "og.login.subtitle",
        activeTag: "Players",
    },
    settings: {
        titleKey: "og.settings.title",
        subtitleKey: "og.settings.subtitle",
        activeTag: "Players",
    },
    operators: {
        titleKey: "og.operators.title",
        subtitleKey: "og.operators.subtitle",
        activeTag: "Collection",
    },
    enemies: {
        titleKey: "og.enemies.title",
        subtitleKey: "og.enemies.subtitle",
        activeTag: "Collection",
    },
    stages: {
        titleKey: "og.stages.title",
        subtitleKey: "og.stages.subtitle",
        activeTag: "Collection",
    },
    stats: {
        titleKey: "og.stats.title",
        subtitleKey: "og.stats.subtitle",
        activeTag: "Stats",
    },
    "user-search": {
        titleKey: "og.userSearch.title",
        subtitleKey: "og.userSearch.subtitle",
        activeTag: "Players",
    },
    "user-leaderboard": {
        titleKey: "og.userLeaderboard.title",
        subtitleKey: "og.userLeaderboard.subtitle",
        activeTag: "Players",
    },
    "gacha-community": {
        titleKey: "og.gachaCommunity.title",
        subtitleKey: "og.gachaCommunity.subtitle",
        activeTag: "Gacha",
    },
    "gacha-history": {
        titleKey: "og.gachaHistory.title",
        subtitleKey: "og.gachaHistory.subtitle",
        activeTag: "Gacha",
    },
    "tools-dps": {
        titleKey: "og.toolsDps.title",
        subtitleKey: "og.toolsDps.subtitle",
        activeTag: "Tools",
    },
    "tools-hps": {
        titleKey: "og.toolsHps.title",
        subtitleKey: "og.toolsHps.subtitle",
        activeTag: "Tools",
    },
    "tools-recruitment": {
        titleKey: "og.toolsRecruitment.title",
        subtitleKey: "og.toolsRecruitment.subtitle",
        activeTag: "Tools",
    },
    "tools-planner": {
        titleKey: "og.toolsPlanner.title",
        subtitleKey: "og.toolsPlanner.subtitle",
        activeTag: "Tools",
    },
    "tools-randomizer": {
        titleKey: "og.toolsRandomizer.title",
        subtitleKey: "og.toolsRandomizer.subtitle",
        activeTag: "Tools",
    },
    "tools-birthdays": {
        titleKey: "og.toolsBirthdays.title",
        subtitleKey: "og.toolsBirthdays.subtitle",
        activeTag: "Tools",
    },
    "tools-release": {
        titleKey: "og.toolsRelease.title",
        subtitleKey: "og.toolsRelease.subtitle",
        activeTag: "Tools",
    },
    "tier-lists": {
        titleKey: "og.tierLists.title",
        subtitleKey: "og.tierLists.subtitle",
        activeTag: "Home",
    },
    terms: {
        titleKey: "og.terms.title",
        subtitleKey: "og.terms.subtitle",
        activeTag: "Home",
    },
    privacy: {
        titleKey: "og.privacy.title",
        subtitleKey: "og.privacy.subtitle",
        activeTag: "Home",
    },
    changelog: {
        titleKey: "og.changelog.title",
        subtitleKey: "og.changelog.subtitle",
        activeTag: "Home",
    },
} as const satisfies Record<string, IDefaultOgPreset>;

export type DefaultOgPresetSlug = keyof typeof DEFAULT_OG_PRESETS;

/**
 * `Home` -> `og.tag.home`. The chip ids are the English labels, so the key is
 * derived rather than restated.
 */
function tagKey(tag: (typeof DEFAULT_OG_TAGS)[number]): string {
    return `og.tag.${tag.toLowerCase()}`;
}

/**
 * The five chip labels for one locale, keyed by chip id. The template matches
 * `activeTag` against the id and renders the label from this.
 */
export function defaultOgTagLabels(source?: IMetaSource | null): Record<string, string> {
    const labels: Record<string, string> = {};
    for (const tag of DEFAULT_OG_TAGS) labels[tag] = metaMessage(tagKey(tag), source);
    return labels;
}

/** Turn a preset's keys into the text the default template renders. */
export function resolveDefaultOgPreset(preset: IDefaultOgPreset, source?: IMetaSource | null): IDefaultOgData {
    return {
        title: metaMessage(preset.titleKey, source),
        subtitle: preset.subtitleKey === undefined ? undefined : metaMessage(preset.subtitleKey, source),
        activeTag: preset.activeTag,
        tagLabels: defaultOgTagLabels(source),
    };
}

/** The preset a slug names, or `null` for an id that is not a preset at all. */
export function defaultOgPreset(slug: string): IDefaultOgPreset | null {
    return (DEFAULT_OG_PRESETS as Record<string, IDefaultOgPreset>)[slug] ?? null;
}
