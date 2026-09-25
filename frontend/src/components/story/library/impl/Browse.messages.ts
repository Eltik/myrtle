import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "browse.search.placeholder": {
        text: "Search stories",
        description: "Placeholder of the story library's search box.",
    },
    "browse.search.aria": {
        text: "Search stories by chapter, story or operation code",
        description: "Accessible name of the story library's search box.",
    },
    "browse.filter.all": {
        text: "All",
        description: "Filter pill: every chapter and every operator record.",
    },
    "browse.filter.main": {
        text: "Main story",
        description: "Filter pill: the main story chapters.",
    },
    "browse.filter.events": {
        text: "Events",
        description: "Filter pill: the event chapters.",
    },
    "browse.filter.side": {
        text: "Side stories",
        description: "Filter pill: vignettes and intermezzi, the short-form shelves.",
    },
    "browse.filter.records": {
        text: "Operator records",
        description: "Filter pill: the operators' own record sets.",
    },
    "browse.view.aria": {
        text: "Layout",
        description: "Accessible name of the grid or list layout toggle.",
    },
    "browse.view.grid": {
        text: "Grid",
        description: "Layout toggle: cover-art cards.",
    },
    "browse.view.list": {
        text: "List",
        description: "Layout toggle: one row per chapter.",
    },
    "browse.jump.aria": {
        text: "Jump to a section",
        description: "Accessible name of the sticky section jump list.",
    },
    "browse.section.main": {
        text: "Main story",
        description: "Section heading holding the main story chapters, used when the game's own storyline shelves are not available.",
    },
    "browse.section.other": {
        text: "Other events",
        description: "Section heading holding the chapters the game puts on no storyline shelf.",
    },
    "browse.section.undated": {
        text: "Undated",
        description: "Section heading holding the chapters with no release date, used when the game's own storyline shelves are not available.",
    },
    "browse.section.records": {
        text: "Operator records",
        description: "Section heading holding the operators' own record sets, collapsed until opened.",
    },
    "browse.section.count": {
        text: "{count, plural, one {# chapter} other {# chapters}}",
        description: "Count beside a section heading.",
    },
    "browse.section.recordCount": {
        text: "{count, plural, one {# operator} other {# operators}}",
        description: "Count beside the operator records heading.",
    },
    "browse.section.show": {
        text: "Show",
        description: "Button that opens the collapsed operator records section.",
    },
    "browse.section.hide": {
        text: "Hide",
        description: "Button that closes the operator records section.",
    },
    "browse.fallback": {
        text: "The running backend sends no storyline shelves yet, so the page is grouped by chapter order and release year instead of by the game's own themes. A backend restart fills them in.",
        description: "Notice shown when the wire carries no storylines and the fallback sectioning is in use.",
    },
    "browse.empty": {
        text: "Nothing matches that search.",
        description: "Shown when the search box and the filter pills leave no chapter.",
    },
    "browse.badge.main": {
        text: "MAIN STORY",
        description: "Card badge: a main story chapter.",
    },
    "browse.badge.event": {
        text: "EVENT",
        description: "Card badge: an event chapter.",
    },
    "browse.badge.intermezzo": {
        text: "INTERMEZZO",
        description: "Card badge: one of the three intermezzi, the game's own long-form side shelf.",
    },
    "browse.badge.vignette": {
        text: "VIGNETTE",
        description: "Card badge: a vignette, the game's short-form side shelf.",
    },
    "browse.badge.record": {
        text: "OPERATOR RECORD",
        description: "Card badge: an operator's own record set.",
    },
    "browse.card.chapter": {
        text: "Chapter {n}",
        description: "How a main story chapter is named on a card, a row and a heading. Readers know the mainline by its number, not by the act's title.",
    },
    "browse.card.chapterShort": {
        text: "EP.{n}",
        description: "The small monospace chapter mark printed over a list row's key visual, for main story chapters only. {n} is zero-padded to two digits.",
    },
    "browse.chip.range": {
        text: "Chapters {from} to {to}",
        description: "The chapter run a section covers, in full words. Shown in the tooltip of a collapsed jump chip, where there is room for the words.",
    },
    "browse.chip.rangeOne": {
        text: "Chapter {n}",
        description: "The single main story chapter a section covers, in full words. Shown in the tooltip of a collapsed jump chip.",
    },
    "browse.chip.compact": {
        text: "Ch. {from}-{to}",
        description: "The chapter run a section covers, abbreviated. The secondary line of a jump chip and of a section heading, where the section's name is the primary. A plain hyphen, in a monospace run.",
    },
    "browse.chip.compactOne": {
        text: "Ch. {n}",
        description: "The single main story chapter a section covers, abbreviated. The secondary line of a jump chip and of a section heading.",
    },
    "browse.chip.act": {
        text: "Act {ordinal} · {range}",
        description: "The secondary line of the active jump chip where the section's own name names an act ({ordinal} is the numeral out of that name, I or 1). No English arc names one today; the ordinal is drawn into the act's banner art instead.",
    },
    "browse.chip.includes": {
        text: "Includes chapters {from} to {to}",
        description: "Muted secondary under a themed shelf's name, when that shelf holds a run of main story chapters without being one. Never a heading.",
    },
    "browse.chip.includesOne": {
        text: "Includes chapter {n}",
        description: "Muted secondary under a themed shelf's name, when that shelf holds one main story chapter.",
    },
    "browse.chip.otherAbbr": {
        text: "MISC",
        description: 'The mono stand-in for "Other events" on a collapsed jump chip, where the full name does not fit. The full name is in the chip\'s tooltip and accessible name.',
    },
    "browse.chip.recordsAbbr": {
        text: "REC",
        description: 'The mono stand-in for "Operator records" on a collapsed jump chip, where the full name does not fit. The full name is in the chip\'s tooltip and accessible name.',
    },
    "browse.jump.picker": {
        text: "Section picker, current section: {name}",
        description: "Accessible name of the pill that replaces the jump chip row on a phone. It opens a sheet listing every section; {name} is the section the reader is in.",
    },
    "browse.jump.sheetTitle": {
        text: "Jump to a section",
        description: "Title of the bottom sheet that lists every section, opened from the section picker on a phone.",
    },
    "browse.row.fraction": {
        text: "{read} / {total}",
        description: "Read fraction printed on a list row, beside its progress bar.",
    },
    "browse.card.entries": {
        text: "{count, plural, one {# entry} other {# entries}}",
        description: "How many stories a chapter holds, on its card's code line.",
    },
    "browse.card.open": {
        text: "Open {name}",
        description: "Accessible name of a chapter card.",
    },
    "browse.card.openWithProgress": {
        text: "Open {name}, {read} of {total} read",
        description: "Accessible name of a story ticket. The ticket prints the read fraction as a bar only, so the number lives here.",
    },
    "browse.continue.kicker": {
        text: "Continue reading",
        description: "Kicker above the compact continue card.",
    },
    "browse.continue.kickerFresh": {
        text: "Start here",
        description: "Kicker above the compact continue card for a reader who has opened nothing yet.",
    },
    "chapter.close": {
        text: "Close",
        description: "Button that closes the chapter sheet.",
    },
    "chapter.continue": {
        text: "Continue",
        description: "Primary action on a chapter sheet: opens the first story with no read mark, or the last story once every one of them has one.",
    },
    "chapter.restart": {
        text: "Start from the beginning",
        description: "Secondary action on a chapter sheet: opens the chapter's first story. Shown only when it is not the same story Continue would open.",
    },
    "chapter.words": {
        text: "{words} words",
        description: "Word count of a chapter, and of one operation row inside it.",
    },
    "chapter.about": {
        text: "about {time}",
        description: 'Reading time of a whole chapter, beside its word count. {time} is a span such as "31m" or "1d 6h", derived from the reader\'s own words-per-minute setting.',
    },
    "chapter.timeLeft": {
        text: "{time} left",
        description: "Reading time of the stories in a chapter that are not read yet. Shown only while some but not all of it is read.",
    },
    "chapter.wordsUnknown": {
        text: "Not counted yet",
        description: "Stands in for the word count on a chapter the backend sends none for.",
    },
    "chapter.phase.before": {
        text: "Before",
        description: "One half of an operation row in a chapter sheet: the story that plays before the battle.",
    },
    "chapter.phase.after": {
        text: "After",
        description: "One half of an operation row in a chapter sheet: the story that plays after the battle.",
    },
    "chapter.phase.interlude": {
        text: "Interlude",
        description: "Marker on a chapter sheet row that is an interlude rather than an operation, so it has no before and after halves.",
    },
    "chapter.segment.aria": {
        text: "{title}, {phase}",
        description: "Accessible name of one half of an operation row. The visible label is only Before or After, which names nothing on its own.",
    },
    "chapter.video": {
        text: "Video",
        description: "Marker on a chapter sheet row whose script plays a cutscene video.",
    },
    "chapter.readFraction": {
        text: "{read} of {total} read",
        description: "Read fraction beside a chapter sheet's progress bar.",
    },
    "browse.read.aria": {
        text: "Read state",
        description: "Accessible name of the read-state filter, the segmented control beside the kind pills.",
    },
    "browse.read.any": {
        text: "Any",
        description: "Read-state filter: every chapter, whatever has been read of it. The default.",
    },
    "browse.read.unread": {
        text: "Unread",
        description: "Read-state filter: chapters with nothing read in them.",
    },
    "browse.read.progress": {
        text: "In progress",
        description: "Read-state filter: chapters with some but not all of their stories read.",
    },
    "browse.read.done": {
        text: "Finished",
        description: "Read-state filter: chapters with every readable story read.",
    },
    "browse.sort.label": {
        text: "Sort",
        description: "Label of the browse order picker.",
    },
    "browse.sort.default": {
        text: "Default",
        description: "Browse order: the game's own storyline shelves, in the game's own order. The only order that keeps the page in sections.",
    },
    "browse.sort.newest": {
        text: "Newest release",
        description: "Browse order: most recently released first. Chapters the table gives no release date sort last.",
    },
    "browse.sort.oldest": {
        text: "Oldest release",
        description: "Browse order: earliest released first. Chapters the table gives no release date sort last.",
    },
    "browse.sort.mostRead": {
        text: "Most read first",
        description: "Browse order: the chapters furthest through first.",
    },
    "browse.sort.leastRead": {
        text: "Least read first",
        description: "Browse order: the chapters least far through first.",
    },
    "browse.sort.title": {
        text: "Title A to Z",
        description: "Browse order: alphabetical by chapter name.",
    },
    "browse.section.sorted": {
        text: "All chapters",
        description: "Heading over the single flat section a sort other than Default produces, where the game's shelves would hide the order.",
    },
    "browse.toolbar.filters": {
        text: "Filters",
        description: "Button on a phone that opens the sheet holding the category, read-state and sort controls.",
    },
    "browse.toolbar.filtersAria": {
        text: "Filters, {count, plural, one {# active} other {# active}}",
        description: "Accessible name of the phone Filters button while some of category, read state and sort are off their default.",
    },
    "browse.toolbar.open": {
        text: "Search and filters",
        description: "Accessible name and tooltip of the sticky bar's button that opens the search box and the category, read-state, sort and layout controls.",
    },
    "browse.toolbar.openActive": {
        text: "Search and filters, {count, plural, one {# filter active} other {# filters active}}",
        description: "Accessible name of the sticky bar's search and filters button while some of category, read state and sort are off their default.",
    },
    "browse.toolbar.category": {
        text: "Category",
        description: "Label over the category pills (All, Main story, Events...) inside the filters sheet and popover.",
    },
    "browse.toolbar.done": {
        text: "Done",
        description: "Button that closes the filters sheet on a phone. The filters apply as they are tapped; this only closes it.",
    },
    "browse.continue.more": {
        text: "More reading options",
        description: "Accessible name of the menu button on the continue-reading row on a phone. It holds Start from the beginning and View chapter.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
