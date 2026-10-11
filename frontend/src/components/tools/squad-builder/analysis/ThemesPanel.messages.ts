import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.analysis.themes.title": { text: "Themes", description: "Title of the Squad Builder panel with fun facts the squad shares." },
    "squad.analysis.themes.question": { text: "Just for fun", description: "One-line description under the Themes panel's title." },
    "squad.analysis.themes.rarity": { text: "Total rarity", description: "Stat cell label: the squad's stars added up." },
    "squad.analysis.themes.rarity.caption": { text: "lower is a harder run", description: "Caption under the total rarity." },
    "squad.analysis.themes.topRarity": { text: "{stars} stars", description: "Stat cell label: how many operators of the top rarity ({stars} is the highest star count in the game, e.g. 6)." },
    "squad.analysis.themes.topRarity.caption": { text: "of {seats}", description: "Caption under the top-rarity count: out of the squad's seats." },
    "squad.analysis.themes.medics": { text: "Medics", description: "Stat cell label: how many Medic operators." },
    "squad.analysis.themes.classes": { text: "Classes", description: "Stat cell label: how many of the game's classes the squad has." },
    "squad.analysis.themes.classes.value": { text: "{count} of {total}", description: "How many of the game's classes the squad has, e.g. '6 of 8'; {total} is the number of classes in the game." },
    "squad.analysis.themes.classes.missing": { text: "{count} missing", description: "Caption under the class count: how many classes nobody has." },
    "squad.analysis.themes.classes.none": { text: "none missing", description: "Caption under the class count when every class is there." },
    "squad.analysis.themes.labels": { text: "Labels", description: "Section heading: things every seat shares." },
    "squad.analysis.themes.labels.hint": { text: "Every seat shares it", description: "Hint beside the Labels heading." },
    "squad.analysis.themes.close": { text: "Close", description: "Section heading: things all but one or two seats share." },
    "squad.analysis.themes.close.hint": { text: "All but one or two", description: "Hint beside the Close heading." },
    "squad.analysis.themes.shared": { text: "Shared", description: "Section heading: things two or more seats share." },
    "squad.analysis.themes.shared.hint": { text: "Two or more", description: "Hint beside the Shared heading." },
    "squad.analysis.themes.empty": { text: "None yet for this squad.", description: "Empty Labels or Close section." },
    "squad.analysis.themes.emptyShared": { text: "Nothing two of them share.", description: "Empty Shared section." },
    "squad.analysis.themes.kind.gender": { text: "Gender", description: "Theme row kind." },
    "squad.analysis.themes.kind.race": { text: "Race", description: "Theme row kind (the operator's in-game race, e.g. Feline)." },
    "squad.analysis.themes.kind.birthplace": { text: "Birthplace", description: "Theme row kind." },
    "squad.analysis.themes.kind.faction": { text: "Faction", description: "Theme row kind (e.g. Rhodes Island, Kazimierz)." },
    "squad.analysis.themes.kind.class": { text: "Class", description: "Theme row kind (e.g. Guard, Medic)." },
    "squad.analysis.themes.kind.branch": { text: "Branch", description: "Theme row kind: the sub-class (e.g. Lord, Ranger)." },
    "squad.analysis.themes.kind.position": { text: "Position", description: "Theme row kind: melee or ranged." },
    "squad.analysis.themes.kind.rarity": { text: "Rarity", description: "Theme row kind: star count." },
    "squad.analysis.themes.kind.artist": { text: "Artist", description: "Theme row kind: the illustrator." },
    "squad.analysis.themes.kind.voice": { text: "Voice", description: "Theme row kind: the voice actor." },
    "squad.analysis.themes.kind.birthday": { text: "Birthday", description: "Theme row kind." },
    "squad.analysis.themes.rarityValue": { text: "{count, plural, one {# star} other {# stars}}", description: "A rarity value in a theme row, e.g. '6 stars'." },
    "squad.analysis.themes.footer": { text: "Nothing here is ranked. Voice actors follow the {dub} dub. The support is left out.", description: "Footer of the Themes panel. {dub} is the voice language, e.g. EN or JP." },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
