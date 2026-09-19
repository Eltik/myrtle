import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The hand-written release notes' own prose. Keyed at entry granularity - one
 * key per title, one per lead, one per bullet - because this is document
 * content, not UI chrome: a translator works through a whole entry at once and
 * the entry is what goes stale when it is rewritten.
 *
 * Every new entry in `entries.ts` needs its keys added here.
 */
export const namespace = "changelog";

export const messages = {
    "note.2026-09-19.title": {
        text: "Fixed: Base planning and upgrade advice",
        description: "Title of the 2026-09-19 release note. 'Base' is the player's RIIC facility; 'upgrade advice' is the panel that ranks what to invest in next.",
    },
    "note.2026-09-19.lead": {
        text: "The Base Optimizer now plans with the same economy the Score tab grades you on, so the staffing it shows is the staffing you are scored against. Trading posts are valued at their own level and crewed by what they actually earn. The Operators below milestone card lists only operators you have really invested in, so the gain it advertises is one you can reach. No score, breakdown or weight has moved.",
        description:
            "Lead paragraph of the 2026-09-19 release note, rendered as Markdown. The Base Optimizer and the Score tab are sections of a player profile; 'Operators below milestone' is a card inside the Score tab and should match its translation there; a 'trading post' is an RIIC room and keeps the game's name.",
    },
    "note.2026-09-19.item.1": {
        text: "The Base Optimizer and the Score tab now solve the same base, so the plan you are shown is the plan you are graded on. It seats combinations the planner used to miss.",
        description: "Bullet in the 2026-09-19 release note, filed under 'Improved'. The Base Optimizer and the Score tab are sections of a player profile.",
    },
    "note.2026-09-19.item.2": {
        text: "Promotion, potential and mastery badges on the roster are readable on the light theme again, and no longer sit on top of the module icons beside them.",
        description: "Bullet in the 2026-09-19 release note, filed under 'Improved'. Promotion, potential, mastery and modules are in-game operator upgrade systems.",
    },
    "note.2026-09-19.item.3": {
        text: "The language picker is in two places instead of three, and the header now shows which language you are reading rather than a globe.",
        description: "Bullet in the 2026-09-19 release note, filed under 'Improved'. The header shows the language's own code, such as EN or RU.",
    },
    "note.2026-09-19.item.4": {
        text: "In the Account Optimizer, EXP and LMD each read the same way: what is needed, what is short, and what you hold.",
        description: "Bullet in the 2026-09-19 release note, filed under 'Improved'. The Account Optimizer is a section of a player profile; EXP and LMD are in-game currencies and keep the game's names.",
    },
    "note.2026-09-19.item.5": {
        text: "A trading post is valued at its own level. A level 2 post standing next to a level 3 one made both of them count as level 2, which understated the order limit and order rarity of the better room.",
        description: "Bullet in the 2026-09-19 release note, filed under 'Fixed'. A trading post is an RIIC room; 'orders' are the deliveries it produces and keep the game's name.",
    },
    "note.2026-09-19.item.6": {
        text: "A trading post's crew is chosen by what the post actually earns beside the factories feeding it. A gold-starved base cannot sell bonus bars, so the squad that earns more per bar wins there, while the bonus-bar squad still wins when supply is spare.",
        description: "Bullet in the 2026-09-19 release note, filed under 'Fixed'. A 'bar' is a gold bar produced by an RIIC factory and sold at a trading post.",
    },
    "note.2026-09-19.item.7": {
        text: "The Control Center no longer seats an operator for a clue, HR or training bonus that a stronger operator already sitting there covers.",
        description: "Bullet in the 2026-09-19 release note, filed under 'Fixed'. The Control Center is an RIIC room; clues, HR and training are the kinds of bonus its base skills give and keep the game's names.",
    },
    "note.2026-09-19.item.8": {
        text: "Operators below milestone listed operators you had never promoted or levelled, and priced them as though you had. One section advertised a gain of 25.2 where only 13.8 was available. Your score, its breakdown and every weight are unchanged.",
        description: "Bullet in the 2026-09-19 release note, filed under 'Fixed'. 'Operators below milestone' is a card inside the Score tab and should match its translation there. The two numbers are grade points and are not formatted per locale.",
    },
    "note.2026-09-19.item.9": {
        text: "Page titles no longer collapse into a column of single letters on a phone, on the Stages page and anywhere else a heading shares a row with a counter.",
        description: "Bullet in the 2026-09-19 release note, filed under 'Fixed'.",
    },
    "note.2026-09-19.item.10": {
        text: "Roster cards show an operator's alternate name in full instead of cutting off its top.",
        description: "Bullet in the 2026-09-19 release note, filed under 'Fixed'. An 'alternate name' is the second name a variant operator carries, shown above the operator's own.",
    },
    "note.2026-09-19.item.11": {
        text: "In the translation editor, the message key no longer breaks into single characters on a phone, and turning a page returns you to the top of the list.",
        description: "Bullet in the 2026-09-19 release note, filed under 'Fixed'. The translation editor is the admin screen volunteer translators work in; a 'message key' is a string's identifier.",
    },
    "note.2026-09-19.item.12": {
        text: "The release note panel gets a real share of the screen on a phone rather than whatever was left over.",
        description: "Bullet in the 2026-09-19 release note, filed under 'Fixed'. The 'release note panel' is the list of secondary changes in this very dialog.",
    },
    "note.2026-09-18.title": {
        text: "Improved: Readability and pull planner",
        description: "Title of the 2026-09-18 release note, covering interface readability and changes to the pull planner.",
    },
    "note.2026-09-18.lead": {
        text: "The roster fits more operators per row at every width, and its promotion, potential and mastery badges are legible on the light theme again. The pull planner's goals say what they measure, its currencies carry the game's own icons, and setting a potential no longer stalls before the numbers move. Margins and page headings are now shared across the site. Korean, Japanese and Chinese render in a matching typeface throughout, labels that used to break mid-word hold together, and the translation editor now explains a message's plural forms and shows what changed when its English moves.",
        description: "Lead paragraph of the 2026-09-18 release note, rendered as Markdown. 'Roster' is a player's collection of operators; promotion, potential and mastery are in-game upgrade systems; a 'pull' is one gacha roll and a 'goal' here is a target number of copies.",
    },
    "note.2026-09-18.hrefLabel": {
        text: "Open the pull planner",
        description: "Label of the 2026-09-18 release note's call to action, which opens the release planner on its Pulls tab.",
    },
    "note.2026-09-18.item.1": {
        text: "The roster fits more operators per row at every width, and three per row on any phone rather than two on some and three on others.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Improved'. 'Roster' is a player's collection of operators.",
    },
    "note.2026-09-18.item.2": {
        text: "Orundum, Originite Prime, permits and certificates carry their own icons in the pull planner, and its smallest labels have been enlarged.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Improved'. Orundum, Originite Prime, Headhunting Permits and certificates are in-game currencies and keep the game's names.",
    },
    "note.2026-09-18.item.3": {
        text: "The pull planner's goals say what they measure: a specific rate-up, any rate-up, six copies of one. On a banner with more than two rate-ups the old wording claimed something the estimate never covered.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Improved'. A 'rate-up' is an operator with raised odds on a banner; a banner is a time-limited gacha pool.",
    },
    "note.2026-09-18.item.4": {
        text: "The Spark and Guarantee buttons ask for what you actually have to spend, with the banner's own free pulls already subtracted.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Improved'. A 'spark' is the pity exchange that trades a fixed number of pulls for an operator outright.",
    },
    "note.2026-09-18.item.5": {
        text: "Margins, page headings and the wording for players are consistent across the site; the leaderboard, search and profiles all say 'players' now.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Improved'. The site previously alternated between 'Doctors', the game's own word for the player, and 'Players'.",
    },
    "note.2026-09-18.item.6": {
        text: "Promotion ranks, potential ranks and mastery badges were nearly invisible on the light theme. They are legible again, and larger on phones.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Fixed'. Promotion, potential and mastery are in-game operator upgrade systems.",
    },
    "note.2026-09-18.item.7": {
        text: "Stepping an operator's potential in the pull planner no longer stalls for about a second before the numbers move.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Fixed'. 'Potential' is the in-game upgrade earned from duplicate copies of an operator.",
    },
    "note.2026-09-18.item.8": {
        text: "A banner's Clear button now clears the operators picked on it as well as the pulls, and a banner with no art of its own no longer borrows an unrelated event's picture or hides half its rate-up operators.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Fixed'. A 'rate-up' is an operator with raised odds on a banner.",
    },
    "note.2026-09-18.item.9": {
        text: "Profile, settings and tier-list pages no longer scroll sideways by a few pixels on a phone.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Fixed'.",
    },
    "note.2026-09-18.item.10": {
        text: "When a message's English is edited, its translation now shows the old and new wording word by word, instead of only being marked out of date.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Improved'. It describes the translation editor in the admin panel, which volunteer translators use.",
    },
    "note.2026-09-18.item.11": {
        text: "The translation editor explains a message's placeholders: which plural forms your language needs, which numbers each one covers, and the English wording for each.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Improved'. A 'placeholder' is a slot in a message that is filled with a value; a 'plural form' is one of the wordings a language selects between by number, such as one/few/many/other in Russian.",
    },
    "note.2026-09-18.item.12": {
        text: "Korean, Japanese and Chinese fell back to a mismatched typeface in small headings, and carried letter-spacing meant for the Latin alphabet that pushed the characters apart.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Fixed'. 'Letter-spacing' is the extra space set between characters.",
    },
    "note.2026-09-18.item.13": {
        text: "The appearance switch's labels no longer break mid-word in a language whose word for 'Dark' is wider than the button.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Fixed'. The 'appearance switch' is the light/dark/auto control in the site header.",
    },
    "note.2026-09-18.item.14": {
        text: "The leaderboard's Top movers heading no longer runs into the column label beside it when it is translated into something longer.",
        description: "Bullet in the 2026-09-18 release note, filed under 'Fixed'. 'Top movers' is the card listing the players whose rank changed most.",
    },
    "note.2026-09-17-3.title": {
        text: "Latin names for CN-only operators",
        description: "Title of the third 2026-09-17 release note. 'CN' is the Chinese game server's short name and stays as-is.",
    },
    "note.2026-09-17-3.lead": {
        text: "Operators that have not reached the global server yet now show under the Latin-script name the game gives them, so 予愿安洁莉娜 reads as Angelina the Mellow Wish. It is an appearance setting, on by default, and search finds these operators by either name.",
        description: "Lead paragraph of the third 2026-09-17 release note, rendered as Markdown. The Chinese example and its Latin name are game data and stay as-is. 'Global server' is the English-language game server.",
    },
    "note.2026-09-17-3.hrefLabel": {
        text: "Open appearance settings",
        description: "Label of the third 2026-09-17 release note's call to action, which opens the settings page where the switch lives.",
    },
    "note.2026-09-17-3.item.1": {
        text: "Appearance setting: show Latin names for CN-only operators. Applies to the operator list, operator pages, the planner and the search palette.",
        description: "Bullet in the third 2026-09-17 release note, filed under 'New'. 'CN' is the Chinese game server's short name and stays as-is.",
    },
    "note.2026-09-17-3.item.2": {
        text: "The search palette now includes CN-only operators and matches their Latin name as strongly as any other name.",
        description: "Bullet in the third 2026-09-17 release note, filed under 'Improved'. 'CN' is the Chinese game server's short name and stays as-is.",
    },
    "note.2026-09-17-2.title": {
        text: "Integrated Strategies regrade",
        description: "Title of the second 2026-09-17 release note. 'Integrated Strategies' is an in-game mode name and is never translated. A grade is this site's own letter rating of a player.",
    },
    "note.2026-09-17-2.lead": {
        text: "A bug in how Integrated Strategies runs were scored has been found and fixed. Every account has been regraded with the corrected scoring, so your Roguelike subscore, composite score and grade may differ from what you saw before.",
        description: "Lead paragraph of the second 2026-09-17 release note, rendered as Markdown. 'Integrated Strategies' is an in-game mode name and is never translated; 'Roguelike' is the label this site's leaderboard and profile give that mode's subscore. Composite score and grade are the site's own player rating.",
    },
    "note.2026-09-17-2.hrefLabel": {
        text: "See the leaderboard",
        description: "Label of the second 2026-09-17 release note's call to action, which opens the player leaderboard where grades and the Roguelike sort are shown.",
    },
    "note.2026-09-17-2.item.1": {
        text: "Integrated Strategies scoring counted some runs incorrectly; the scoring is now correct.",
        description: "Bullet in the second 2026-09-17 release note, filed under 'Fixed'. 'Integrated Strategies' is an in-game mode name and is never translated.",
    },
    "note.2026-09-17-2.item.2": {
        text: "All users have been regraded. Grades and leaderboard positions have been recomputed from the corrected scores.",
        description: "Bullet in the second 2026-09-17 release note, filed under 'Fixed'. A grade is this site's own letter rating of a player.",
    },
    "note.2026-09-17.title": {
        text: "Pull planner",
        description: "Title of the 2026-09-17 release note. 'Pull' is the community's word for a single gacha roll.",
    },
    "note.2026-09-17.lead": {
        text: "The planner's Pulls tab is live. It holds one ledger of your rolls across every upcoming banner, prices each goal against the pity you will actually be carrying when that banner lands, and shows what committing early to one banner costs you at the next. Login, roster and settings fixes from Discord feedback ship alongside it.",
        description: "Lead paragraph of the 2026-09-17 release note, rendered as Markdown. 'Pity' is the game's guarantee counter that improves the odds as rolls go without a top-rarity result; 'banner' is a time-limited gacha pool.",
    },
    "note.2026-09-17.hrefLabel": {
        text: "Open the pull planner",
        description: "Label of the 2026-09-17 release note's call to action, which opens the release planner on its Pulls tab.",
    },
    "note.2026-09-17.item.1": {
        text: "Added a pull planner: budget one pool of rolls across upcoming banners and see which goals you can actually afford.",
        description: "Bullet in the 2026-09-17 release note, filed under 'New'. 'Rolls' and 'banners' are gacha terms.",
    },
    "note.2026-09-17.item.2": {
        text: "Pull odds are computed exactly, and cover every guarantee the game ships: soft and hard pity, rate-up guarantees, and the Limited spark.",
        description: "Bullet in the 2026-09-17 release note, filed under 'New'. 'Soft pity' is the rising-rate window, 'hard pity' the forced result, and 'spark' the exchange after a set number of rolls on a Limited banner. Keep the game's own terms.",
    },
    "note.2026-09-17.item.3": {
        text: "Rosters can be sorted by investment, scoring each operator against its own ceiling rather than against the roster.",
        description: "Bullet in the 2026-09-17 release note, filed under 'New'. 'Investment' means how far an operator has been levelled and upgraded.",
    },
    "note.2026-09-17.item.4": {
        text: 'Signing in now asks whether to keep your game credentials with a "Keep me synced" choice, and declining deletes anything an earlier sign-in stored.',
        description: "Bullet in the 2026-09-17 release note, filed under 'New'. 'Keep me synced' is the control's own label and should be translated as a label.",
    },
    "note.2026-09-17.item.5": {
        text: "Settings now has a single Account section in place of separate Profile and Account & data panels.",
        description: "Bullet in the 2026-09-17 release note, filed under 'Improved'. 'Account', 'Profile' and 'Account & data' are settings panel names.",
    },
    "note.2026-09-17.item.6": {
        text: "Planner events read as their in-game names instead of raw tags, a banner matched only by date says so, and the calendar keeps its month header and legend in view while you scroll.",
        description: "Bullet in the 2026-09-17 release note, filed under 'Improved'.",
    },
    "note.2026-09-17.item.7": {
        text: 'First sign-in no longer fails with "Failed to fetch user data", and signing in on mobile no longer signs you straight back out.',
        description: "Bullet in the 2026-09-17 release note, filed under 'Fixed'. The quoted string is an error message the user saw in English; keep it recognisable.",
    },
    "note.2026-09-17.item.8": {
        text: "The roster's view mode and your chosen profile tab stay where you left them.",
        description: "Bullet in the 2026-09-17 release note, filed under 'Fixed'. 'View mode' is the roster's icons-or-list toggle.",
    },
    "note.2026-09-17.item.9": {
        text: "Removed the leaderboard's Skins sort, which ranked every player at 0.0%.",
        description: "Bullet in the 2026-09-17 release note, filed under 'Fixed'. 'Skins' are operator outfits.",
    },
    "note.2026-09-16.title": {
        text: "Release & event planner",
        description: "Title of the 2026-09-16 release note. The planner forecasts when CN content (operators, events, skins) reaches the global server.",
    },
    "note.2026-09-16.lead": {
        text: "A new release and event planner forecasts when upcoming operators, events, and skins arrive on the global server, so you can plan your pulls ahead of time. Translations have also started rolling out.",
        description: "Lead paragraph of the 2026-09-16 release note, rendered as Markdown. 'Pulls' is the community's word for gacha rolls.",
    },
    "note.2026-09-16.hrefLabel": {
        text: "Open the planner",
        description: "Label of the 2026-09-16 release note's call to action, which opens the release and event planner.",
    },
    "note.2026-09-16.item.1": {
        text: "Added release & event planner feature.",
        description: "Bullet in the 2026-09-16 release note, filed under 'New'.",
    },
    "note.2026-09-16.item.2": {
        text: "Added translation support.",
        description: "Bullet in the 2026-09-16 release note, filed under 'New'.",
    },
    "note.2026-09-16.item.3": {
        text: "Improved/fixed base optimizer (daily LMD, control center, perception).",
        description: "Bullet in the 2026-09-16 release note, filed under 'Improved'. 'LMD' is the game's currency; 'control center' and 'perception' are base-building terms.",
    },
    "note.2026-09-16.item.4": {
        text: "Improved dynamic illustration.",
        description: "Bullet in the 2026-09-16 release note, filed under 'Improved'. 'Dynamic illustration' is the game's name for animated operator art.",
    },
    "note.2026-09-16.item.5": {
        text: "Workflow and security fixes.",
        description: "Bullet in the 2026-09-16 release note, filed under 'Fixed'.",
    },
    "note.2026-09-10.title": {
        text: "Build statistics",
        description: "Title of the 2026-09-10 release note. 'Build' here means how players level and equip an operator.",
    },
    "note.2026-09-10.lead": {
        text: "Based on community statistics, operators now display what users build for masteries and modules. More bug fixes and grading have been improved.",
        description: "Lead paragraph of the 2026-09-10 release note, rendered as Markdown. 'Mastery' and 'module' are the game's own names for skill and equipment upgrades.",
    },
    "note.2026-09-10.hrefLabel": {
        text: "Browse operators",
        description: "Label of the 2026-09-10 release note's call to action, which opens the operator index.",
    },
    "note.2026-09-10.item.1": {
        text: "Added changelog notifications.",
        description: "Bullet in the 2026-09-10 release note, filed under 'New'.",
    },
    "note.2026-09-10.item.2": {
        text: "Added build statistics.",
        description: "Bullet in the 2026-09-10 release note, filed under 'New'.",
    },
    "note.2026-09-10.item.3": {
        text: "Display breakpoints and percentages for mastery/module levels.",
        description: "Bullet in the 2026-09-10 release note, filed under 'New'. A breakpoint is the level at which a skill's numbers jump.",
    },
    "note.2026-09-10.item.4": {
        text: "Operator skills/modules default to what is most used.",
        description: "Bullet in the 2026-09-10 release note, filed under 'Improved'.",
    },
    "note.2026-09-10.item.5": {
        text: "Operator grades were computed from the wrong baseline. Scores across the roster have shifted.",
        description: "Bullet in the 2026-09-10 release note, filed under 'Improved'. A grade is this site's own letter rating of an operator.",
    },
    "note.2026-09-10.item.6": {
        text: "Profile rosters no longer play E2 dynamic art over an operator who has not reached E2.",
        description: "Bullet in the 2026-09-10 release note, filed under 'Fixed'. 'E2' is the game's second promotion tier and stays as-is.",
    },
    "note.2026-09-10.item.7": {
        text: "Check-in dates and Reclamation Algorithm scoring.",
        description: "Bullet in the 2026-09-10 release note, filed under 'Fixed'. 'Reclamation Algorithm' is a game mode name and is never translated.",
    },
} satisfies MessageMap;

// `dynamic`: every key here is reached through a field on a RELEASE_NOTES entry
// (`t(note.titleKey)`, `t(item.textKey)`), never as a literal, so the extractor
// has no call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
