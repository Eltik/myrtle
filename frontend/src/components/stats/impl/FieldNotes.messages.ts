import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stats";

export const messages = {
    "fieldNotes.kicker": {
        text: "Field notes",
        description: "Small uppercase heading over a row of miscellaneous facts drawn from the game data and synced players.",
    },
    "fieldNotes.picker": {
        text: "Status",
        description: "Accessible name of the chip row that picks which crowd-control status the first two facts are about.",
    },
    "fieldNotes.status": {
        text: "{status, select, stun {Stun} silence {Silence} sleep {Sleep} frozen {Freeze} levitate {Levitate} other {{status}}}",
        description: "A chip label naming one status, title case. {status} is one of stun, silence, sleep, frozen, levitate: the game's crowd-control statuses. Use the game's own term for each.",
    },
    "fieldNotes.immune.label": {
        text: "Enemy immunity",
        description: "Fact tile label, rendered uppercase: the share of enemies immune to the picked status.",
    },
    "fieldNotes.immune.sentence": {
        text: "of enemies are immune to {status, select, stun {stun} silence {silence} sleep {sleep} frozen {freeze} levitate {levitation} other {{status}}}",
        description: "Sentence that follows a big percentage, e.g. '12% / of enemies are immune to stun'. {status} is one of stun, silence, sleep, frozen, levitate: the game's crowd-control statuses.",
    },
    "fieldNotes.immune.levels": {
        text: "Boss {boss} · Elite {elite} · Normal {normal}",
        description: "Per-grade split of the immunity share. Boss, Elite and Normal are the game's enemy grades; each placeholder is an already-formatted percentage or '-'.",
    },
    "fieldNotes.inflict.label": {
        text: "Inflicted by",
        description: "Fact tile label, rendered uppercase: the share of operators with a skill that applies the picked status.",
    },
    "fieldNotes.inflict.sentence": {
        text: "of operators have a skill that inflicts {status, select, stun {stun} silence {silence} sleep {sleep} frozen {freeze} levitate {levitation} other {{status}}}",
        description: "Sentence that follows a big percentage. It says SKILL on purpose: talents are not counted. {status} is one of stun, silence, sleep, frozen, levitate: the game's crowd-control statuses.",
    },
    "fieldNotes.mostOwned.label": {
        text: "Most owned 6★",
        description: "Fact tile label, rendered uppercase: the 6-star operator the most synced players own. ★ is the rarity star.",
    },
    "fieldNotes.mostOwned.sentence": {
        text: "of synced players own {name}",
        description: "Sentence that follows a big percentage, e.g. '76% / of synced players own Gladiia'. {name} is the operator's name. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "fieldNotes.rarest.label": {
        text: "Rarest 6★",
        description: "Fact tile label, rendered uppercase: the six-star operator the fewest synced players own. Keep the star symbol.",
    },
    "fieldNotes.rarest.sentence": {
        text: "of synced players own {name}, the rarest 6★",
        description: "Sentence that follows a big percentage, e.g. '7.1% / of synced players own Bellone, the rarest 6★'. Keep the star symbol. {name} is the operator's name. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "fieldNotes.progress.label": {
        text: "Handbook progress",
        description: "Fact tile label, rendered uppercase: how much of the enemy handbook the average player has met.",
    },
    "fieldNotes.progress.sentence": {
        text: "The average player has met {met} of {total} enemies",
        description: "Sentence under a percentage. {met} and {total} are already-formatted numbers; {total} is always large, so the label is always plural. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "fieldNotes.progress.meta": {
        text: "across {count} players",
        description: "Meta line: how many players the average is taken over. {count} is an already-formatted number.",
    },
    "fieldNotes.ratio": {
        text: "{n} of {total}",
        description: "Numerator over denominator under a percentage, e.g. '179 of 1,541'. Both are already-formatted numbers.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
