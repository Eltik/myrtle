import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The DPS calculator's team-buff panel, the per-card zone editor and the supporter picker.
 * Operator, skill and talent names come from the game data; the scope condition text comes
 * from the backend's supporter table and is shown as served.
 */
export const namespace = "tools";

export const messages = {
    "dps.team.title": {
        text: "Team buffs",
        description: "Card heading: buffs from teammates and debuffs on the enemy, applied to every operator on the page.",
    },
    "dps.team.desc": {
        text: "Buffs from teammates and debuffs on the enemy. Every card gets them, and the ranking uses them too.",
        description: "Card subtitle explaining what the team buffs panel applies to.",
    },
    "dps.team.clear": {
        text: "Clear",
        description: "Button that resets every team buff and switches every supporter off.",
    },
    "dps.zones.more": {
        text: "More buffs",
        description: "Toggle that reveals the less common buff fields.",
    },
    "dps.zones.less": {
        text: "Fewer buffs",
        description: "Toggle that hides the less common buff fields again.",
    },
    "dps.zones.moreSet": {
        text: "{count, plural, one {# set} other {# set}}",
        description: "Badge on the 'More buffs' toggle: how many of the hidden fields have a value.",
    },
    "dps.helper.open": {
        text: "Which buff is this?",
        description: "Link that opens a guide matching game skill wording to the right input field.",
    },
    "dps.helper.intro": {
        text: "Find the phrase in the skill or talent text, then type its number into the field it names.",
        description: "Intro line of the buff guide popover.",
    },
    "dps.stacking.title": {
        text: "How the supporters combine",
        description: "Heading of the list that shows, per buff, which sources apply and which are ignored.",
    },
    "dps.stacking.typed": {
        text: "Typed in",
        description: "Name of a buff source that the user typed into a field, as opposed to a supporter preset.",
    },
    "dps.stacking.ignored": {
        text: "Ignored: only the highest applies",
        description: "Tooltip on a buff source that does not count because a stronger source of the same kind exists.",
    },
    "dps.stacking.applied": {
        text: "Applied: {value}",
        description: "What finally reaches the calculation for one buff, after combining its sources.",
    },
    "dps.stacking.outOfScope": {
        text: "{classes} only: skipped for other classes",
        description: "Note on a supporter whose effect reaches only some classes; cards of other classes do not get it.",
    },

    "dps.supporters.add": {
        text: "Supporters",
        description: "Button that opens the supporter preset picker.",
    },
    "dps.supporters.title": {
        text: "Supporter presets",
        description: "Dialog heading: a list of operators whose buffs or debuffs can be switched on.",
    },
    "dps.supporters.desc": {
        text: "Switch a supporter on to add its buff. Values are read from game data at the investment you set here.",
        description: "Dialog subtitle of the supporter picker.",
    },
    "dps.supporters.promotion": {
        text: "Promotion",
        description: "Label of the supporters' Elite promotion selector.",
    },
    "dps.supporters.skillLevel": {
        text: "Skill rank",
        description: "Label of the supporters' skill level / mastery selector.",
    },
    "dps.supporters.potential": {
        text: "Potential",
        description: "Label of the supporters' potential selector.",
    },
    "dps.supporters.potentialValue": {
        text: "Potential {rank}",
        description: "One option of the supporters' potential selector.",
    },
    "dps.supporters.moduleLevel": {
        text: "Module",
        description: "Label of the supporters' module level selector.",
    },
    "dps.supporters.moduleNone": {
        text: "No module",
        description: "Option of the supporters' module selector: modules ignored.",
    },
    "dps.supporters.search": {
        text: "Search supporters",
        description: "Placeholder of the supporter picker's search box.",
    },
    "dps.supporters.loading": {
        text: "Loading supporters…",
        description: "Shown while the supporter list loads.",
    },
    "dps.supporters.failed": {
        text: "Supporters could not be loaded: {error}",
        description: "Shown when the supporter list request failed. {error} is the server's message.",
    },
    "dps.supporters.none": {
        text: "No supporter matches that search.",
        description: "Empty state of the supporter picker's filtered list.",
    },
    "dps.supporters.unresolved": {
        text: "{count, plural, one {# preset is} other {# presets are}} missing from this game data and hidden.",
        description: "Footnote: some supporter presets could not be read from the loaded game data.",
    },
    "dps.supporters.skill": {
        text: "Skill {index}",
        description: "Which part of a supporter the buff comes from: one of its skills, by number.",
    },
    "dps.supporters.talent": {
        text: "Talent {index}",
        description: "Which part of a supporter the buff comes from: one of its talents, by number.",
    },
    "dps.supporters.duration": {
        text: "{seconds}s",
        description: "How long a supporter's effect lasts, in seconds. Keep it compact.",
    },
    "dps.supporters.durationHint": {
        text: "The buff is applied for the whole calculation; in game it lasts {seconds}s.",
        description: "Tooltip on a supporter's duration: the calculator treats the buff as always on.",
    },
    "dps.supporters.inspireFrom": {
        text: "from {atk} ATK",
        description: "Shown beside an Inspiration value: the supporter ATK it was computed from.",
    },
    "dps.supporters.remove": {
        text: "Switch off {name}",
        description: "Accessible name of the small × on an active supporter chip.",
    },
    "dps.supporters.toggle": {
        text: "Use {name}",
        description: "Accessible name of a supporter's on/off switch.",
    },
    "dps.supporters.rankNote": {
        text: "The ranking applies every supporter to every row, including class-limited ones.",
        description: "Note in the ranking view: class limits on supporters are not applied per row there.",
    },
    "dps.supporters.blocked": {
        text: "Results are paused: the supporter values could not be loaded ({error}). Switch the supporters off or try again.",
        description: "Shown when switched-on supporters' values failed to load; the calculator does not compute without them. {error} is the server's message.",
    },
    "dps.supporters.waiting": {
        text: "Waiting for supporter values…",
        description: "Shown while results wait for the switched-on supporters' values to load.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
