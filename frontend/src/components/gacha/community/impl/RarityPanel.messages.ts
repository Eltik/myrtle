import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "community.rarity.kicker": {
        text: "Outcome mix",
        description: "Small uppercase label above the rarity-distribution panel, used while the pull total is still loading.",
    },
    "community.rarity.kicker.withTotal": {
        text: "Outcome mix · {count} pulls",
        description: "Small uppercase label above the rarity-distribution panel once the total is known. Keep the middle dot.",
    },
    "community.rarity.title": {
        text: "Where every pull lands.",
        description: "Heading of the panel breaking all pulls down by rarity.",
    },
    "community.rarity.you": {
        text: "you · {count} pulls",
        description: "Pill marking the viewer's own column in the rarity panel. Lowercase, rendered uppercase by the style; keep the middle dot.",
    },
    "community.rarity.hint.sixStar": {
        text: "2% base · +2%/pull after 50",
        description: "The advertised 6-star odds: a 2% chance per pull, rising by two points per pull once fifty pulls have passed without one. Keep the middle dot.",
    },
    "community.rarity.hint.fiveStar": {
        text: "8% base · 10-pull guarantee",
        description: "The advertised 5-star odds: an 8% chance per pull, with one guaranteed in the first ten pulls. Keep the middle dot.",
    },
    "community.rarity.hint.fourStar": {
        text: "50% base",
        description: "The advertised 4-star odds: half of all pulls.",
    },
    "community.rarity.hint.threeStar": {
        text: "40% base",
        description: "The advertised 3-star odds: two pulls in five.",
    },
    "community.rarity.pulls": {
        text: "{count} pulls",
        description: "How many pulls landed on this rarity, under its bar.",
    },
    "community.rarity.chip.vsExpected": {
        text: "vs expected",
        description: "Label on the chip comparing the community's observed rate against the advertised one. Rendered uppercase in a tight row.",
    },
    "community.rarity.chip.youVsCommunity": {
        text: "you vs community",
        description: "Label on the chip comparing the viewer's own rate against the community's. Rendered uppercase in a tight row.",
    },
    "community.rarity.chip.youVsExpected": {
        text: "you vs expected",
        description: "Label on the chip comparing the viewer's own rate against the advertised one. Rendered uppercase in a tight row.",
    },
    "community.rarity.bar.expected": {
        text: "Expected {value}",
        description: "Tooltip on the tick marking the advertised rate on a rarity bar. {value} is already a formatted percentage.",
    },
    "community.rarity.bar.you": {
        text: "You {value}",
        description: "Tooltip on the tick marking the viewer's own rate on a rarity bar. {value} is already a formatted percentage.",
    },
    "community.rarity.legend.expected": {
        text: "expected (with pity)",
        description: "Legend entry for the tick showing the advertised rate. 'Pity' is the game's rising-chance mechanic; the advertised rate is adjusted for it.",
    },
    "community.rarity.legend.you": {
        text: "you",
        description: "Legend entry for the tick showing the viewer's own rate. Rendered uppercase.",
    },
    "community.rarity.legend.note": {
        text: "expected rates fold 6★ soft pity into the advertised 2/8/50/40% base - long-run 6★ is ~2.89%, and lower rarities dip slightly because a pity 6★ takes their slot",
        description: "Legend footnote explaining how the expected rates were derived. 'Soft pity' is the game's rising 6-star chance after fifty pulls.",
    },
    "community.rarity.legend.scaleNote": {
        text: 'comparisons scale to baseline (e.g. 4% vs 2.89% expected = 138% of expected, not "+1.1%")',
        description: "Legend footnote explaining that the comparison chips express a ratio of the baseline rather than a point difference.",
    },
} satisfies MessageMap;

// `dynamic`: the rate-hint keys are stored on the rarity rows and resolved by
// the row component as `t(row.rateHintKey)`, so the extractor has no literal
// call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
