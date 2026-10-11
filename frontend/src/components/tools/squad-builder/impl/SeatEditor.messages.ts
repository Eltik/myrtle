import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.editor.seat": {
        text: "Seat {seat}",
        description: "Under the operator's name in the seat editor. {seat} is 1 to 12.",
    },
    "squad.editor.supportSeat": {
        text: "Support",
        description: "Under the operator's name in the seat editor when editing the support.",
    },
    "squad.editor.close": {
        text: "Close",
        description: "Button that closes the seat editor.",
    },
    "squad.editor.source.roster": {
        text: "From your roster",
        description: "Pill in the seat editor: the build shown is read from the player's synced roster.",
    },
    "squad.editor.source.hand": {
        text: "Build set by hand",
        description: "Pill in the seat editor: the player set this build themselves.",
    },
    "squad.editor.source.full": {
        text: "Full build, not in your roster",
        description: "Pill in the seat editor: the player does not own this operator, so a fully built version is assumed.",
    },
    "squad.editor.source.support": {
        text: "Support, full build",
        description: "Pill in the seat editor for the support: a borrowed operator is assumed fully built.",
    },
    "squad.editor.skill": {
        text: "Skill",
        description: "Label over the skill choice in the seat editor.",
    },
    "squad.editor.skillFallback": {
        text: "Skill {skill}",
        description: "Name shown for a skill whose name has not loaded yet. {skill} is 1, 2 or 3.",
    },
    "squad.editor.skillLevel": {
        text: "Skill level",
        description: "Label over the skill level slider. Levels go 1 to 7, then M1 to M3 (masteries).",
    },
    "squad.editor.rankLevel": {
        text: "Lv {rank}",
        description: "Skill level value next to the slider, e.g. 'Lv 7'.",
    },
    "squad.editor.rankMastery": {
        text: "M{mastery}",
        description: "Mastery value next to the skill level slider, e.g. 'M3'. M is the usual community abbreviation for mastery.",
    },
    "squad.editor.module": {
        text: "Module",
        description: "Label over the module choice in the seat editor.",
    },
    "squad.editor.noModule": {
        text: "None",
        description: "Module option for no module.",
    },
    "squad.editor.moduleRank": {
        text: "Module rank",
        description: "Label over the module rank choice (1 to 3).",
    },
    "squad.editor.promotion": {
        text: "Promotion",
        description: "Label over the Elite 0 / 1 / 2 choice.",
    },
    "squad.editor.level": {
        text: "Level",
        description: "Label over the operator level slider.",
    },
    "squad.editor.potential": {
        text: "Potential",
        description: "Label over the potential choice (1 to 6).",
    },
    "squad.editor.potentialAria": {
        text: "Potential {rank}",
        description: "Accessible name of a potential option. {rank} is 1 to 6.",
    },
    "squad.editor.useRoster": {
        text: "Use my roster",
        description: "Button that drops the hand-set build so the seat follows the player's synced roster again.",
    },
    "squad.editor.useFull": {
        text: "Use a full build",
        description: "Button on the support editor that drops the hand-set build and goes back to a fully built operator.",
    },
    "squad.editor.remove": {
        text: "Remove",
        description: "Button that takes the operator out of this seat.",
    },
    "squad.editor.hint.skill": {
        text: "Skill {skill} unlocks at Elite {phase}",
        description: "Hint under the skill choice when the chosen skill needs a higher promotion than the build has.",
    },
    "squad.editor.hint.module": {
        text: "The module unlocks at Elite {phase} Lv {level}",
        description: "Hint under the module choice when the build is below the module's unlock.",
    },
    "squad.editor.hint.roster": {
        text: "Your roster has E{phase} Lv {level}",
        description: "Hint when a hand-set build differs from what the player's roster has. E is Elite (promotion).",
    },
    "squad.editor.lab": {
        text: "Open in DPS calculator",
        description: "Button that opens the DPS calculator with this operator.",
    },
    "squad.editor.labPick": {
        text: "Pick squadmates to bring",
        description: "Accessible name of the small arrow next to Open in DPS calculator that opens the squadmate picker.",
    },
    "squad.editor.labTitle": {
        text: "Bring squadmates",
        description: "Heading of the popover that picks squadmates to send to the DPS calculator.",
    },
    "squad.editor.labDesc": {
        text: "The DPS calculator holds 4. Bring up to 3 squadmates; the buffs it knows apply there.",
        description: "Explainer in the squadmate popover.",
    },
    "squad.editor.labGives": {
        text: "buffs this seat",
        description: "Small note next to a squadmate who gives this seat a buff the DPS calculator can use.",
    },
    "squad.editor.labGo": {
        text: "Open with {count, plural, =0 {no squadmates} one {# squadmate} other {# squadmates}}",
        description: "Button in the squadmate popover that opens the DPS calculator.",
    },
    "squad.editor.labNone": {
        text: "No one else in the squad yet.",
        description: "Shown in the squadmate popover when the squad has no other operators.",
    },
    "squad.editor.readOnly": {
        text: "This is a shared squad. Copy it to edit.",
        description: "Shown in the seat editor when viewing a squad from a share link.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
