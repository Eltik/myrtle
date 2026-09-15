import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Shared chrome for the release planner.
 *
 * The `AutoTag` labels explain where a guessed English name came from - the
 * name itself is game data. A row whose name is still only in Chinese keeps
 * its `lang="zh-CN"` marker and its raw Chinese text: that marker is the
 * planner's source-language signal for the reader's own translator, not a
 * translation gap.
 */
export const namespace = "tools";

export const messages = {
    "release.auto.memory": {
        text: "matched",
        description: "Tag on a name this site matched to an English one it already knew. Very small tag; keep it to one word, lowercase.",
    },
    "release.auto.memory.title": {
        text: "Same name under an id both servers share, or the EN name of this event's original run",
        description: "Tooltip explaining the 'matched' tag. 'EN' is the English game server and stays as-is.",
    },
    "release.auto.appellation": {
        text: "codename",
        description: "Tag on a name taken from the operator's Latin-script appellation. Very small tag; keep it to one word, lowercase.",
    },
    "release.auto.appellation.title": {
        text: "The operator's shipped Latin name",
        description: "Tooltip explaining the 'codename' tag: the Latin-script name the game itself ships alongside the Chinese one.",
    },
    "release.auto.override": {
        text: "entered",
        description: "Tag on a name a person typed into this site's override table. Very small tag; keep it to one word, lowercase.",
    },
    "release.auto.override.title": {
        text: "An English name entered on this row's override",
        description: "Tooltip explaining the 'entered' tag.",
    },
    "release.sectionCount": {
        text: "· {count}",
        description: "Row count after a section heading, e.g. '· 12'. Keep the leading middle dot.",
    },
    "release.loadFailed": {
        text: "Failed to load.",
        description: "Fallback error line when the request failed with no message of its own.",
    },
    "release.retry": {
        text: "Retry",
        description: "Button that tries the failed request again.",
    },
} satisfies MessageMap;

// `dynamic`: the AutoTag label and tooltip keys are stored in a lookup table
// and resolved as `t(AUTO_TAG_KEYS[source].label)`, so the
// extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
