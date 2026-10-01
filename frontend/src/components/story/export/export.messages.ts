import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "export.open": {
        text: "Export",
        description: "Button on the chapter sheet and in the reader that opens the export sheet (save stories as an e-book or a text file).",
    },
    "export.openStory": {
        text: "Export this story",
        description: "Accessible name of the reader's export control, which opens the export sheet with the current story selected.",
    },
    "export.settingsHint": {
        text: "Save this story or its chapter as an e-book or a text file.",
        description: "Hint beside the Export button in the reader's settings sheet.",
    },
    "export.title": {
        text: "Export",
        description: "Title of the export sheet.",
    },
    "export.subtitle": {
        text: "Save {group} to read offline, as an e-book or a text file.",
        description: "One-line description under the export sheet's title. {group} is the chapter or event name.",
    },
    "export.scope": {
        text: "Stories",
        description: "Label of the export sheet's scope control: which stories go into the file.",
    },
    "export.scope.story": {
        text: "This story",
        description: "Scope option: only the story open in the reader.",
    },
    "export.scope.chapter": {
        text: "Whole chapter",
        description: "Scope option: every story in the chapter or event.",
    },
    "export.scope.selection": {
        text: "Choose…",
        description: "Scope option: a checklist of the chapter's stories appears to pick from.",
    },
    "export.selection.all": {
        text: "All",
        description: "Button above the story checklist that ticks every story.",
    },
    "export.selection.none": {
        text: "None",
        description: "Button above the story checklist that unticks every story.",
    },
    "export.selection.count": {
        text: "{count, plural, one {# story} other {# stories}} chosen",
        description: "How many stories are ticked in the export checklist.",
    },
    "export.format": {
        text: "Format",
        description: "Label of the export sheet's file format control.",
    },
    "export.format.epub": {
        text: "EPUB",
        description: "Format option: an e-book file for Apple Books, Kobo, Google Play Books, Send to Kindle.",
    },
    "export.format.markdown": {
        text: "Markdown",
        description: "Format option: a Markdown text file.",
    },
    "export.format.text": {
        text: "Text",
        description: "Format option: a plain text file.",
    },
    "export.format.pdf": {
        text: "PDF",
        description: "Format option: a paginated PDF with page numbers, a contents page and bookmarks.",
    },
    "export.images": {
        text: "Pictures",
        description: "Label of the export sheet's image control: which pictures go into the file.",
    },
    "export.images.cg": {
        text: "Illustrations",
        description: "Image option: only the story's CG illustrations.",
    },
    "export.images.cgbg": {
        text: "Illustrations and scenes",
        description: "Image option: CG illustrations plus every scene background.",
    },
    "export.images.none": {
        text: "None",
        description: "Image option: text only.",
    },
    "export.images.count": {
        text: "This chapter has {cgs, plural, one {# illustration} other {# illustrations}} and {scenes, plural, one {# scene} other {# scenes}}.",
        description: "Hint under the pictures control: how many CG illustrations and scene backgrounds the whole chapter uses, from its illustration listing.",
    },
    "export.images.linked": {
        text: "Markdown and text link to pictures online instead of carrying them.",
        description: "Hint under the image control when Markdown or Text is chosen.",
    },
    "export.branches": {
        text: "Choices",
        description: "Label of the export sheet's control for the story's decision branches.",
    },
    "export.branches.all": {
        text: "Every branch",
        description: "Branch option: print every option the Doctor could pick, with what each one leads to.",
    },
    "export.branches.path": {
        text: "My choices",
        description: "Branch option: print only the options the reader picked in the reader (the first option where they picked none).",
    },
    "export.name": {
        text: "Doctor's name",
        description: "Label of the field for the name printed wherever the story addresses the Doctor. Prefilled from the reader's setting.",
    },
    "export.typeface": {
        text: "Typeface",
        description: "Label of the export sheet's typeface control, embedded in the e-book.",
    },
    "export.typeface.inter": {
        text: "Inter",
        description: "Typeface option: the site's own sans-serif, embedded in the e-book.",
    },
    "export.typeface.opendyslexic": {
        text: "OpenDyslexic",
        description: "Typeface option: a face designed for readers with dyslexia, embedded in the e-book.",
    },
    "export.typeface.device": {
        text: "Reader's own",
        description: "Typeface option: embed no font, the e-reader's own typeface is used.",
    },
    "export.typeface.packagedOnly": {
        text: "Only an EPUB or a PDF carries a typeface.",
        description: "Hint under the typeface control when Markdown or Text is chosen.",
    },
    "export.estimate": {
        text: "{stories, plural, one {# story} other {# stories}} · {words, plural, one {# word} other {# words}} · about {time} · {images, plural, one {# picture} other {# pictures}} · about {size}",
        description: "The live size estimate above the Download button. {time} is a reading time like '5h 50m', {size} an estimated file size like '11.6 MB'.",
    },
    "export.estimate.rough": {
        text: "The picture count is an estimate until the chapter's illustration list loads.",
        description: "Hint under the estimate when the image count is a fallback guess.",
    },
    "export.download": {
        text: "Download",
        description: "Button that builds the file and saves it.",
    },
    "export.cancel": {
        text: "Cancel",
        description: "Button that stops a running export.",
    },
    "export.close": {
        text: "Close",
        description: "Button that closes the export sheet.",
    },
    "export.progress.scripts": {
        text: "Loading {done} of {total} · {story}",
        description: "Progress line while the stories' scripts download. {story} is the story's code and title.",
    },
    "export.progress.book": {
        text: "Writing {done} of {total} · {story}",
        description: "Progress line while the e-book is written, one story at a time, pictures included.",
    },
    "export.progress.finishing": {
        text: "Finishing the file…",
        description: "Progress line after the last story, while the file is closed and saved.",
    },
    "export.done": {
        text: "Saved {file}.",
        description: "Status line after the file was saved. {file} is its file name.",
    },
    "export.doneMissing": {
        text: "{count, plural, one {# picture} other {# pictures}} could not be loaded and were left out.",
        description: "Second sentence of the status line after saving when some pictures failed to download.",
    },
    "export.doneDisk": {
        text: "Written to {file}.",
        description: "Status line after the file was written straight to the place the reader picked. {file} is its name.",
    },
    "export.partial.removed": {
        text: "The unfinished file was removed.",
        description: "Added to the cancelled or failed line when a file that was being written to disk was deleted again.",
    },
    "export.partial.kept": {
        text: "The unfinished file may still be on disk; this browser could not remove it.",
        description: "Added to the cancelled or failed line when a partly written file could not be deleted.",
    },
    "export.progress.layoutStory": {
        text: "Laying out story {done} of {total} · {story}",
        description: "Progress line while the PDF is typeset, one story at a time. {story} is the story's code and title.",
    },
    "export.stalled": {
        text: "The export stopped making progress for {seconds} s at {story} and was stopped. Try a smaller scope.",
        description: "Error line when a PDF export went silent (no progress for a minute). {story} is where it was.",
    },
    "export.storyFailed": {
        text: "The export failed at {story}: {message}",
        description: "Error line when one story could not be laid out. {story} is its code and title, {message} the technical reason.",
    },
    "export.progress.layout": {
        text: "Laying out pages…",
        description: "Progress line while the PDF is typeset, after every story and picture is loaded.",
    },
    "export.blobWarning": {
        text: "This file will be about {size}. This browser builds it in memory before saving, which may fail on a phone; a smaller scope is safer.",
        description: "Warning above the Download button when the export is large and cannot be streamed to disk. {size} is like '180 MB'.",
    },
    "export.paper": {
        text: "Paper",
        description: "Label of the PDF paper-size control.",
    },
    "export.paper.a5": {
        text: "A5",
        description: "Paper size option, the small book size.",
    },
    "export.paper.a4": {
        text: "A4",
        description: "Paper size option.",
    },
    "export.paper.letter": {
        text: "Letter",
        description: "Paper size option, US Letter.",
    },
    "export.book.open": {
        text: "Print this story",
        description: "Link in the export sheet (reader only) to the story as one printable page, for the browser's Print or Save as PDF.",
    },
    "export.book.back": {
        text: "Back to the reader",
        description: "Button above the printable document view of a story, returning to the reader.",
    },
    "export.book.print": {
        text: "Print or save as PDF",
        description: "Button above the printable document view of a story; opens the browser's print dialog, where Save as PDF is a destination.",
    },
    "export.scope.arc": {
        text: "This arc",
        description: "Scope option: every chapter in the mainline arc this chapter belongs to (for example Hour of An Awakening).",
    },
    "export.scope.storyline": {
        text: "This storyline",
        description: "Scope option: every event on the storyline shelf this event belongs to.",
    },
    "export.scope.range": {
        text: "Reading order…",
        description: "Scope option: pick a first and a last chapter or event in a reading order, and export everything between.",
    },
    "export.scope.groups": {
        text: "{title}: {count, plural, one {# chapter or event} other {# chapters and events}}",
        description: "Line under a wide scope naming it and how many groups it covers. {title} is data (an arc or shelf name).",
    },
    "export.range.release": {
        text: "Release order",
        description: "Reading order mode for the range picker: by release date.",
    },
    "export.range.storyline": {
        text: "Storyline order",
        description: "Reading order mode for the range picker: the Main Theme in chapter order, then the rest.",
    },
    "export.range.from": {
        text: "From",
        description: "Label of the first-group picker of a reading-order range.",
    },
    "export.range.to": {
        text: "To",
        description: "Label of the last-group picker of a reading-order range.",
    },
    "export.cancelled": {
        text: "Export cancelled.",
        description: "Status line after the reader cancelled an export.",
    },
    "export.error": {
        text: "The export failed: {message}",
        description: "Error line when building the file failed. {message} is the technical reason.",
    },
    "export.empty": {
        text: "Choose at least one story.",
        description: "Shown instead of the estimate when the checklist has nothing ticked.",
    },
    "book.contents": {
        text: "Contents",
        description: "Heading of the table of contents inside an exported e-book.",
    },
    "book.cover": {
        text: "Cover",
        description: "Name of the cover page inside an exported e-book (shown in the e-reader's navigation).",
    },
    "book.colophon": {
        text: "Colophon",
        description: "Name of the closing credits page inside an exported e-book.",
    },
    "book.choice": {
        text: "Choose",
        description: "Label printed above a decision's list of options inside an exported book.",
    },
    "book.ifChose": {
        text: "If you chose {options}",
        description: "Heading over the lines one option leads to, inside an exported book. {options} is the quoted option text, several joined by ' / '.",
    },
    "book.earlier": {
        text: "If you had chosen {options}",
        description: "Heading over lines that depend on an EARLIER decision, inside an exported book. {options} is the quoted option text.",
    },
    "book.cutscene": {
        text: "Cutscene",
        description: "Marker printed where the story plays an animated cutscene, inside an exported book.",
    },
    "book.figureAlt": {
        text: "Illustration",
        description: "Alternative text of a CG illustration inside an exported book.",
    },
    "book.synopsis": {
        text: "Synopsis",
        description: "Label before a story's summary in a plain-text export.",
    },
    "book.credit": {
        text: "Story text and art by Hypergryph and Yostar, from Arknights.",
        description: "Credit line on the colophon page and at the end of an exported book.",
    },
    "book.madeWith": {
        text: "Made with myrtle.moe.",
        description: "Line on the colophon page naming the site that made the file.",
    },
    "book.rights": {
        text: "Arknights and all of its story text and art belong to Hypergryph and Yostar. This book is a personal reading copy.",
        description: "Rights line on the colophon page of an exported book.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
