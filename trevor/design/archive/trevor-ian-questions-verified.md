# Ian's 32 questions, verified against the wikis

Date: 2026-10-03, 23:00 JST. References come from `eval/ian-questions.jsonl`; every row now has a `verified` field ({status, sources, note}). Trevor's answers come from `artifacts/answers/ian-baseline-night9.jsonl` (default config, marker written at 22:59:57).

Sources: arknights.wiki.gg was read directly for every page cited. arknights.fandom.com returned HTTP 402 to fetches, so its pages were used only through search snippets. Every quote was also checked against the corpus (`artifacts/p4x/chunks.jsonl`).

## Counts

References: 22 of 32 are correct, 8 are partly correct, 1 is wrong (ian11) and 1 is unverifiable (ian14).

Trevor's current answers: 12 right, 12 partial, 1 wrong (ian19) and 7 declined. Two of the 7 declines, ian14 and ian18, are acceptable, because the answer is not in game text.

## Table

| qid | question (short) | reference | Trevor now | what Trevor got wrong | fix |
|---|---|---|---|---|---|
| ian1 | best cook | correct (the Gummy quote is actually in profile_gummy#0000) | partial | It names Matterhorn, Lee, Senshi and Warmy, mixed with NPC boasts (Young Chef, Mo Bufu). It misses Hung, Shu, Yu and Gummy, and it drops Matterhorn's SilverAsh detail. | retrieval (breadth on superlatives) |
| ian2 | Matsukiri marriage fraud | correct | right | none | none |
| ian3 | Midnight vs Matsukiri | correct | right | It does not say that Midnight's host career is the closer fit, which is a small miss. | none |
| ian4 | Andoain married | correct | right | none | none |
| ian5 | Harold married twice, children | correct | partial | It gets a wife and daughter, but says his daughter "was the same age as a burdenbeast named Lily", which garbles two passages. It retrieved only 2 chunks, while the reference cites 6. | retrieval + answer step |
| ian6 | operator who wants to pee in public | correct | declined | The line says "piss" (Chiave, act15d5 st02#0000), so the query word "pee" never matched it. | retrieval (vocabulary expansion) |
| ian7 | Flamebringer romance | correct | right | none | none |
| ian8 | Insider favourite food | partly correct (the file says "sweets", with donuts as the example) | right | none: it answered "sweets, especially donuts", which is better than the reference | none |
| ian9 | Ulpianus talkative | correct (the Amaia line says LESS tight-lipped) | right | none | none |
| ian10 | Iris origin + all Victorian NPCs | partly correct (the NPC half is unscoped) | partial | Victoria is right. The NPC half is a few fragments ("Godmother", "old noble boy"). | missing data (NPC-by-nation table) and a scoped rubric |
| ian11 | operator based on an animal with "phantom" in its name | WRONG | declined | The answer is Lucilla, based on the giant phantom jelly per wiki trivia, and that basis is not in the game text. | not in game text (needs a design-basis table) |
| ian12 | Mon3tr suggests assassinating a child | correct | declined | The line is in the corpus (act42side_08_beg#0000) and was not retrieved. | retrieval |
| ian13 | Angelina's age | correct | right | none (labelled estimate of 15 to 18) | none |
| ian14 | height written in E2 art | unverifiable (Ian says Pallas; no text source confirms it) | declined, acceptable | It said it could not find the answer. The rubric wants "cannot see illustrations", which it did not say. | not in game text (art captions) |
| ian15 | why Swire joined the LGD | correct | right | none: it starts from the kidnapping and adds the grandfather thread | none |
| ian16 | bosses who join our side | partly correct (needs a list, not a method) | partial | It lists 12. 'Tragodia' is probably a false positive, since the boss is the NPC troupe leader. It misses Mon2tr = Mon3tr, Theresa = Civilight Eterna, 'Misery', Mechanist and Highmore. Varkáris = Xanthos is a good catch. | missing data (identity links) |
| ian17 | Rhodes Island founding | correct | partial | Its labelled estimate is 1086 to 1090, from the landship excavation and "Rhodes Island of Babel". It omits the post-1094 reorganisation into Rhodes Island Pharmaceutical, which the wiki timeline treats as the founding. | answer step (chronology) |
| ian18 | who is Ed | partly correct ("architect" is too strong; summaries say "director") | declined, acceptable | It found only Herr Ed and did not say that the ARG is outside its data. | not in game text (ARG) |
| ian19 | Kristen and Friston | correct (he says "You're not my sunshine", he does not call her that) | WRONG | Its first sentence says Friston calls Kristen his "daughter" and "my sunshine". It never states that they are unrelated, and it adds an unsupported "part of Kristen's family". | answer step (people kept apart) |
| ian20 | Abyssal Hunter project start | partly correct (Ian's "halted, then revived" sequence is not supported) | right | It says two centuries of debate, then made real twenty-some years ago by Ulpianus. That matches the wiki. | none |
| ian21 | Aroma's grandmother's job | correct | right | none | none |
| ian22 | who helped Mutsumi with the plants | correct | partial | It opens with unnamed operators "without saying a word" before naming the 4 robots, and it adds Perfumer and Beanstalk. | answer step (lead with the module's answer) |
| ian23 | Lava's elite idol | correct (the teacher can be named: Mr. Johann) | right | none | none |
| ian24 | RI operator who visited Iris's castle | correct (Bluishsilver = Mabel Grimm) | declined | archive_char_338_iris#0001 and act15d5 st03 were not retrieved. | retrieval (codename to real-name link) |
| ian25 | brownie recipe | correct | partial | It did not decline as off-topic. It searched and offered apple pie and cookie passages instead. | routing (off-topic gate) |
| ian26 | Deepcolor in the Church of the Deep | correct (note: her nationId is egir) | partial | It says "passages do not mention" instead of concluding no: nothing links her to the Church. | answer step (absence verdict) |
| ian27 | IS4 Ending 2 + endbooks | correct | partial | Endbook 2 is told as "destroyed by a black spear", leaving out the self-freezing that seals the core. Endbook 3 omits that Santalla takes the vacant Snowpriest seat. The broken horn is moved from the Snowpriest to a warrior. | answer step |
| ian28 | who are the Sarkaz | partly correct (the reference omits original inhabitants, many races, the Originium curse) | partial | It gives a thin topic card. It omits Infection susceptibility, the kings, the civil war, Theresis and Londinium, and the end of the war in Episode 14. | retrieval + routing (topic depth) |
| ian29 | what Sami guards | partly correct (should name the Collapsals, the Myrkwood, the Aethergate) | partial | It covers the northern front and the walls, but not what is guarded against, and it adds "keep out ... other Sami". | retrieval (topic depth) |
| ian30 | three Dossoles factions | correct (strictly, Bolívar is split; Dossoles is neutral ground) | declined | Dossoles was not routed to Bolívar and act12side. | routing (place to nation) |
| ian31 | the Khaganquest | partly correct (the reference wrongly calls the Nightzmora a "demonic army" and the Kuranta "horned"; Kharanduu is one historical Khagan) | partial | The rite is right, but it is given to "the Kuranta, a people described as knights", and no terms are defined. | answer step + missing data (glossary) |
| ian32 | Elysium's chapters | correct (add his own paradox simulation) | right | It names the paradox simulations as not counted, which is acceptable. | none |

## References that need rewriting

1. ian11 (corrected 2026-10-04 after Ian's review: both readings hold, see eval/ian-questions.jsonl; the 'Canada lynx' claim below was not on the Phantom trivia page and is withdrawn; Phantom's trivia ties his motif to Poe's The Black Cat). Original note: ian11 is wrong. wiki.gg gives Canada lynx as the basis for both Phantom and the operator Tragodia, and that animal has no "phantom" in its name. Lucilla is based on Stygiomedusa gigantea, the "giant phantom jelly" (https://arknights.wiki.gg/wiki/Lucilla/Trivia). This disagrees with Ian: Lucilla should be the answer, with Phantom accepted only as wordplay.
2. ian31 has 3 wrong term definitions. The Nightzmora are an Elder subgroup of the Kuranta known for illusory Arts. The Kuranta are horse-like, not horned. Kharanduu Khagan is one historical Khagan (https://arknights.wiki.gg/wiki/Nightzmora).
3. ian20 contradicts Ian's "halted, then revived by Ulpianus" sequence. The wiki says the project was green-lit in the 1080s with Ulpianus as its first subject and shut down after the 1095 Ishar-mla disaster. In act34side the Institute asks Blandus to restart it and Ulpianus refuses (https://arknights.wiki.gg/wiki/Abyssal_Hunters).
4. ian18 overstates Ian's claims in 2 ways. Ed is addressed in Oracle's diary in ARG #3, and summaries call him director of the Celestial Fulcrum project, not its architect. Ed is not in the game data (https://arknights.wiki.gg/wiki/Arknights_ARG/3).
5. ian14 rests only on Ian's word. No text source mentions a height in Pallas's E2 art, so someone needs to look at https://arknights.wiki.gg/wiki/Pallas/Gallery.
6. ian16 needs an explicit list (see the ian16 row). ian10 needs a scoped NPC rubric. ian28 and ian29 need the missing depth items. ian19, ian30 and ian8 need the wording fixes given in their notes.

Ian's other answers check out: Swire's kidnapping (ian15), Bluishsilver = Mabel Grimm (ian24), the Dossoles factions (ian30) and Elysium's chapters (ian32). The examples Ian also gave (IS3 canon = Precious Days; Margaret = Nearl the Radiant Knight vs Blemishine; the Nearls are Kuranta) are not among these 32 questions, so they were not checked here.

## Premises

No question has a false premise outright. ian2 and ian3 presuppose acts that never happen, and both references already say so. ian30 locates the factions in Dossoles when they belong to Bolívar, but the wiki calls Dossoles Bolívar's de jure capital, so the premise holds loosely. ian11's premise is true, but only for Lucilla, not for Ian's answer.

## Recurring failure causes in Trevor's answers

1. Retrieval misses a fact that sits in a single line: ian6, ian12 and ian24 decline although the line is in the corpus, and ian1 and ian5 are thin. That is 5 questions, and the cause is a vocabulary mismatch (pee vs piss) or a codename vs real-name mismatch.
2. The answer step hedges or garbles: ian19, ian22, ian26, ian27 and ian31, plus part of ian5. That is 5 to 6 questions. The model buries the verdict, merges two people or two passages, or skips the key outcome.
3. Routing: ian25 has no off-topic gate, ian30 has no place-to-nation link, and ian28 and ian29 get thin topic cards. That is 4 questions.
4. Missing data (not fixable by retrieval): ian10 (NPCs by nation), ian11 (animal basis), ian14 (art) and ian16 (identity links). That is 4 questions.
