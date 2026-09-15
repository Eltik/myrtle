import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * One room's detail popover. Room and operator names come from the base
 * catalog and the game data; the skill dispositions are keyed by the API's own
 * `disposition` values, so those tables hold message KEYS.
 */
export const namespace = "user";

export const messages = {
    "profile.base.room.value": {
        text: "+{pct}% value",
        description: "Second half of a skill's marginal chip, when the skill raises the room's output value rather than its speed. The sign is part of the number and may be negative.",
    },
    "profile.base.room.disposition.inactive": {
        text: "inactive",
        description: "Marker on a base skill whose condition this crew does not meet. Lowercase, rendered uppercase by CSS.",
    },
    "profile.base.room.disposition.inactive.hint": {
        text: "This skill's condition isn't met by this crew, so it adds nothing here.",
        description: "Tooltip on the 'inactive' marker.",
    },
    "profile.base.room.disposition.covered": {
        text: "covered",
        description: "Marker on a base skill a stronger one of the same type already covers. Lowercase, rendered uppercase by CSS.",
    },
    "profile.base.room.disposition.covered.hint": {
        text: "A stronger skill of the same type is already active in this crew - the game only applies the most effective one, so this copy adds nothing on top.",
        description: "Tooltip on the 'covered' marker.",
    },
    "profile.base.room.disposition.per_room": {
        text: "per-room",
        description: "Marker on a base skill whose value lands in other rooms. Lowercase, rendered uppercase by CSS.",
    },
    "profile.base.room.disposition.per_room.hint": {
        text: "Active - this skill buffs matching operators in the rooms that satisfy it, so its value is counted inside those rooms' numbers. Open those rooms to see the credit.",
        description: "Tooltip on the 'per-room' marker.",
    },
    "profile.base.room.disposition.morale": {
        text: "morale",
        description: "Marker on a base skill that affects morale rather than output. Lowercase, rendered uppercase by CSS.",
    },
    "profile.base.room.disposition.morale.hint": {
        text: "This skill changes morale drain or recovery - it shows up in the sustainability simulation, not in this room's efficiency.",
        description: "Tooltip on the 'morale' marker.",
    },
    "profile.base.room.disposition.capacity": {
        text: "capacity",
        description: "Marker on a base skill that raises how much the room can hold. Lowercase, rendered uppercase by CSS.",
    },
    "profile.base.room.disposition.capacity.hint": {
        text: "This skill raises the room's order capacity, not its speed - it buys longer gaps between check-ins.",
        description: "Tooltip on the 'capacity' marker.",
    },
    "profile.base.room.disposition.non_production": {
        text: "reception / HR",
        description: "Marker on a base skill whose value is clues or hiring rather than production. 'HR' is the base's hiring office. Lowercase, rendered uppercase by CSS.",
    },
    "profile.base.room.disposition.non_production.hint": {
        text: "Non-production value (clues, training, HR) - counted in its own units, never folded into the efficiency number.",
        description: "Tooltip on the 'reception / HR' marker. 'Clues' are the in-game Reception Room's collectibles.",
    },
    "profile.base.room.disposition.unmodeled": {
        text: "not modeled",
        description: "Marker on a base skill the optimizer does not price. Lowercase, rendered uppercase by CSS.",
    },
    "profile.base.room.disposition.unmodeled.hint": {
        text: "The optimizer deliberately prices this at zero rather than guessing.",
        description: "Tooltip on the 'not modeled' marker.",
    },
    "profile.base.room.level": {
        text: "Lv {level}/{max}",
        description: "The facility's level out of its highest, in the popover header. 'Lv' is the game's abbreviation.",
    },
    "profile.base.room.staffed": {
        text: "Staffed",
        description: "Stat label: how many of the room's seats are filled. Rendered uppercase by CSS.",
    },
    "profile.base.room.generates": {
        text: "Generates",
        description: "Stat label on a power plant: the electricity it makes. Rendered uppercase by CSS.",
    },
    "profile.base.room.draws": {
        text: "Draws",
        description: "Stat label on a room that consumes electricity. Rendered uppercase by CSS.",
    },
    "profile.base.room.power": {
        text: "{kw} kW",
        description: "Value beside 'Generates' or 'Draws'. 'kW' is the unit the game uses for base power.",
    },
    "profile.base.room.producing": {
        text: "Producing",
        description: "Stat label: which recipe the factory is set to. Rendered uppercase by CSS.",
    },
    "profile.base.room.efficiency": {
        text: "Efficiency",
        description: "Stat label on a producing room: its combined speed bonus. Rendered uppercase by CSS.",
    },
    "profile.base.room.droneRecovery": {
        text: "Drone recovery",
        description: "Stat label on a power plant: how much faster drones come back. 'Drones' are the game's base helpers. Rendered uppercase by CSS.",
    },
    "profile.base.room.clueSearch": {
        text: "Clue search",
        description: "Stat label on the Reception Room: how much faster clues are found. 'Clues' are its in-game collectibles. Rendered uppercase by CSS.",
    },
    "profile.base.room.hrContact": {
        text: "HR contact",
        description: "Stat label on the Office: how much faster it finds recruits. 'HR' is the base's hiring office. Rendered uppercase by CSS.",
    },
    "profile.base.room.lmd": {
        text: "LMD / day",
        description: "Stat label: currency this room produces per day. 'LMD' is the in-game currency and stays as-is.",
    },
    "profile.base.room.sustained": {
        text: "24/7",
        description: "Tag on an operator the rotation never swaps out. Written as the around-the-clock shorthand.",
    },
    "profile.base.room.sustained.tooltip": {
        text: "Runs around the clock: Fiammetta's morale swap keeps this operator at full morale, so the plan never rotates them out.",
        description: "Tooltip on the 24/7 tag. 'Fiammetta' is an operator whose base skill restores another's morale; keep the game's name.",
    },
    "profile.base.room.bench": {
        text: "Bench",
        description: "Tag on an operator seated only to fill a spare seat. Rendered uppercase by CSS.",
    },
    "profile.base.room.bench.tooltip": {
        text: "Spare seat: this operator fills a free seat at the lowest opportunity cost. They were not chosen for their skills - any effect that still applies is a bonus.",
        description: "Tooltip on the 'Bench' tag.",
    },
    "profile.base.room.replaced": {
        text: "replaced",
        description: "Marker on a lower tier of a base skill that a promotion has superseded. Lowercase, rendered uppercase by CSS.",
    },
    "profile.base.room.nobodyWorking": {
        text: "Nobody is working here.",
        description: "Shown under an empty room's seats.",
    },
    "profile.base.room.seatsOpen": {
        text: "{count, plural, one {# seat} other {# seats}} open.",
        description: "Shown under a partly staffed room. A room has at most five seats.",
    },
    "profile.base.room.marginalNote": {
        text: "Values are marginals - what this room loses if that one skill is removed. Coupled skills overlap, so they don’t sum to the room total.",
        description: "Footnote under a room's skill ledger. The apostrophe is a typographic one.",
    },
    "profile.base.room.fromControlCenter": {
        text: "From the Control Center",
        description: "Heading over the bonuses the Control Center's crew casts into this room. 'Control Center' is the game's own room name. Rendered uppercase by CSS.",
    },
    "profile.base.room.scoreFailed": {
        text: "This layout could not be scored.",
        description: "Shown in place of a room's efficiency when the scoring request failed.",
    },
    "profile.base.room.scoring": {
        text: "Scoring…",
        description: "Shown in place of a room's efficiency while the layout is being scored. Ends with an ellipsis character.",
    },
    "profile.base.room.emptyRoom": {
        text: "An empty room produces nothing and buffs nothing.",
        description: "Shown for a room with seats but nobody in them.",
    },
    "profile.base.room.nonProducing": {
        text: "Only producing rooms report an efficiency. This crew contributes through the bonuses they cast elsewhere.",
        description: "Shown for a staffed room that has no output of its own.",
    },
    "profile.base.room.optimizeOne": {
        text: "Optimize this room only",
        description: "Button that re-staffs just this room, leaving the rest of the board alone.",
    },
    "profile.base.room.optimized": {
        text: "Optimized",
        description: "Label on the before-and-after figures of a room the optimizer changed. Rendered uppercase by CSS.",
    },
} satisfies MessageMap;

// `dynamic`: the disposition labels and hints are looked up by the API's own
// disposition value, so the extractor has no literal call site for them.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
