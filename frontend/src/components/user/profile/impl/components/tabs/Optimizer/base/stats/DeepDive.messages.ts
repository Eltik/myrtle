import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The check-in deep dive. Room names come from the base catalog; the produced
 * goods keep the game's own names (LMD, Pure Gold, Battle Records).
 */
export const namespace = "user";

export const messages = {
    "profile.base.deep.title": {
        text: "Deep Dive",
        description: "Label on the collapsible panel of check-in economics and simulation detail.",
    },
    "profile.base.deep.noChange": {
        text: "no change",
        description: "Replaces a delta figure when the snapshot and the live number are equal. Lowercase on purpose.",
    },
    "profile.base.deep.hours": {
        text: "{hours}h",
        description: "A duration in hours, one decimal. 'h' is the abbreviation; very little room.",
    },
    "profile.base.deep.days": {
        text: "{days}d",
        description: "A duration in days, one decimal, used past two days. 'd' is the abbreviation; very little room.",
    },
    "profile.base.deep.product.lmd": {
        text: "LMD orders",
        description: "What a trading post produces: orders paid in LMD, the in-game currency. Keep the game's name. Rendered uppercase by CSS.",
    },
    "profile.base.deep.product.gold": {
        text: "Pure Gold",
        description: "What a factory set to the gold recipe produces. The game's own item name. Rendered uppercase by CSS.",
    },
    "profile.base.deep.product.exp": {
        text: "Battle Records",
        description: "What a factory set to the experience recipe produces. The game's own item name. Rendered uppercase by CSS.",
    },
    "profile.base.deep.product.none": {
        text: "Unconfigured",
        description: "Shown for a factory with no recipe set. Rendered uppercase by CSS.",
    },
    "profile.base.deep.checkin": {
        text: "Check-in economics",
        description: "Section heading: what logging in more or less often is worth. Rendered uppercase by CSS.",
    },
    "profile.base.deep.logInBefore": {
        text: "Log in before {time} to lose nothing",
        description: "Deadline sentence over the check-in figures. {time} is the highlighted clock time and may move wherever the sentence needs it.",
    },
    "profile.base.deep.stalls": {
        text: "{room} stalls {duration} after a claim - buffers below.",
        description: "Sentence under the deadline: which room fills up first and how long that takes. A 'claim' is collecting what the base has produced.",
    },
    "profile.base.deep.firstRoom": {
        text: "First room",
        description: "Stands in for the room's name in that sentence when it is not known.",
    },
    "profile.base.deep.conservative": {
        text: "conservative",
        description: "The word carrying the tooltip that explains how the estimates are modeled. Lowercase, mid-sentence.",
    },
    "profile.base.deep.conservative.tooltip": {
        text: "Order sizes are modeled conservatively (small orders, fast turnover), so real deadlines can be later than shown - never earlier. Trading posts are assumed gold-supplied.",
        description: "Tooltip on the word 'conservative'. 'Trading post' is the game's own room name.",
    },
    "profile.base.deep.drones": {
        text: "Drones",
        description: "Opens the drone line; the count follows. 'Drones' are the game's base helpers.",
    },
    "profile.base.deep.dronesFilling": {
        text: "Drones {meter} - full in {when}, spend before then",
        description: "The drone line while the meter is still filling. {meter} is the highlighted 'current/max' pair and {when} the highlighted time until it is full; both may move wherever the sentence needs them. 'Drones' are the game's base helpers.",
    },
    "profile.base.deep.dronesFull": {
        text: " - full, recovery is being wasted",
        description: "Replaces the rest of the drone line when the meter is already full. Keep the leading space and dash.",
    },
    "profile.base.deep.trainers": {
        text: "Best trainers for {class}: ",
        description: "Opens the trainer list; the operator names follow. {class} is an operator class name from the game data. Keep the trailing space.",
    },
    "profile.base.deep.atRisk": {
        text: "At risk from their current bar: ",
        description: "Opens the list of operators closest to running out of morale. 'Their current bar' is the morale left as of the last sync. Keep the trailing space.",
    },
    "profile.base.deep.atRisk.synced": {
        text: "At risk from their current bar (projected from your sync {ago} ago): ",
        description: "The same opener when the sync's age is known. {ago} is a duration such as '6.0h'. Keep the trailing space.",
    },
    "profile.base.deep.cadence.aria": {
        text: "Check-in cadence",
        description: "Accessible name of the buttons that pick how often the player logs in.",
    },
    "profile.base.deep.cadence.hours": {
        text: "{hours}h",
        description: "One cadence button: that many hours between check-ins. 'h' is the abbreviation.",
    },
    "profile.base.deep.cadence.custom": {
        text: "Custom check-in cadence in hours",
        description: "Accessible name of the box for a cadence the buttons do not offer.",
    },
    "profile.base.deep.cadence.customPlaceholder": {
        text: "h",
        description: "Placeholder in that box, standing for hours. One character of room.",
    },
    "profile.base.deep.period.aria": {
        text: "Loss period",
        description: "Accessible name of the buttons that pick whether losses are shown per day, week, month or year.",
    },
    "profile.base.deep.period.day": {
        text: "/day",
        description: "Suffix after the loss figures when they are counted per day.",
    },
    "profile.base.deep.period.week": {
        text: "/week",
        description: "Suffix after the loss figures when they are counted per week.",
    },
    "profile.base.deep.period.month": {
        text: "/month",
        description: "Suffix after the loss figures when they are counted per month.",
    },
    "profile.base.deep.period.year": {
        text: "/year",
        description: "Suffix after the loss figures when they are counted per year.",
    },
    "profile.base.deep.period.dayInitial": {
        text: "D",
        description: "One-letter button for the per-day period. The initial of 'day'; exactly one character fits.",
    },
    "profile.base.deep.period.weekInitial": {
        text: "W",
        description: "One-letter button for the per-week period. The initial of 'week'; exactly one character fits.",
    },
    "profile.base.deep.period.monthInitial": {
        text: "M",
        description: "One-letter button for the per-month period. The initial of 'month'; exactly one character fits.",
    },
    "profile.base.deep.period.yearInitial": {
        text: "Y",
        description: "One-letter button for the per-year period. The initial of 'year'; exactly one character fits.",
    },
    "profile.base.deep.lost.lmd": {
        text: "−{amount} LMD",
        description: "Currency lost at the chosen cadence. Keep the leading minus sign; 'LMD' is the in-game currency.",
    },
    "profile.base.deep.lost.gold": {
        text: "−{amount} gold",
        description: "Gold bars lost at the chosen cadence. Keep the leading minus sign; refers to the game's Pure Gold.",
    },
    "profile.base.deep.lost.exp": {
        text: "−{amount} EXP",
        description: "Experience lost at the chosen cadence. Keep the leading minus sign; 'EXP' is the game's abbreviation.",
    },
    "profile.base.deep.lost.suffix": {
        text: "{losses} {period}",
        description: "The loss figures followed by the period they are counted over, e.g. '−1,200 LMD /day'.",
    },
    "profile.base.deep.lost.nothing": {
        text: "nothing lost",
        description: "Shown instead of the loss figures when the chosen cadence wastes nothing. Lowercase on purpose.",
    },
    "profile.base.deep.furniture": {
        text: "Furniture upside",
        description: "Section heading: recovery the player could gain by decorating the dormitories. Rendered uppercase by CSS.",
    },
    "profile.base.deep.furniture.rate": {
        text: "+{rate}/h",
        description: "The extra morale recovery per hour a fully decorated dormitory would give. 'h' abbreviates hour.",
    },
    "profile.base.deep.furniture.line": {
        text: "Dormitory Lv{level} ambience {meter} - {rate} recovery sitting in the furniture shop.",
        description:
            "One dormitory's furniture line. {meter} is the highlighted 'comfort/limit' pair and {rate} the highlighted per-hour gain labelled by profile.base.deep.furniture.rate; both may move wherever the sentence needs them. 'Ambience' is the game's own name for a dormitory's comfort rating, 'Lv' its level abbreviation, and the 'furniture shop' is the in-game store.",
    },
    "profile.base.deep.outputByFacility": {
        text: "Output by facility",
        description: "Section heading over each room's daily output. Rendered uppercase by CSS.",
    },
    "profile.base.deep.daily.lmd": {
        text: "{amount} LMD",
        description: "A trading post's daily output. 'LMD' is the in-game currency.",
    },
    "profile.base.deep.daily.gold": {
        text: "{amount} gold",
        description: "A factory's daily output of gold bars. Refers to the game's Pure Gold.",
    },
    "profile.base.deep.daily.exp": {
        text: "{amount} EXP",
        description: "A factory's daily output of experience. 'EXP' is the game's abbreviation.",
    },
    "profile.base.deep.perDay": {
        text: " /day",
        description: "Follows a room's daily output figure. Keep the leading space.",
    },
    "profile.base.deep.fullIn": {
        text: "full in {duration} · {count} {unit}",
        description: "One room's buffer line: how long until it stops producing, and how much it holds. Keep the middle dot.",
    },
    "profile.base.deep.unit.orders": {
        text: "orders",
        description: "What a trading post's buffer holds. Always in this one form, whatever the count.",
    },
    "profile.base.deep.unit.items": {
        text: "items",
        description: "What a factory's buffer holds. Always in this one form, whatever the count.",
    },
    "profile.base.deep.overflows": {
        text: " - overflows at this cadence",
        description: "Appended to a room's buffer line when it fills before the next check-in. Keep the leading space and dash.",
    },
    "profile.base.deep.compare": {
        text: "Compare",
        description: "Section heading over the snapshot controls. Rendered uppercase by CSS.",
    },
    "profile.base.deep.copyPlan": {
        text: "Copy plan",
        description: "Button that copies the planned staffing to the clipboard as text.",
    },
    "profile.base.deep.copy.header": {
        text: "Base plan - {lmd} LMD/day, {exp} EXP/day",
        description: "First line of the copied plan. 'LMD' and 'EXP' are the game's own abbreviations.",
    },
    "profile.base.deep.copy.shift": {
        text: "Shift {n}",
        description: "Heading of one shift inside the copied plan.",
    },
    "profile.base.deep.copy.room": {
        text: "{room}: {operators}",
        description: "One room's line inside the copied plan: the room's name and its crew, comma-separated.",
    },
    "profile.base.deep.resnapshot": {
        text: "Re-snapshot",
        description: "Button that replaces the stored snapshot with the current numbers.",
    },
    "profile.base.deep.snapshot": {
        text: "Snapshot",
        description: "Button that stores the current numbers to compare against.",
    },
    "profile.base.deep.clearSnapshot": {
        text: "Clear snapshot",
        description: "Accessible name of the button that discards the stored snapshot.",
    },
    "profile.base.deep.snapshotHint": {
        text: "Snapshot the current numbers, then edit the board or run the optimizer - the difference tracks live.",
        description: "Shown in place of the comparison table before a snapshot is taken.",
    },
    "profile.base.deep.diff.efficiency": {
        text: "Production efficiency",
        description: "Comparison row: the combined speed bonus of the producing rooms.",
    },
    "profile.base.deep.diff.gold": {
        text: "Pure Gold / day",
        description: "Comparison row: gold bars per day. 'Pure Gold' is the game's own item name.",
    },
    "profile.base.deep.diff.tradingCapacity": {
        text: "Trading capacity",
        description: "Comparison row: how many orders the trading posts hold in total.",
    },
    "profile.base.deep.diff.powerNet": {
        text: "Power net",
        description: "Comparison row: generated electricity minus consumed.",
    },
    "profile.base.deep.diff.dormRecovery": {
        text: "Dorm recovery",
        description: "Comparison row: morale the dormitories restore per hour. 'Dorm' is short for dormitory.",
    },
    "profile.base.deep.diff.firstStall": {
        text: "First room stalls",
        description: "Comparison row: how long until the first room's buffer is full.",
    },
    "profile.base.deep.simulating.title": {
        text: "Simulated totals & rotation events",
        description: "Section heading shown while the rotation simulation is still running.",
    },
    "profile.base.deep.simulating": {
        text: "Simulating the rotation - this is the slow part…",
        description: "Placeholder under that heading. Ends with an ellipsis character.",
    },
    "profile.base.deep.simulated.title": {
        text: "Simulated totals · {days} days under the rotation",
        description: "Section heading over each room's simulated output. Keep the middle dot; the horizon is a handful of days.",
    },
    "profile.base.deep.idle": {
        text: "{hours}h idle",
        description: "How long a room sat idle during the simulation. 'h' abbreviates hour.",
    },
    "profile.base.deep.noIdle": {
        text: "no idle",
        description: "Shown instead when a room never sat idle. Lowercase on purpose.",
    },
    "profile.base.deep.unrotated.title": {
        text: "Without rotating · {days} days",
        description: "Section heading for the simulation where nobody is ever swapped out. Keep the middle dot.",
    },
    "profile.base.deep.unrotated.fine": {
        text: "Your stationed crews hold up even with no swaps at all.",
        description: "Shown under that heading when nobody runs out of morale.",
    },
    "profile.base.deep.unrotated.dry": {
        text: "{count, plural, one {# operator runs dry} other {# operators run dry}}",
        description: "Highlighted opener under that heading: how many operators hit zero morale. A handful at most.",
    },
    "profile.base.deep.unrotated.line": {
        text: "{count} if you never swap - first at {hours}h ({name}). Rotate or lose the hours.",
        description: "Sentence under that heading when at least one operator runs out of morale. {count} is the highlighted opener labelled by profile.base.deep.unrotated.dry and may move wherever the sentence needs it. {name} is an operator's name; 'h' abbreviates hour.",
    },
    "profile.base.deep.events.title": {
        text: "Rotation events · {days} simulated days",
        description: "Section heading over the moments operators run out of morale. Keep the middle dot.",
    },
    "profile.base.deep.events.none": {
        text: "Nobody runs dry - every operator survives the recommended rotation.",
        description: "Shown under that heading when the rotation holds up.",
    },
    "profile.base.deep.events.when": {
        text: "Day {day}, {hours}h",
        description: "When one depletion happens: the simulated day and the hour within it.",
    },
    "profile.base.deep.events.runsDry": {
        text: "runs dry in {room}",
        description: "Follows the operator's name: which room they were working in when their morale hit zero.",
    },
} satisfies MessageMap;

// `dynamic`: the period suffixes and one-letter buttons are resolved through
// tables keyed by the selected period, which the extractor cannot see.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
