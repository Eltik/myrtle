# Arknights story reading order: what the community recommends

Research date: 2026-09-29. Purpose: a baseline to compare an automatically built reading guide against.

## Method and sources

- **Reddit r/arknights**, through the Arctic Shift archive (reddit.com itself is blocked here).
  - Post search for "reading order", "story order", "chronological", "where to start", "essential events", "event order", "timeline" gave 335 unique posts, 60 of them on-topic. I fetched full comment trees for 57 of those threads, including the 5 most-linked guide threads.
  - Most story-order questions get removed under Rule 5 ("use the megathreads"), so the real advice lives in the weekly question threads. I sampled every 5th thread from 724 threads ("Daily Questions Megathread" 2020 to 2022, "Help Center and Megathread Hub" 2022 to 2026). That was 145 threads and 159,487 comments, filtered for reading-order language.
  - In total, 2,522 comments were screened. Around 250 were read by hand.
- **arknights.wiki.gg**, via the MediaWiki API. Pages used: [Arknights Timeline](https://arknights.wiki.gg/wiki/Arknights_Timeline), [Story/Movements](https://arknights.wiki.gg/wiki/Story/Movements), and the 14 Movement subpages. The subpages carry the in-game "related story" links, which the wiki describes as stories "recommended to be read first".
- **Community guides** that are linked again and again in the threads:
  - G1: the 2025 community reading guide (Google Doc). [doc](https://docs.google.com/document/d/1j6erNIDEeDz4PfVW7XGoGkKVfgNyjlHnzevvOKNhTc8/), [post](https://www.reddit.com/r/arknights/comments/1mmxksx/). It offers three methods (release, timeline, storyline) and recommends the storyline method. It was the most-linked guide in 2025 and 2026. The timo.beer web reader's "reading order" page is taken from G1 ([guides.json](https://arknights.timo.beer/json/guides.json) says so), so it is not an independent source.
  - G2: the 2023 "don't know where to start" infographic. [post](https://www.reddit.com/r/arknights/comments/13utigi/), 171 comments.
  - G3: "Reading Guide to Arknights, Version 2.0" (Sept 2024), a gallery of storyline maps. [post](https://www.reddit.com/r/arknights/comments/1ffn2cj/), 78 comments.
  - G4: a release-order checklist with Global dates. [post](https://www.reddit.com/r/arknights/comments/1bclc6v/)
  - G5: a release-order Google Sheet. [sheet](https://docs.google.com/spreadsheets/d/1yio_STPNsP1lnkjbVTJC5acCLQ1HurGqo-C5haKFpU0/), posts [1](https://www.reddit.com/r/arknights/comments/16tudb3/) and [2](https://www.reddit.com/r/arknights/comments/1wfneld/)
  - G6: a hybrid reading order from a returning player (2026). [post](https://www.reddit.com/r/arknights/comments/1sdczib/)
  - G7: Terra Log's reading-order page. [page](https://terra-log.org/en/guide/reading-order)
- **Counting rules.**
  - An "independent source" is a distinct commenter, or a distinct guide. Comment counts are given next to commenter counts because some regulars repeat the same advice. One 2022 to 2023 regular posted a near-identical "must-watch order" copypasta at least 8 times.
  - Placement counts come from pattern matching (the event name within 120 characters of the anchor, such as "before ch 7" or "after chapter 8"), with the top rules checked by hand. Treat them as close estimates, not exact figures.
  - All links are reddit permalinks: `/comments/<post>/_/<comment>/`.

## 1. Recommendation patterns

| Pattern | Sources that recommend it (hand count, threads read) | Typical reasons given |
|---|---|---|
| **Release order** (Global unlock order, main story and events interleaved) | About 40 commenters, plus G4, G5 and G7. "Release order" appears in 162 screened comments in total. | "the order the devs intended" ([hy7eh84](https://www.reddit.com/r/arknights/comments/szes46/_/hy7eh84/)). Keeps foreshadowing and reveals in place ([p07w9hx](https://www.reddit.com/r/arknights/comments/1v84q8e/_/p07w9hx/)). Older events have easier stages ([p881h08](https://www.reddit.com/r/arknights/comments/1w3afcj/_/p881h08/)). "Optimal reading order is just equivalent to release order" ([k9c5ozr](https://www.reddit.com/r/arknights/comments/17v1ie9/_/k9c5ozr/)). |
| **Storyline / Movement order** (main story first, then one faction arc at a time) | About 10 commenters, plus G1 (its final recommendation), G2 and G3 (both drawn as storyline maps) | Keeps each arc together. Avoids jumping between plots. "Story arc order is probably the best way... escalating sense of quality" ([n83dayc](https://www.reddit.com/r/arknights/comments/1mmxksx/_/n83dayc/)). Since the Sept 2025 Global update, the game's own Terminal "Movements" view sorts this way by default ([p03j94b](https://www.reddit.com/r/arknights/comments/1v84q8e/_/p03j94b/)). |
| **Main story first, then events, with a short list of must-read intermezzi slotted in** | About 12 commenters (for example [lgzkf65](https://www.reddit.com/r/arknights/comments/1emjihr/_/lgzkf65/), [nfc7q13](https://www.reddit.com/r/arknights/comments/1nhiqbp/_/nfc7q13/), [mys3ssy](https://www.reddit.com/r/arknights/comments/1lfxmkx/_/mys3ssy/), [lb71zc2](https://www.reddit.com/r/arknights/comments/1dt3bl3/_/lb71zc2/)) | "Most events... don't have much relationship to the main story". Only the intermezzi matter for the main plot ([md6oi68](https://www.reddit.com/r/arknights/comments/1im3kik/_/md6oi68/)). |
| **Hybrid** (release order, but with prequels and sequels pulled next to each other) | G6, the copypasta "must-watch order" (1 author, about 8 posts, for example [hvqhn8i](https://www.reddit.com/r/arknights/comments/sjgez2/_/hvqhn8i/)), and 3 or 4 other commenters | Keeps the reveals at the right moments while keeping related stories together. |
| **Chronological (in-universe) order** | About 3 recommend it, all with caveats ([mvr1g2l](https://www.reddit.com/r/arknights/comments/1l1f39q/_/mvr1g2l/): "outside of various prequels... chronological order is also fine"). At least 6 advise against it ([or4piuq](https://www.reddit.com/r/arknights/comments/1u3dfv6/_/or4piuq/): "you absolutely should not read the events in in-universe chronological order"; [or4mx6t](https://www.reddit.com/r/arknights/comments/1u3dfv6/_/or4mx6t/); [hy7eh84](https://www.reddit.com/r/arknights/comments/szes46/_/hy7eh84/); [k9c5ozr](https://www.reddit.com/r/arknights/comments/17v1ie9/_/k9c5ozr/); G1; G5) | Against: Babel, Darknights Memoir and A Walk in the Dust come first in the timeline and spoil the Doctor's amnesia reveal (G1 method 2, cons). Many stories have no fixed date. Chronological order is asked about far more often than it is recommended: 88 screened comments mention it. |
| **Anime for Episodes 0 to 8, then start reading at Episode 9** | 3 commenters ([o0w4ry5](https://www.reddit.com/r/arknights/comments/1qj4guk/_/o0w4ry5/), [o0w6039](https://www.reddit.com/r/arknights/comments/1qj4guk/_/o0w6039/), [o0wf0fd](https://www.reddit.com/r/arknights/comments/1qj4guk/_/o0wf0fd/)) | Early chapters are "overly verbose, slightly unpolished". The anime covers Episodes 0 to 8. The caveat is that the anime compresses Darknights Memoir ([o0wal7p](https://www.reddit.com/r/arknights/comments/1qj4guk/_/o0wal7p/)). |

Two more things to know:

- Since the Global engine update in about September 2025, many answers simply point to the in-game Movements view, with its "Unlock time" sort and related-story branches. Examples: [ngpsnt7](https://www.reddit.com/r/arknights/comments/1nsyxws/_/ngpsnt7/), [neumhyb](https://www.reddit.com/r/arknights/comments/1nhiqbp/_/neumhyb/), [ney7oof](https://www.reddit.com/r/arknights/comments/1nhiqbp/_/ney7oof/).
- Several people warn that a Movement's in-game link can be misread. Path of Life's link spoils Lone Trail and Episode 14 if it is read too early ([nrb31to](https://www.reddit.com/r/arknights/comments/1mmxksx/_/nrb31to/)).

## 2. Events named as essential for the main story

This counts sources that name the event as important or recommended for following the Main Theme. The main number is distinct commenters, with comments in brackets. "In-game" means the event is linked from a Main Theme episode as a related story on the wiki's [Main Theme movement](https://arknights.wiki.gg/wiki/Story/Movements).

| # | Event | Commenters (comments) | Guides | In-game link | Example |
|---|---|---|---|---|---|
| 1 | Darknights Memoir | 31 (50) | G1, G2, G3, G6 | yes (before Ep 7) | "this is the single most important one in here" ([jhbvltk](https://www.reddit.com/r/arknights/comments/12pbt1n/_/jhbvltk/)); "basically the real chapter7" ([ido3qlo](https://www.reddit.com/r/arknights/comments/vjlrtc/_/ido3qlo/)) |
| 2 | Babel | 14 (17) | G1, G6 | yes (before Ep 14) | "a must-read before chapter 14 because it tells the story of the Doctor's past" ([nep2uoz](https://www.reddit.com/r/arknights/comments/1nhiqbp/_/nep2uoz/)) |
| 3 | A Walk in the Dust | 11 (18) | G1, G2, G3, G6 | yes (before Ep 10) | "read Vigilo and Walk in the Dust asap after ch8 since those two events help give context to basically everything going forward" ([n83yogv](https://www.reddit.com/r/arknights/comments/1mmxksx/_/n83yogv/)) |
| 4 | What the Firelight Casts | 12 (14) | G1, G3, G6 | yes (before Ep 12) | "What the Firelight casts comes after chapter 9 or 10, so that's when it should be read" ([oekyisa](https://www.reddit.com/r/arknights/comments/1sdczib/_/oekyisa/)) |
| 5 | Vigilo | 8 (15) | G1, G2, G3, G6 | no (it is a vignette) | "why is vigilo considered optional? its pretty major for learning about pre-amnesia doctor" ([n82b5hl](https://www.reddit.com/r/arknights/comments/1mmxksx/_/n82b5hl/)). G1 changed Vigilo from optional to required after that comment. |
| 6 | Lone Trail (with Dorothy's Vision as its prerequisite) | 10 (11) | G1 ("the story kind of assume[s]... you've already read Lone Trail" before Ep 15), G3, G6 | via Path of Life and Retracing Our Steps | "Lone Trail ties into the main story" ([ogq8p3c](https://www.reddit.com/r/arknights/comments/1sk7ghz/_/ogq8p3c/)) |
| 7 | See You Soon / Ending a Grand Overture / When Elegies Are Ashes (the post-Ep 14 cluster) | 5 (5) | G1, G6 | yes (EAGO before Ep 15) | "see you soon is pretty much vital cause it's the ending of the war" ([oel69nf](https://www.reddit.com/r/arknights/comments/1sdczib/_/oel69nf/)); read "All the Unrevealed" from See You Soon before Ep 15 ([nel45nb](https://www.reddit.com/r/arknights/comments/1nhiqbp/_/nel45nb/)) |
| 8 | Operational Intelligence (only the "Anonymous One's War" and "Destroyer" stories) | 4 (6) | G1 (optional), G3 | yes (after Ep 4) | "It's necessary to read Operational Intelligence before reading Chapter 7. Otherwise, it's hard to understand the behavior of Patriot." ([ie0de4m](https://www.reddit.com/r/arknights/comments/vm3ke4/_/ie0de4m/)) |
| 9 | Path of Life | 5 (5) | G1 ("skippable but nice") | via Glimpse of the Depths | "the big ones that connect to the main story... Lone Trail after episode 8, Path of Life after episode 14 and The Masses' Travels after episode 15" ([o3qfw60](https://www.reddit.com/r/arknights/comments/1qts1st/_/o3qfw60/)) |
| 10 | The Masses' Travels | 5 (5) | G1 (optional before Ep 16) | links back to Ep 15 | same comment as row 9; "You should definitely read Episode 15 before starting The Masses Travel" (G1) |

Named less often:

- **Children of Ursus**: in-game link from Ep 1. G1 says it is needed for People, A People. Three sources.
- **A Light Spark in Darkness**: introduces Red, who appears in Ep 12. Named by G1 plus 3 commenters, for example [jm3e51i](https://www.reddit.com/r/arknights/comments/13utigi/_/jm3e51i/).
- **Ch'en's Operator Record "In One Effort"** (between Ep 5 and Ep 6): G3, G6 and [n83d1w9](https://www.reddit.com/r/arknights/comments/1mmxksx/_/n83d1w9/).
- **Near Light**: before Ep 10, and "especially" before Ep 14. G1 plus 1 commenter ([o6pai7w](https://www.reddit.com/r/arknights/comments/1mmxksx/_/o6pai7w/)).
- **The Rhine Lab manga** (Records of Originium), which comes before Lone Trail: 5 commenters.

## 3. Placement rules

| Rule | Commenters (comments) | Other sources | Example |
|---|---|---|---|
| Darknights Memoir after Ep 6, before Ep 7 | 31 (50) | G1, G2, G3, G6, in-game link | "highly recommended you read darknightmemoir after ch6 and before ch7" ([hw4yl0a](https://www.reddit.com/r/arknights/comments/snhjme/_/hw4yl0a/)) |
| Babel before Ep 14 (most say right after Ep 13, as "chapter 13.5") | 14 (17) | G1, in-game link | "Babel works really well as a chapter 13.5" ([o3b4iy1](https://www.reddit.com/r/arknights/comments/1qts1st/_/o3b4iy1/)); "Chapter 13 > Babel > Chapter 14" ([meu64m4](https://www.reddit.com/r/arknights/comments/1ffn2cj/_/meu64m4/)) |
| A Walk in the Dust after Ep 8 (and before Ep 10) | 11 (18) | G1, G2, G3, G6, in-game (read Ep 7 first; read it before Ep 10) | "a walk in the dust should be read anytime after chapter8 and before chapter10" ([ido3qlo](https://www.reddit.com/r/arknights/comments/vjlrtc/_/ido3qlo/)) |
| Dorothy's Vision before Lone Trail | 10 (11) | G1, G3, G6 | "you need to read Dorothy's Vision first for Lone Trail" ([p1ep19r](https://www.reddit.com/r/arknights/comments/1v7x3od/_/p1ep19r/)) |
| Under Tides before Stultifera Navis | 11 (12) | G1, G3 | "the upcoming Stultifera Navis is a sequel to it so... I'd definitely read this first" ([itruo5m](https://www.reddit.com/r/arknights/comments/yc892o/_/itruo5m/)) |
| Lone Trail after Ep 12 to 14 (the 2025 to 2026 consensus is after Ep 14) | 10 (11) | G1, G6 | "Lone Trail takes place after Chapter 14, so it might be best to read that one after 14 but before 15" ([nrlxcm6](https://www.reddit.com/r/arknights/comments/1p5e4y6/_/nrlxcm6/)) |
| Maria Nearl (and Pinus Sylvestris) before Near Light | 9 (11) | G1, G3, wiki movement order | "Pinus Sylvestris -> Maria Nearl -> Near Light" ([ilab7at](https://www.reddit.com/r/arknights/comments/wowodc/_/ilab7at/)) |
| Vigilo after Ep 8, before Ep 9 or Ep 10 | 8 (15) | G1, G2, G3, G6 | "walk in the dust and vigilo should be read anytime after ch8 and before ch10" ([jubkmwf](https://www.reddit.com/r/arknights/comments/15eczib/_/jubkmwf/)) |
| Grani and the Knights' Treasure before Under Tides | 8 (8) | G1 | "Grani event should probably be done before Under Tides because it serves as a Skadi introduction" ([mc5lr80](https://www.reddit.com/r/arknights/comments/1im3kik/_/mc5lr80/)) |
| What the Firelight Casts after Ep 11, before Ep 12 | 7 (7) | G1 (between Ep 11 and 12), in-game (before Ep 12) | "wtfc happens directly after ch11" ([oel69nf](https://www.reddit.com/r/arknights/comments/1sdczib/_/oel69nf/)) |
| What the Firelight Casts after Ep 9 (at least) | 6 (7) | in-game (read Ep 9 first) | "What the Firelight Casts shouldn't be read until after chapter 9" ([m0fxua2](https://www.reddit.com/r/arknights/comments/1h4sgn3/_/m0fxua2/)) |
| Rhine Lab manga before Lone Trail | 5 (6) | none | "Rhine Lab manhwa should be read before the Lone Trail" ([n86lbji](https://www.reddit.com/r/arknights/comments/1mmxksx/_/n86lbji/)) |
| A Kazdelian Rescue (and IS5) after Ep 14 | 6 (6) | G1, in-game link | "IS5 is set on the aftermath of Chapter 14" ([lmxxo5a](https://www.reddit.com/r/arknights/comments/1ffn2cj/_/lmxxo5a/)) |
| Operational Intelligence ("Anonymous One's War") before Ep 7, after Ep 3 to 5 | 4 (6) | G3, in-game (after Ep 4) | "after ch3 and before darknights read the cutscenes 'anonymous one's war' and 'destroyer'" ([jhbvltk](https://www.reddit.com/r/arknights/comments/12pbt1n/_/jhbvltk/)) |
| Code of Brawl before Il Siracusano | 5 (5) | G1, in-game link | "I believe you should read Code of Brawl first either way" ([jl8ts6h](https://www.reddit.com/r/arknights/comments/13onno7/_/jl8ts6h/)) |
| Path of Life after Ep 14 and Lone Trail | 5 (5) | G1, in-game link | "Path of Life event rather explicitly spoils elements from Lone Trail and Chapter 14" ([nrb31to](https://www.reddit.com/r/arknights/comments/1mmxksx/_/nrb31to/)) |
| The Masses' Travels after Ep 15 | 5 (5) | G1, in-game link | "The Masses' Travels happens basically right after Episode 15" ([oekqdgr](https://www.reddit.com/r/arknights/comments/1sdczib/_/oekqdgr/)) |
| See You Soon / EAGO / Elegies after Ep 14, before Ep 15 | 5 (5) | G1, in-game (EAGO before Ep 15) | "See you soon > Ending a grand Overture > A Kazdelian rescue" ([nbh7sf4](https://www.reddit.com/r/arknights/comments/1n3zote/_/nbh7sf4/)) |
| Lingering Echoes and Hortus de Escapismo before Zwillingsturme im Herbst | 4 (4) | G1, in-game (Hortus) | "just read Lingering and Hortus before Zwillingsturme" ([mxan4id](https://www.reddit.com/r/arknights/comments/1l1f39q/_/mxan4id/)) |
| Episodes 7 and 8 before Lone Trail | 5 (6) | G1 | "It's recommended to read Episode 7 and Episode 8 before starting Lone Trail" (G1) |
| Ep 8 before I Portatori dei Velluti ("will spoil its biggest twist") | 1 | G1 | G1, Siracusa plotline note |
| Children of Ursus before People, A People | 1 | G1, in-game link | "for people, a people it's better to have read children of ursus" ([p9bh57s](https://www.reddit.com/r/arknights/comments/1we7dg8/_/p9bh57s/)) |

The Babel placement is the one real disagreement:

- **Before Ep 14** (usually right after Ep 13) is the majority view: 14 commenters and G1.
- **Any time after Vigilo or Ep 8**: 4 commenters. Examples: [nfca2nu](https://www.reddit.com/r/arknights/comments/1nhiqbp/_/nfca2nu/), "Babel can be read at any time after Vigilo and before chapter 14"; [nr2qhhl](https://www.reddit.com/r/arknights/comments/1p5e4y6/_/nr2qhhl/); [o3rywqp](https://www.reddit.com/r/arknights/comments/1mmxksx/_/o3rywqp/). A new player reports that guides "keep [giving] different answers... before vigilo, and before chapter 14" ([1qwsdnd](https://www.reddit.com/r/arknights/comments/1qwsdnd/)).
- **G6** puts Babel after Lone Trail and before Ep 13. A commenter corrected it ([oel69nf](https://www.reddit.com/r/arknights/comments/1sdczib/_/oel69nf/)).
- **Some recommend Lone Trail before Babel** ([mvwy13z](https://www.reddit.com/r/arknights/comments/1l1f39q/_/mvwy13z/)).

## 4. In-world chronology compared with release order

In-world dates come from the wiki [Arknights Timeline](https://arknights.wiki.gg/wiki/Arknights_Timeline), matched by section, and from G1 method 2. Global release dates come from G4.

| Story | Global release position | In-world placement | Conflict |
|---|---|---|---|
| A Walk in the Dust | After Ep 8 (2021-09) | 1070s to 1080s, the earliest event (G1: "happen before anything else in the timeline") | Chronological order would put it first |
| Babel | After Ep 13, before Ep 14 (Global Oct 2024) | Kazdel Civil War, 1090 to 1094 | Chronologically before the Prologue, but almost everyone reads it at Ep 13.5 |
| Darknights Memoir | Between Ep 6 and 7 (2020-12) | About 1094 to 1096. G1 places it "in the middle of Babel. Around BB-7" | Before the Prologue. It spoils the Doctor's amnesia if read first (G1) |
| Children of Ursus | Before Darknights (2020-09) | Flashbacks during the Chernobog crisis (Dec 1096 to Jan 1097); G1 places it between Ep 3 and 4 | Minor |
| Near Light, Maria Nearl, Pinus Sylvestris | Near Light after Ep 9 (2022-04) | 24th Kazimierz Major, June to Nov 1097, before Ep 9 | Released after Ep 9, set before it. G1 advises reading it after Ep 9 anyway |
| Dossoles Holiday, Code of Brawl, Heart of Surging Flame, Twilight of Wolumonde, Break the Ice | Scattered, 2020 to 2022 | 1097, in the "two year gap" between Ep 8 and Ep 9 (G2) | Most side stories sit in this gap or after the Victoria arc ([idyvkvz](https://www.reddit.com/r/arknights/comments/vm3ke4/_/idyvkvz/)) |
| Guide Ahead | Before Ep 10 (2022-09) | March 1099, after the Victoria war ([jhaalbl](https://www.reddit.com/r/arknights/comments/12v9a9y/_/jhaalbl/)) | Released mid Victoria arc, set after it |
| Il Siracusano | Between Ep 11 and 12 (2023-05) | Volsinii unrest, Oct 1099 (wiki timeline) | Set after Ep 14. "Il Siracusano, Dorothy Vision... and all the Yan events take place after the stuffs in Victoria has been resolved" ([jm3bcv7](https://www.reddit.com/r/arknights/comments/13utigi/_/jm3bcv7/)) |
| Dorothy's Vision | Before Ep 11 (2023-03) | 1099 | Set after Ep 14 |
| Lone Trail | Between Ep 12 and 13 (2023-11) | Nov 21, 1099 ([12v9a9y thread](https://www.reddit.com/r/arknights/comments/12v9a9y/)) | Set after Ep 14. Readers now mostly place it after Ep 14 |
| See You Soon | After Ep 14 | Takes place during Ep 14 (G1) | Minor |
| Episode 15 | 2025 | Nodes 15-1 and 15-2 follow Ep 14 directly (1098); the rest is about 1101, after roughly half of all side stories (G1) | G1 splits Ep 15 in its timeline order |
| Yan / Sui arc (Ancient Forge, Who is Real, Invitation to Wine, Where Vernal Winds, Here A People Sows, Fantasy in the Mirage, First of a Thousand Autumns) | Ancient Forge 2020, the rest 2021 to 2026 | 1101 to 1102, among the latest events | Ancient Forge was released in year 1 but sits near the end of the timeline. A reader said the Code of Brawl and Ancient Forge placements "confused me a lot" ([oel28ql](https://www.reddit.com/r/arknights/comments/1sdczib/_/oel28ql/)) |
| Come Catastrophes or Wakes of Vultures | 2024 | Before Originium Dust by some readings ([1j10k47](https://www.reddit.com/r/arknights/comments/1j10k47/)) | Unresolved |

G1's timeline order starts: A Walk in the Dust, Babel, Darknights Memoir, Prologue to Ep 3, Children of Ursus, Ep 4 to 8, Mansfield Break, Grani, Heart of Surging Flame, Dossoles, Maria Nearl, Pinus Sylvestris, Near Light, Wolumonde, Code of Brawl, A Light Spark in Darkness, Break the Ice, Ep 9, Great Chief Returns, Ep 10, Ep 11, What the Firelight Casts, Ep 12 to 14, See You Soon, Ep 15 (15-1, 15-2), and Ending a Grand Overture. Its own verdict is that this order is "technically impossible" and spoils reveals.

## 5. What not to skip, and what is called optional

**Do not skip** (each with its main reason):

- **Darknights Memoir**: "incredibly important", "basically chapter 6.5" ([jheve95](https://www.reddit.com/r/arknights/comments/12pbt1n/_/jheve95/), [jubkmwf](https://www.reddit.com/r/arknights/comments/15eczib/_/jubkmwf/)).
- **Babel**: "a must-read before chapter 14".
- **Vigilo**: covers the pre-amnesia Doctor.
- **Episode 14**: "14 is pretty important but 9-13 is mehhhh" ([p9bgi6u](https://www.reddit.com/r/arknights/comments/1we7dg8/_/p9bgi6u/)). Others push back: "ch14, and Babel are closely related to what's happening in ch15 and 16" ([p9bgs6u](https://www.reddit.com/r/arknights/comments/1we7dg8/_/p9bgs6u/)).
- **See You Soon**: "vital".
- **Lone Trail**: needed before Ep 15 (G1).
- **The prerequisite chains**:
  - Dorothy's Vision before Lone Trail.
  - Code of Brawl before Il Siracusano.
  - Maria Nearl before Near Light.
  - Under Tides before Stultifera Navis.
  - Heart of Surging Flame, because it introduces the AUS characters who reappear in Stultifera Navis ([jm5yjnp](https://www.reddit.com/r/arknights/comments/13utigi/_/jm5yjnp/)).

**Commonly called optional or skippable:**

- **Most side stories**: "the vast majority of AK side stories are self contained" ([lmljip4](https://www.reddit.com/r/arknights/comments/1fe91y9/_/lmljip4/)). Many answers say the rest "can be read at any time with no issue of main story spoilers" ([jubkmwf](https://www.reddit.com/r/arknights/comments/15eczib/_/jubkmwf/)).
- **Operational Intelligence**: all of it except "Anonymous One's War" and "Destroyer" (G1, [n83d1w9](https://www.reddit.com/r/arknights/comments/1mmxksx/_/n83d1w9/)).
- **Vignettes**: Beyond Here, Preluding Lights and To Be Continued. Only single sub-stories in each tie into arcs (G1).
- **A Kazdelian Rescue and Sarkaz Furnaceside Fables**: "Not at all necessary" (G1).
- **Path of Life**: "skippable but nice" (G1).
- **The Masses' Travels**: optional before Ep 16 (G1).
- **Mansfield Break**: "skippable since it's only tangentially related" ([ocurjwi](https://www.reddit.com/r/arknights/comments/1mmxksx/_/ocurjwi/)).
- **Tales Within the Sand** (G1).
- **Dossoles Holiday**: optional (G1). It also "spoils the main story", so release order protects you ([idyvkvz](https://www.reddit.com/r/arknights/comments/vm3ke4/_/idyvkvz/)).
- **Crossovers**: omitted as non-canon (G1, G5).
- **Integrated Strategies**: only the first ending is canon for IS1, IS3 and IS5 ([olddlqr](https://www.reddit.com/r/arknights/comments/1talyhb/_/olddlqr/)).
- **Contested**:
  - Grani: "you can skip Grani as its not really relevant and start with Under Tides" ([nf5nl7a](https://www.reddit.com/r/arknights/comments/1nhiqbp/_/nf5nl7a/)), against 8 commenters who say to read it before Under Tides.
  - Code of Brawl: "very optional" ([mpvzxni](https://www.reddit.com/r/arknights/comments/1k9t7z1/_/mpvzxni/)), against 5 who put it before Il Siracusano.
  - Episodes 0 to 3: called a slog by several ([m01e11u](https://www.reddit.com/r/arknights/comments/1h4u1nv/_/m01e11u/) suggests starting with Originium Dust instead).

**Outside the game:**

- The **Rhine Lab manga** is recommended before Lone Trail by 5 commenters.
- **Operator Records** are sometimes flagged: Ch'en before Ep 6, Siege before Ending a Grand Overture, Virtuosa before The Masses' Travels ([pa4vmos](https://www.reddit.com/r/arknights/comments/1wfneld/_/pa4vmos/)).
- **Near Light's three epilogues** cannot be read in-game ([ndeulu8](https://www.reddit.com/r/arknights/comments/16tudb3/_/ndeulu8/)).

## Official in-game related-story links (wiki Movements pages)

The wiki says each story's "related stories" are the ones recommended to be read first. Read X before Y:

- Darknights Memoir before Ep 7
- Ep 7 before A Walk in the Dust
- A Walk in the Dust before Ep 10
- Ep 9 before What the Firelight Casts
- What the Firelight Casts before Ep 12
- Babel before Ep 14
- Ep 14 before Ending a Grand Overture, See You Soon and A Kazdelian Rescue
- Ending a Grand Overture before Ep 15
- Ep 12 and Ending a Grand Overture before When Elegies Are Ashes
- Ep 1 before Children of Ursus
- Ep 16 and Ep 17 before People, A People
- Ep 4 before Operational Intelligence
- Ep 9 before Come Catastrophes or Wakes of Vultures
- Ep 14 and Lone Trail before Path of Life
- Ep 15 and Zwillingsturme before The Masses' Travels
- Hortus before Zwillingsturme
- Code of Brawl before Il Siracusano
- Lone Trail before Retracing Our Steps
- Adventure That Cannot Wait for the Sun before Medjehtiqedti Bound

These match the community consensus except for two points:

- The game has no Vigilo link, while the community treats Vigilo as a main-story prerequisite.
- The game has no "after Ep 14" gate on Lone Trail.

## Unreachable or partial sources

- **reddit.com**: blocked. Everything above comes through Arctic Shift.
- **Arctic Shift full-text comment search**: timed out on every query ("Timeout. Maybe slow down a bit"). So did several post queries: "chronological order", "which events", "read before", "story guide" and a few others returned HTTP 422. I worked around this by sampling the megathreads, which covers about 1 in 5 of the 724 threads.
- **The older community timeline Google Doc** (1htVLOcG..., linked in 2022 threads): now returns "page not found".
- **The G5 Google Sheet**: only the intro tab was readable through CSV export.
- **G3's gallery**: only its first image was viewed. The rest were read through the comments.
- **Not attempted**: the Google Drive timeline folders linked in [mc61418](https://www.reddit.com/r/arknights/comments/1im3kik/_/mc61418/) and [mvr1g2l](https://www.reddit.com/r/arknights/comments/1l1f39q/_/mvr1g2l/), and prts.wiki (CN).
- **Terra Log**: its timeline page lists only Movements, with no placement notes.
