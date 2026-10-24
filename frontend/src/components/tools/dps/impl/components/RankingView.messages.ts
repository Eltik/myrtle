import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The all-operator ranking view of the DPS calculator. Operator, skill and module names come
 * from the game data. `S1` and the module type codes are the game's own designators.
 */
export const namespace = "tools";

export const messages = {
    "dps.rank.title": {
        text: "Every operator, one setup",
        description: "Heading of the ranking view: all operators ranked under the same enemy and buffs.",
    },
    "dps.rank.desc": {
        text: "Each skill and module of every supported operator against the enemy and team buffs on the left.",
        description: "Subtitle of the ranking view.",
    },
    "dps.rank.preset": {
        text: "Investment",
        description: "Label of the selector for the investment every ranked operator is computed at.",
    },
    "dps.rank.preset.p1Max": {
        text: "Max, P1",
        description: "Investment option: highest promotion and level, max trust, highest skill rank, module level 3, potential 1.",
    },
    "dps.rank.preset.p6Max": {
        text: "Max, P6",
        description: "Investment option: as 'Max, P1' but at potential 6.",
    },
    "dps.rank.preset.default": {
        text: "Calculator default",
        description: "Investment option: the calculator's own default for each operator.",
    },
    "dps.rank.preset.hint": {
        text: "Max: highest promotion and level, trust 100, highest skill rank, module level 3.",
        description: "Tooltip explaining what the 'Max' investment presets set.",
    },
    "dps.rank.sort": {
        text: "Sort by",
        description: "Label of the ranking's sort selector.",
    },
    "dps.rank.search": {
        text: "Search operators, skills, modules",
        description: "Placeholder of the ranking's search box.",
    },
    "dps.rank.class": {
        text: "Class",
        description: "Label of the ranking's class filter chips.",
    },
    "dps.rank.rarity": {
        text: "Rarity",
        description: "Label of the ranking's star-rarity filter chips.",
    },
    "dps.rank.rarityValue": {
        text: "{stars}★",
        description: "One rarity filter chip, e.g. '6★'. Keep the star sign.",
    },
    "dps.rank.damageType": {
        text: "Damage",
        description: "Label of the ranking's damage-type filter chips.",
    },
    "dps.rank.physical": {
        text: "Physical",
        description: "Damage type filter: physical attackers.",
    },
    "dps.rank.arts": {
        text: "Arts",
        description: "Damage type filter: arts attackers. 'Arts' is the game's magic damage type.",
    },
    "dps.rank.clearFilters": {
        text: "Clear filters",
        description: "Button that removes every class, rarity and damage-type filter and the search.",
    },
    "dps.rank.count": {
        text: "{shown} of {total} rows",
        description: "How many ranked rows pass the filters, out of all computed rows.",
    },
    "dps.rank.skipped": {
        text: "{count, plural, one {# combination} other {# combinations}} skipped (missing game data)",
        description: "Footnote: skill/module combinations the engine could not compute.",
    },
    "dps.rank.loading": {
        text: "Ranking every operator…",
        description: "Shown while the ranking is computed.",
    },
    "dps.rank.failed": {
        text: "The ranking could not be computed: {error}",
        description: "Shown when the ranking request failed. {error} is the server's message.",
    },
    "dps.rank.empty": {
        text: "No row matches these filters.",
        description: "Empty state of the ranking table.",
    },
    "dps.rank.col.rank": {
        text: "#",
        description: "Column heading: position in the ranking. A single symbol.",
    },
    "dps.rank.col.operator": {
        text: "Operator",
        description: "Column heading of the ranking table.",
    },
    "dps.rank.col.skill": {
        text: "Skill",
        description: "Column heading of the ranking table.",
    },
    "dps.rank.col.module": {
        text: "Module",
        description: "Column heading of the ranking table.",
    },
    "dps.rank.col.category": {
        text: "Shape",
        description: "Column heading: skill shape (sustained, burst, instant). Very narrow.",
    },
    "dps.rank.col.retention": {
        text: "Ref. retention",
        description: "Column heading: the roster score, how much of its damage an operator keeps across the game's real enemies. 'Ref.' marks that it is computed at a fixed reference setup, not this page's.",
    },
    "dps.rank.col.retention.hint": {
        text: "Share of its no-armor damage this operator keeps across real enemies, weighted by how often each appears in stages. Computed at the reference setup, not the one on this page.",
        description: "Tooltip on the retention column heading.",
    },
    "dps.rank.col.startup": {
        text: "First cast",
        description: "Column heading: seconds until the skill can first be used.",
    },
    "dps.rank.col.duration": {
        text: "Duration",
        description: "Column heading: how long the skill lasts, in seconds.",
    },
    "dps.rank.col.cycle": {
        text: "Cycle",
        description: "Column heading: seconds per full skill cycle.",
    },
    "dps.rank.col.split": {
        text: "Main / others",
        description: "Column heading: the sorted metric split between the main target and the secondary targets.",
    },
    "dps.rank.col.ratio": {
        text: "vs #1",
        description: "Column heading: this row's sorted metric as a share of the top row's. Very narrow.",
    },
    "dps.rank.retentionPending": {
        text: "Retention scores are still being computed; the column appears when they are ready.",
        description: "Note while the server has not finished the roster scores.",
    },
    "dps.rank.timingsHint": {
        text: "Start-up and cycle times need the SP-aware cycle model (enemy panel).",
        description: "Note shown when the timing columns are empty because the classic cycle model is selected.",
    },
    "dps.rank.basicAttack": {
        text: "Basic attack",
        description: "Skill column value for a row computed with no skill.",
    },
    "dps.rank.noModule": {
        text: "None",
        description: "Module column value for a row computed without a module.",
    },
    "dps.rank.add": {
        text: "Add {name} to the comparison",
        description: "Accessible name and tooltip of the + button that adds a ranked row as a calculator card.",
    },
    "dps.rank.inComparison": {
        text: "Already in the comparison",
        description: "Tooltip on the check mark of a ranked row that matches a card in the comparison.",
    },
    "dps.rank.added": {
        text: "Added {name} to the comparison",
        description: "Toast after a ranked row became a calculator card.",
    },
    "dps.rank.full": {
        text: "The comparison holds at most {count} cards",
        description: "Toast when a ranked row cannot be added because the comparison is full.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
