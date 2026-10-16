import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

/**
 * Trevor, the story companion in the reader (provisional, behind
 * `VITE_TREVOR=1`). "Trevor" is the feature's name and stays as-is.
 */
export const messages = {
    "trevor.open": {
        text: "Trevor",
        description: "Reader toolbar button that opens Trevor, the story companion panel (recap, characters, terms, questions). A name; keep it short.",
    },
    "trevor.sheet.title": {
        text: "Trevor",
        description: "Title of the story companion panel. A name.",
    },
    "trevor.sheet.subtitle": {
        text: "Recaps and answers that stop at this story.",
        description: "One line under the panel title: nothing in the panel reveals anything after the story being read.",
    },
    "trevor.sheet.sections": {
        text: "Trevor sections",
        description: "Accessible name of the tab strip that switches between the panel's sections.",
    },
    "trevor.tab.recap": {
        text: "Story so far",
        description: "Tab with a recap of the chapter up to the story being read.",
    },
    "trevor.tab.who": {
        text: "Who's here",
        description: "Tab listing the characters who speak in this story.",
    },
    "trevor.tab.terms": {
        text: "Terms",
        description: "Tab listing places, groups and concepts named in this story, with short definitions.",
    },
    "trevor.tab.ask": {
        text: "Ask",
        description: "Tab with suggested questions and a box to ask Trevor your own.",
    },
    "trevor.panel.loading": {
        text: "Loading Trevor's notes…",
        description: "Shown while the panel's data loads.",
    },
    "trevor.panel.none": {
        text: "Trevor has no notes for this story yet.",
        description: "Shown when the story has no panel (no script, or not built yet).",
    },
    "trevor.panel.error": {
        text: "Couldn't load Trevor's notes.",
        description: "Shown when loading the panel failed.",
    },
    "trevor.panel.retry": {
        text: "Try again",
        description: "Button that retries loading the panel.",
    },
    "trevor.recap.previous": {
        text: "Previously",
        description: "Heading over the summary of the story just before this one.",
    },
    "trevor.recap.soFar": {
        text: "Earlier in {group}",
        description: "Heading over short synopses of earlier stories in the same chapter. {group} is the chapter or event name.",
    },
    "trevor.recap.soFarPlain": {
        text: "Earlier in this chapter",
        description: "Heading over short synopses of earlier stories, used when the chapter name is unknown.",
    },
    "trevor.recap.earlierHidden": {
        text: "{count, plural, one {# earlier story is not listed.} other {# earlier stories are not listed.}}",
        description: "Note under the list of earlier stories when the chapter has more than the list shows.",
    },
    "trevor.recap.beforeYouStart": {
        text: "Before you start",
        description: "Heading over a short primer on what the chapter builds on, shown on a chapter's first stories.",
    },
    "trevor.recap.buildsOn": {
        text: "Builds on",
        description: "Label before a list of earlier chapters this chapter follows from.",
    },
    "trevor.recap.thisStory": {
        text: "This story's summary (spoilers for it)",
        description: "Toggle that reveals a summary of the story being read. It spoils this story only, so it starts closed.",
    },
    "trevor.recap.empty": {
        text: "No recap for this story yet.",
        description: "Empty state of the Story so far tab.",
    },
    "trevor.who.debut": {
        text: "First appearance",
        description: "Badge on a character who appears for the first time in this story.",
    },
    "trevor.who.firstSeen": {
        text: "First seen in {group}",
        description: "Where a character first appeared. {group} is a chapter or event name.",
    },
    "trevor.who.seenIn": {
        text: "{count, plural, one {Seen in # chapter} other {Seen in # chapters}}",
        description: "How many chapters a character has appeared in so far.",
    },
    "trevor.who.noIdentity": {
        text: "No spoiler-safe description yet.",
        description: "Shown under a character's name when there is nothing safe to say about them at this point.",
    },
    "trevor.who.more": {
        text: "{count, plural, one {# more character speaks.} other {# more characters speak.}}",
        description: "Note under the character list when more characters speak than are listed.",
    },
    "trevor.who.empty": {
        text: "No speaking characters found in this story.",
        description: "Empty state of the Who's here tab.",
    },
    "trevor.terms.firstNamed": {
        text: "First named in {group}",
        description: "Where a term was first mentioned. {group} is a chapter or event name.",
    },
    "trevor.terms.noDefinition": {
        text: "No spoiler-safe definition yet.",
        description: "Shown under a term when there is nothing safe to say about it at this point.",
    },
    "trevor.terms.more": {
        text: "{count, plural, one {# more term is named.} other {# more terms are named.}}",
        description: "Note under the term list when more terms are named than are listed.",
    },
    "trevor.terms.empty": {
        text: "No terms for this story.",
        description: "Empty state of the Terms tab.",
    },
    "trevor.terms.kind.place": { text: "Place", description: "Kind badge on a term: a location." },
    "trevor.terms.kind.nation": { text: "Nation", description: "Kind badge on a term: a country." },
    "trevor.terms.kind.organization": { text: "Organization", description: "Kind badge on a term: a group or company." },
    "trevor.terms.kind.event": { text: "Event", description: "Kind badge on a term: something that happened." },
    "trevor.terms.kind.concept": { text: "Concept", description: "Kind badge on a term: an idea, a disease, a technology." },
    "trevor.terms.kind.race": { text: "Race", description: "Kind badge on a term: one of the setting's peoples." },
    "trevor.suggest.heading": {
        text: "Suggested questions",
        description: "Heading over questions the reader can click to see or ask.",
    },
    "trevor.suggest.prepared": {
        text: "A prepared answer, spoiler-safe up to this story.",
        description: "Small note under a suggested question's answer, which was written ahead of time rather than just now.",
    },
    "trevor.ask.label": {
        text: "Your question about the story",
        description: "Accessible label of the question box.",
    },
    "trevor.ask.placeholder": {
        text: "Ask about what you've read so far",
        description: "Placeholder in the question box.",
    },
    "trevor.ask.submit": {
        text: "Ask",
        description: "Button that sends the question.",
    },
    "trevor.ask.hint": {
        text: "Answers only use the stories you have read, and never go past this one.",
        description: "Hint under the question box about the spoiler horizon.",
    },
    "trevor.ask.count": {
        text: "{count} / {max}",
        description: "Character counter of the question box, e.g. 42 / 500.",
    },
    "trevor.ask.signedOut": {
        text: "Sign in to ask Trevor. The recap, characters and terms are free to read.",
        description: "Shown in place of the question box to a visitor who is not signed in.",
    },
    "trevor.ask.signIn": {
        text: "Sign in",
        description: "Button that opens the sign-in dialog.",
    },
    "trevor.ask.spoiler": {
        text: "Spoiler-safe up to {era}",
        description: "Label on an answer: it uses nothing after this point of the story. {era} is a chapter name, e.g. Shatterpoint.",
    },
    "trevor.ask.spoilerNone": {
        text: "Spoiler-safe up to the very start. Mark stories read to widen it.",
        description: "Label on an answer when the reader has no stories marked read, so the answer can use almost nothing.",
    },
    "trevor.ask.sending": {
        text: "Sending…",
        description: "Status while the question is being sent.",
    },
    "trevor.ask.queued": {
        text: "Queued",
        description: "Status of a question waiting for an answerer.",
    },
    "trevor.ask.etaSeconds": {
        text: "{count, plural, one {about # second} other {about # seconds}}",
        description: "Expected wait for an answer, after Queued.",
    },
    "trevor.ask.etaMinutes": {
        text: "{count, plural, one {about # minute} other {about # minutes}}",
        description: "Expected wait for an answer, after Queued.",
    },
    "trevor.ask.position": {
        text: "{position, plural, one {Next in line} other {# in line}}",
        description: "The question's place in the queue.",
    },
    "trevor.ask.noWorker": {
        text: "No answerer is online right now. Your question waits in the queue.",
        description: "Shown while queued when nothing is online to answer.",
    },
    "trevor.ask.offline": {
        text: "Trevor's answerer went offline. Your question is still queued.",
        description: "Shown when the page stopped checking because nothing is online.",
    },
    "trevor.ask.checkAgain": {
        text: "Check again",
        description: "Button that resumes checking for the answer.",
    },
    "trevor.ask.statusOffline": {
        text: "Trevor's answerer is offline. Questions wait in a queue until it is back.",
        description: "Notice above the question box when nothing is online to answer.",
    },
    "trevor.ask.passagesOnly": {
        text: "Quick look: passages only",
        description: "Label on an answer that is the best-matching story passages rather than a written answer.",
    },
    "trevor.ask.passagesNote": {
        text: "No answerer was free, so these are the passages that best match your question.",
        description: "Explains a passages-only answer.",
    },
    "trevor.ask.failed": {
        text: "Trevor couldn't answer that. Try asking another way.",
        description: "Shown when the answerer failed on the question.",
    },
    "trevor.ask.verify": {
        text: "Couldn't confirm you're not a bot. Try again.",
        description: "Shown when the bot check refused the question (HTTP 403).",
    },
    "trevor.ask.busy": {
        text: "Trevor is busy. Try again in a minute.",
        description: "Shown when the queue is full (HTTP 503).",
    },
    "trevor.ask.rateLimited": {
        text: "You've reached this hour's question limit. Try again later.",
        description: "Shown when the account asked too many questions this hour (HTTP 429).",
    },
    "trevor.ask.tooMany": {
        text: "You already have questions waiting. Wait for one to finish.",
        description: "Shown when the account has too many open questions (HTTP 409).",
    },
    "trevor.ask.tooLong": {
        text: "That question is too long.",
        description: "Shown when the question was refused for its size (HTTP 413).",
    },
    "trevor.ask.sessionEnded": {
        text: "Your session ended. Sign in again to ask.",
        description: "Shown when the question was refused because the account is signed out (HTTP 401).",
    },
    "trevor.chat.label": {
        text: "Conversation with Trevor",
        description: "Accessible name of the scrolling list of questions and Trevor's answers in the Ask tab.",
    },
    "trevor.chat.jump": {
        text: "Jump to latest",
        description: "Button over the conversation, shown when the reader has scrolled up: scrolls to the newest message.",
    },
    "trevor.chat.enterHint": {
        text: "Enter to send, Shift+Enter for a new line",
        description: "Small hint under the question box on devices with a keyboard. Key names as printed on the keys.",
    },
    "trevor.chat.offlineFrozen": {
        text: "Trevor's answerer went offline before answering this one.",
        description: "Trevor's reply to an earlier question in the conversation whose answer never came because the answering service went offline.",
    },
    "trevor.answer.label": {
        text: "Trevor's answer",
        description: "Accessible name of the region that holds the answer.",
    },
    "trevor.answer.sources": {
        text: "Sources",
        description: "Heading over the story passages the answer cites.",
    },
    "trevor.answer.cite": {
        text: "Source {n}",
        description: "Accessible name of a citation marker in the answer text, e.g. Source 3.",
    },
    "trevor.answer.line": {
        text: "line {line}",
        description: "Where a cited passage starts in its story script.",
    },
    "trevor.answer.otherSource": {
        text: "Operator file or other record",
        description: "Label of a cited passage that is not a story scene the reader can open.",
    },
    "trevor.answer.openHere": {
        text: "Jump to this line",
        description: "Title of a citation that is in the story being read; clicking it jumps the reader there.",
    },
    "trevor.feedback.label": {
        text: "Was this answer helpful?",
        description: "Label of the thumbs up and down buttons under an answer.",
    },
    "trevor.feedback.up": {
        text: "Helpful",
        description: "Thumbs-up button under an answer.",
    },
    "trevor.feedback.down": {
        text: "Not helpful",
        description: "Thumbs-down button under an answer.",
    },
    "trevor.feedback.thanks": {
        text: "Thanks for the feedback.",
        description: "Shown after a vote was recorded.",
    },
    "trevor.primer.heading": {
        text: "Before you start",
        description: "Heading of a short primer in the chapter sheet on what the chapter builds on.",
    },
    "trevor.primer.note": {
        text: "Trevor's recap of what came before, with no spoilers for this chapter.",
        description: "Note under the chapter primer.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
