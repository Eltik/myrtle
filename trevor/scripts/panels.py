#!/usr/bin/env python3
"""Per-story reader panels for myrtle.moe (stdlib only, no model): recap, who's here, terms, suggested questions.

Moved from the prototype ~/overnight-2026-10-15/prototypes/trevor-panels/build_panels.py (2026-10-16). The panel
content is the prototype's byte for byte, with one deliberate change the binding contract asks for
(design/trevor-contract-v1.md section 1): `era.position` is a gatePos (the 1-based rank of the era group's LAST story
in the release order), where the prototype wrote the reading-guide group position. `--era-guide-position` restores
the prototype's value (kill switch, measurement only).

Output (default artifacts/panels/):
  stories/<storyId>.json   one panel per story, compact UTF-8 JSON, schema 1 (the value of PublishVersion.panels[id])
  manifest.json            {schema, version, server, corpusVersion, builtFrom, builtAt, storyOrder, eras, panels,
                            panelsSha, gateOrderSha, answersInline, stats}: publish.py's version manifest; version
                            defaults to <corpusVersion>-p<first 8 hex of panelsSha>
  stats.json               coverage and size numbers
The directory is written as <out>.tmp and renamed over <out>, so a reader never sees half a build.

THE SHARED GATE ORDER: artifacts/chrono/gate_order.json (schema 1). One spoiler scale for every component
(src/search/horizon/, the panels, trevor_panels.gate_pos, trevor_answers.gate_pos). The schema is horizon/mod.rs's
(`GateOrderFile`, src/search/horizon/mod.rs, also written by `ask --gate-order`); this script writes the same structure
in the same format (serde_json pretty: 2-space indent, sourcePos keys sorted, no trailing newline), so either
writer produces the same file from the same inputs:
  {
    "schema": 1,
    "inputs": {"readingGuide": sha16, "spoiler": sha16, "sourceDates": sha16 | null},  first 16 hex of each file's sha256
    "storyOrder": [storyId, ...],    every story of spoiler.jsonl; gatePos = index + 1
    "never": [storyId, ...],         stories with no guide group and no date: hidden at every horizon (0 today)
    "dated": [storyId, ...],         stories outside the guide placed by their source date (operator records)
    "sourcePos": {sourceId: int},    dated sources that are not stories (operator files, voice lines, modules,
                                     outfits, game text): visible iff sourcePos <= horizonPos; sourcePos = 1 + the
                                     number of stories whose key is below the source's
    "eras": [{"id", "name", "position", "groupPosition"}]   position = gatePos of the era group's last story;
                                     groupPosition = the reading-guide position (the prototype's era `position`)
  }
  Rule (the prototype's, which is horizon/mod.rs's old rule plus a strict tie-break): a story in a reading-guide group
  sorts by (guide position, storySort, line in spoiler.jsonl); the line breaks the storySort ties the old story-id
  horizon treated as mutually visible. An unplaced story or source dated by source_dates.json (an upper bound on its
  EN release) sits just BEFORE the stories of the first guide group whose latest guide date admits that date, ordered
  by date, set name, storySort and line. Anything undated sorts last and is never visible.
  --gate-order auto (default): a file whose recorded inputs match the current ones is READ and its storyOrder, never,
  dated, sourcePos and era positions are checked against this script's own ranking (a mismatch exits 3 and builds
  nothing); a missing or stale file is written. --gate-order write always rewrites it; --gate-order off neither reads nor writes it (the
  prototype).

Spoiler gate: every fact is placed on the release order above and is used only when it sits at or before the story.
After the build every written panel is checked again from its own content (previous, soFar, firstSeen, firstNamed
strictly earlier; suggestions, identities, term definitions, primer and era at or before); any violation exits 1.
--gate-sources (default on) also gates a bank entry by its `sources` (operator files a `who` answer read), not only by
its `gateStories`; --no-gate-sources is the prototype's rule.

EXTRA IDENTITY AND DEFINITION SOURCES (2026-10-17, --extra-sources; `none` is the kill switch and reproduces the
panels before them byte for byte). They only fill an identity or a definition the sources above leave empty (the
operator file's first Profile sentence, then the bank `who` answer); the listing, its order and every other field
are unchanged. Identity, first match in this order, each its earliest occurrence:
  attrs     the operator file's Basic Info (entities/operator_attributes.jsonl), gated by the file's own sourcePos
  enemy     a boss enemy handbook description (artifacts/chrono/enemy_intros.json, scripts/enemy_intros.py), visible
            from the _end story of the first story stage that fields the boss (a _beg-only stage: the next gatePos)
  synopsis  "Name, a/an/the <phrase>," in the official synopsis of a STRICTLY earlier story
  intro     a speaker's self-introduction ("Elena: “I'm Elena, a researcher in the Energy Section.”") or another
            speaker's ("This is Senshi, the newest member of the party"), quoted, from a strictly earlier story
  mention   a mid-sentence appositive in a strictly earlier story ("...told Rankin, the foreman, ..."), not after a
            title or a clause opener, not a list item, not a clause; only for labels that are not terms or roles
  Their identitySource is null (gates.py's schema allows operatorFile|bank|null; attrs writes operatorFile);
  --identity-source-labels writes enemyHandbook/storySynopsis/storyLine instead.
Definitions:
  topic     later sentences of the topic summary (the prototype reads the first 3), first those whose subject is
            the term, then any other that names the term and does not lean on the sentence before it; each still
            gated by every passage it cites
  --ground  (default all when any extra source is on) the summaries were written from all of a topic's passages,
            so a sentence can cite an early passage and state a later one's fact ("The current Speaker of
            Parliament is Vina Victoria [4]", [4] an early outfit text). Every capitalized name and quoted phrase of
            a sentence must occur in its cited passages; one that does not is looked up in the topic's other
            passages and the sentence waits for the earliest that has it; a name in no passage drops it. `all`
            also applies this to the prototype's first 3 sentences (2026-10-17: 13 of them wait for a later
            passage, 11 are dropped, 27 are evidenced at no later gate); `new` leaves those as they were.
provenance.json (not published) records each extra line's kind and source; the post-build check (check_new_lines)
re-derives every one from the gate order's positions alone: a story source must be strictly before the panel, an
enemy stage and an operator file at or before, every cited passage at or before, an undated one never.
--inject-late N is its canary (2N lines from the next story; the build must report 2N violations and exit 1).

Usage: python3 scripts/panels.py [--out artifacts/panels] [--version V] [--built-from RES_VERSION]
                                 [--gate-order auto|read|write|off] [--no-answers] [--self-test]
                                 [--extra-sources LIST|none] [--ground auto|all|new|off] [--inject-late N]
"""
import argparse, bisect, collections, gzip, hashlib, json, os, re, shutil, statistics, sys, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py sits beside the scripts
import common  # noqa: E402
from common import sha_file  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCHEMA = 1
GATE_ORDER_SCHEMA = 1
NEVER = (10**9,)  # sorts after every real key: hidden at every horizon
GATE_ORDER_REL = os.path.join("artifacts", "chrono", "gate_order.json")
ORDER_INPUTS = ("artifacts/chrono/reading_guide.json", "artifacts/spoiler.jsonl", "artifacts/chrono/source_dates.json")

# ---------------------------------------------------------------- helpers
def jl(path):
    with open(path, encoding="utf-8") as f:
        for line in f:
            if line.strip():
                yield json.loads(line)


def parse_day(s):
    m = re.search(r"(\d{4})-(\d{2})-(\d{2})", s or "")
    if not m:
        return None
    y, mo, d = (int(x) for x in m.groups())
    # days from civil (same as src/search/horizon/)
    y2 = y - 1 if mo <= 2 else y
    era = y2 // 400
    yoe = y2 - era * 400
    doy = (153 * (mo - 3 if mo > 2 else mo + 9) + 2) // 5 + d - 1
    doe = yoe * 365 + yoe // 4 - yoe // 100 + doy
    return (era * 146097 + doe - 719468) * 86400


CITE = re.compile(r"\s*\[(\d+(?:\s*,\s*\d+)*)\]")
SENT = re.compile(r"(?<=[.!?])\s+(?=[A-Z\"'(])")


def strip_cites(s):
    return CITE.sub("", s).strip()


def tidy(s):
    """Game-script placeholders: the player's nickname is the Doctor."""
    return s.replace("Dr. {@nickname}", "the Doctor").replace("{@nickname}", "the Doctor") if s else s


def first_sentence(s, maxlen=260):
    s = (s or "").strip().split("\n")[0]
    parts = SENT.split(s)
    out = parts[0] if parts else s
    if len(out) > maxlen:
        out = out[: maxlen - 1].rsplit(" ", 1)[0] + "…"
    return out


TOKEN = re.compile(r"[A-Za-z0-9À-ÿ][A-Za-z0-9À-ÿ'’\-]*")


def tokens(text):
    out = []
    for m in TOKEN.finditer(text):
        t = m.group(0)
        if t.endswith(("'s", "’s")):
            t = t[:-2]
        t = t.rstrip("'’-")
        if t:
            out.append(t)
    return out


GENERIC = re.compile(
    r"^(\?+|…+|\.\.\.)$|(^|\s)[A-Z0-9]$|\b(Member|Soldier|Guard|Citizen|Resident|Mercenary|Officer|Worker|Staff|"
    r"Passerby|Passer-by|Villager|Thug|Customer|Child|Kid|Man|Woman|Voice|Crowd|Everyone|All|Students?|Merchant|"
    r"Civilian|Infected|Clerk|Attendant|Driver|Bystander|Spectator|Audience|Messenger|Scout|Sentry|Patrol|"
    r"Subordinate|Henchman|Rioter|Refugee|Elder|Doctor|Nurse|Researcher|Engineer|Pirate|Bandit|Hunter|Knight|"
    r"Sarkaz|Mercenaries|Narration|Narrator|System|Announcement|Broadcast|Radio|Unknown)s?\b",
    re.I,
)


def dumps_compact(x):
    return json.dumps(x, ensure_ascii=False, separators=(",", ":"))


# ---------------------------------------------------------------- extra identity and term sources (2026-10-17)
# Each fills only an identity or a definition the prototype's sources leave empty; the listing itself is unchanged.
EXTRA_SOURCES = ("attrs", "enemy", "synopsis", "intro", "mention", "topic")
# identitySource for the new kinds under --identity-source-labels (gates.py's schema accepts only operatorFile|bank|null
# today, so by default they are written as null and the kind lives in provenance.json)
EXTRA_LABEL = {"enemy": "enemyHandbook", "synopsis": "storySynopsis", "intro": "storyLine", "mention": "storyLine"}
# A story line where a speaker names themself and says what they are: "I'm Elena, a researcher in the Energy
# Section." The name must be the line's speaker label. Starts that carry no identity are dropped.
INTRO_SELF = re.compile(r"(?:^|(?<=[.!?…] ))(?:I'm|I am|My name is|My name's) ([A-Z][\w'’.\- ]{0,30}?)(, | of )"
                        r"([^.;!?…]{3,90}?)[.!?…]")
# Another speaker introduces them: "That's Mostima, the Messenger..."
INTRO_OTHER = re.compile(r"(?:^|(?<=[.!?…] ))(?:This is|this is|That's|That is|Meet) ([A-Z][\w'’.\- ]{0,30}?), "
                         r"((?:a|an|the|our|my) [^.;!?…,]{3,80}?)[.!?…,]")
INTRO_BAD_START = re.compile(r"^(?:and (?!I'm |I am )|but |not |so |of course|this |that |which |if |right|yes|no\b|"
                             r"okay|ok\b|sorry|please|just |too\b|again|here|now\b|though|you\b|your |remember)", re.I)
# An official synopsis names someone with an appositive: "..., Cecelia, a resident of the outskirts, ..."
SYN_APPOS = re.compile(r"(?:^|(?<=[.!?;] )|(?<=, )|(?<=[a-z] ))([A-Z][\w'’\-]*(?: [A-Z][\w'’\-]*){0,2}), "
                       r"((?:a|an|the) [^,.;:!?]{3,90}?)(?:,(?! (?:and|or|nor) )| who\b)")


# Topic summary sentences that talk about the text instead of the term, or say what is not known.
TOPIC_META = re.compile(r"\b(the text|the passages?|the sources?|does not (?:specify|say|mention|explain|describe)|"
                        r"not (?:specified|mentioned|explained|described|stated)|unclear|unknown|remains? a mystery|"
                        r"questions remain)\b", re.I)
# Later summary sentences that lean on the one before them ("While...", "Her leadership...") make poor definitions.
TOPIC_WEAK_START = re.compile(r"^(?:While|However|Despite|Finally|Additionally|Furthermore|Although|In addition|"
                              r"Ultimately|Notably|Moreover|Beyond|Overall|Meanwhile|Conversely|Similarly|Likewise|"
                              r"Instead|Also|Thus|Therefore|Consequently|For example|For instance|This|These|That|"
                              r"Those|Such|He|She|His|Her|Its|Their|Other|Some|Many|Several|Both|Each|Under|After|"
                              r"Before|During|Since|Following|As|In contrast|By contrast|On the other hand|To solve|"
                              r"To address|In some|In other|In this|In turn|In one|At the same time)\b")
KIND_HEADS = {
    "organization": {"organization", "group", "company", "corporation", "family", "club", "agency", "association",
                     "union", "order", "society", "guild", "faction", "movement", "institution", "team", "band"},
    "place": {"city", "region", "area", "town", "village", "district", "borough", "island", "territory", "mountain",
              "mountains", "settlement", "location", "site", "land", "country", "province"},
    "nation": {"nation", "country", "state", "empire", "kingdom", "federation", "republic"},
    "race": {"race", "species", "people"},
    "event": {"event", "incident", "war", "conflict", "battle", "catastrophe"},
    "concept": set(),
}


GROUND_SKIP = {"The", "It", "They", "In", "While", "However", "This", "These", "Its", "Their", "A", "An", "As", "At",
               "By", "For", "From", "On", "Of", "To", "With", "Despite", "Although", "Additionally", "Furthermore",
               "Notably", "Under", "After", "Before", "During", "Since", "Following", "Some", "Many", "Several",
               "Both", "Each", "Other", "Such", "When", "Where", "Which", "Who", "He", "She", "His", "Her", "Unlike",
               "Ultimately", "Finally", "Moreover", "Overall", "Meanwhile", "Currently", "Historically", "Today",
               "Later", "Once", "Though", "Beyond", "Within", "Through", "Among", "Also", "Because", "If", "One",
               "Two", "Three", "That", "Those", "There", "Here", "According"}


def _norm(x):
    return x.lower().replace("’", "'")


def has_word(w, T):
    """w occurs in the lowercased text T, allowing a suffix ("Victorian" in "victoria", "Felines" in "feline")."""
    w = _norm(w)
    return w in T or any(w.endswith(x) and len(w) - len(x) >= 4 and w[:-len(x)] in T
                         for x in ("ians", "ian", "ans", "an", "es", "s", "n"))


def has_item(item, T):
    """A capitalized word, or a quoted phrase ('"..."'), occurs in the lowercased text T."""
    if item.startswith('"'):
        q = _norm(item.strip('"'))
        return q in T or all(has_word(x, T) for x in q.split())
    return has_word(item, T)


def ungrounded_items(sent, text, names):
    """The capitalized words (after the first) and quoted phrases of a summary sentence that do not occur in text
    (its cited passages). The term's own name and aliases are exempt."""
    T = _norm(text)
    own = {w.lower() for n in names for w in tokens(n)}
    out = []
    for m in re.finditer(r"[A-Z][\w'’\-]+", sent):
        w = re.sub(r"(?:'s|’s)$", "", m.group(0).rstrip("'’"))
        if m.start() and w not in GROUND_SKIP and w.lower() not in own and not has_word(w, T):
            out.append(w)
    for m in re.finditer(r'"([^"]{3,80})"', sent):
        q = '"' + re.sub(r"[\s,.;:!?]+$", "", m.group(1)) + '"'
        if not has_item(q, T):
            out.append(q)
    return out


def grounded(sent, passages, names):
    """True when every name and quoted phrase of the sentence occurs in its cited passages (no text: False)."""
    text = " ".join(passages)
    return bool(text.strip()) and not ungrounded_items(sent, text, names)


def story_intros(text, speakers_seen):
    """(name, kind, line) for every introduction in one chunk's text, in text order. kind is "self" or "other"."""
    out = []
    for line in text.split("\n"):
        m = re.match(r"^([^:]{1,40}): (.*)$", line)
        if not m:
            continue
        spk, body = m.group(1), m.group(2)
        for mm in INTRO_SELF.finditer(body):
            if mm.group(1) == spk and not INTRO_BAD_START.match(mm.group(3)) and len(mm.group(3).split()) <= 14:
                out.append((spk, "self", f"{spk}: “{body[mm.start():mm.end()]}”"))
        for mm in INTRO_OTHER.finditer(body):
            n = mm.group(1)
            if n != spk and n in speakers_seen and len(mm.group(2).split()) <= 10:
                out.append((n, "other", f"{spk}: “{body[mm.start():mm.end()].rstrip(',')}…”"
                            if body[mm.end() - 1] == "," else f"{spk}: “{body[mm.start():mm.end()]}”"))
    return out


# A story line (dialogue of someone else, or narration) that names them with an appositive in mid-sentence:
# "...she told Rankin, the foreman, ..." A lowercase word must come before the name (a vocative follows a comma or
# starts the line), no form of address may precede it, and the phrase may hold no pronoun or auxiliary (a clause,
# "Miss Alty, the town has become quite dangerous", is not an appositive). The phrase must close with a comma that
# no "and"/"or" follows ("Diane, the departing Glasgow Gang, and the crown" is a list).
MENTION_APPOS = re.compile(r"(?<=[a-z] )([A-Z][\w'’\-]*(?: [A-Z][\w'’\-]*){0,2}), "
                           r"((?:a|an|the) [^,.;:!?…]{3,70}?),(?! (?:and|or|nor) )")
TITLE_WORDS = {"Miss", "Mister", "Mr.", "Mrs.", "Ms.", "Dame", "Sir", "Herr", "Frau", "Madam", "Madame", "Lord", "Lady",
               "Master", "Uncle", "Aunt", "Captain", "Doctor", "Dr.", "Signora", "Signore", "Signor", "Mistress", "Old",
               "Little", "Big", "Your", "Mayor", "Patriarch", "Professor", "General", "Commander", "Chief", "Elder"}
# "With Caesar, the Signori dei Lupi surround..." or "After meeting Mostima, the two blend into the crowd": the
# sentence opened with an introductory phrase that ends at the name's comma, so a clause follows, not an appositive.
MENTION_OPENER = re.compile(r"(?:^|[.!?…] )(?:With|After|Before|For|Like|Unlike|As|Since|Without|Besides|Despite|"
                            r"Upon|When|While|Behind|Beside|Against|Including|Except|Once|Until|Following|Thanks to|"
                            r"Together with|Along with|Because of|Due to|During|In|On|At|Through|Under|Among|Alongside|"
                            r"[A-Z][a-z]*(?:ed|ing))"
                            r"\b[^.!?…,;:]*$")
MENTION_TITLE = re.compile(r"\b(?:Miss|Mister|Mr\.|Mrs\.|Ms\.|Dame|Sir|Herr|Frau|Madam|Madame|Lord|Lady|Your|"
                           r"Master|Uncle|Aunt|Captain|Doctor|Dr\.|Signora|Signore|Signor|Mistress|Old|Little|Big|"
                           r"dear|my)\s+$")
MENTION_CLAUSE = re.compile(r"\b(?:I|you|he|she|we|they|it|me|him|her|us|them|his|their|our|your|its|is|are|was|"
                            r"were|be|been|has|have|had|will|would|should|can|could|must|may|might|shall|do|does|did|"
                            r"isn't|aren't|wasn't|weren't|don't|doesn't|didn't|won't|can't|goes|gave|came)\b")


# A head noun that says nothing about who they are, or a plural head (the phrase is a list item, "Theresa, the
# invincible forces of nature, ...").
ROLE_WORDS = {"Operator", "Operators", "Doctor", "King", "Queen", "Tutor", "Mechanist", "Chancellor", "Emperor",
              "Empress", "Gendarmerie", "Grace", "Spokesman", "Spokesperson", "Announcer", "Narrator"}
APPOS_VAGUE_HEAD = {"man", "woman", "girl", "boy", "guy", "one", "kid", "child", "lady", "gentleman", "person", "sort",
                    "type", "thing", "fellow", "lass", "lad"}
APPOS_PREP = re.compile(r"\s(?:of|from|in|at|with|who|whom|that|for|on|to|under|behind)\s")


def appositive_ok(phrase):
    """A noun phrase that can stand as an identity line: not a clause, not vague, not plural."""
    words = phrase.split()
    if len(words) > 8 or MENTION_CLAUSE.search(phrase.split(" ", 1)[1]) or words[1].lower() == "one":
        return False
    head = APPOS_PREP.split(" " + phrase + " ", 1)[0].split()[-1].lower()
    if head in APPOS_VAGUE_HEAD and len(words) <= 3:
        return False
    return not (len(head) > 3 and head.endswith("s") and not head.endswith(("ss", "us", "is")) and head[:1].islower())


def story_mentions(text):
    """(name, line) for each mid-sentence appositive of one chunk's text (speaker != name)."""
    out = []
    for line in text.split("\n"):
        m = re.match(r"^([^:]{1,40}): (.*)$", line)
        spk, body = (m.group(1), m.group(2)) if m else (None, line)
        for mm in MENTION_APPOS.finditer(body):
            words, phrase = mm.group(1).split(" "), mm.group(2)
            if not appositive_ok(phrase) or MENTION_TITLE.search(body[:mm.start()]) or \
                    MENTION_OPENER.search(body[:mm.start()]) or any(w in TITLE_WORDS for w in words[:-1]):
                continue
            out.append((words, phrase, spk))
    return out


def synopsis_appositives(syn, names):
    """(name, line) for each "Name, a/an/the <phrase>," of an official synopsis whose name is a listed speaker."""
    out = []
    for m in SYN_APPOS.finditer(syn or ""):
        words, phrase = m.group(1).split(" "), m.group(2)
        if not appositive_ok(phrase) or MENTION_OPENER.search(syn[:m.start()]) or \
                any(w in TITLE_WORDS for w in words[:-1]):
            continue
        for i in range(len(words)):  # "Rhodes Island operator Grani": the longest suffix that is a speaker
            n = " ".join(words[i:])
            if n in names:
                out.append((n, f"{n}, {phrase}."))
                break
    return out


def lead_sentences(s, min_len=60, maxlen=260):
    """Leading sentences of s until they reach min_len characters ("Age unknown, gender unknown, history unknown."
    alone says little), cut at maxlen like first_sentence."""
    parts = SENT.split((s or "").strip().split("\n")[0])
    out = ""
    for x in parts:
        if out and len(out) + 1 + len(x) > maxlen:
            break
        out = f"{out} {x}" if out else x
        if len(out) >= min_len:
            break
    return first_sentence(out, maxlen) if len(out) > maxlen else out


BASIC_INFO_OK = re.compile(r"^[A-Za-zÀ-ÿ0-9 '’\-.,()&/]+$")


def basic_info_line(at):
    """The operator file's Basic Info as one line (operator_attributes.jsonl row). A field with stray symbols is
    left out (Ifrit's file is glitched on purpose: "race: Sa■&K?uSxw?...")."""
    at = {k: (v if isinstance(v, str) and BASIC_INFO_OK.match(v) else None) for k, v in at.items()}
    head = f"{at['class']} Operator ({at['branch']})" if at.get("class") and at.get("branch") else at.get("class")
    if not head:
        return None
    parts = [head]
    aff = [x for x in (at.get("nation"), at.get("group"), at.get("team")) if x]
    if aff:
        parts.append("affiliation: " + ", ".join(dict.fromkeys(aff)))
    if at.get("race") and at["race"] != "Unknown":
        parts.append("race: " + at["race"])
    if at.get("birthplace") and at["birthplace"] != "Unknown":
        parts.append("birthplace: " + at["birthplace"])
    return "; ".join(parts) + "."


def check_new_lines(p, prov, gp, sp):
    """The extra sources' gate, checked from the written panel, its provenance record and the gate order's positions
    alone (storyId -> gatePos, sourceId -> sourcePos), not from the code that chose the lines. Every identity or
    definition that has a provenance record must match it and its source must sit at or before the panel's gatePos
    (story lines and synopses strictly before). Returns violation strings."""
    sid, here = p["storyId"], p["position"]
    out = []

    def pos_of(x, chunk=False):
        s = x.rsplit("#", 1)[0] if chunk else x  # a chunk id is "<source>#NNNN"; source ids may hold '#' too
        return gp.get(s, sp.get(s))

    ids = (prov or {}).get("identity", {})
    defs = (prov or {}).get("definition", {})
    for c in p["whosHere"]["characters"]:
        r = ids.get(c["name"])
        if r is None:
            if c["identity"] is not None and c["identitySource"] not in ("operatorFile", "bank"):
                out.append(f"{sid}: identity of {c['name']} has no source")
            continue
        if c["identity"] != r["text"]:
            out.append(f"{sid}: identity of {c['name']} differs from its provenance")
        q = pos_of(r["source"])
        if r["kind"] in ("synopsis", "intro", "mention"):
            ok = q is not None and q < here
        elif r["kind"] == "enemy":
            ok = q is not None and q + (1 if r.get("after") else 0) <= here
        elif r["kind"] == "attrs":
            ok = q is not None and q <= here
        else:
            ok = False
        if not ok:
            out.append(f"{sid}: identity of {c['name']} from {r['kind']} {r['source']} at {q} is past {here}")
    for t in p["terms"]["items"]:
        r = defs.get(t["term"])
        if r is None:
            continue
        if t["definition"] != r["text"]:
            out.append(f"{sid}: definition of {t['term']} differs from its provenance")
        qs = [pos_of(x, chunk=True) for x in r["passages"]]
        if r["kind"] != "topic" or not qs or any(q is None or q > here for q in qs):
            out.append(f"{sid}: definition of {t['term']} cites {r['passages']} at {qs}, past {here}")
    return out


# ---------------------------------------------------------------- release order
class Order:
    """Total release order over stories and dated sources, equivalent to horizon/mod.rs `allows` (plus the tie-break)."""

    def __init__(self, art):
        guide = json.load(open(os.path.join(art, "chrono/reading_guide.json"), encoding="utf-8"))["items"]
        self.guide = {i["groupId"]: i for i in guide}
        self.gpos = {i["groupId"]: i["position"] for i in guide}
        dated = sorted((i["position"], parse_day(i.get("released"))) for i in guide)
        self.date_at, latest = {}, None
        for p, d in dated:
            latest = d if latest is None else (latest if d is None else max(latest, d))
            if latest is not None:
                self.date_at[p] = latest
        self.pos_sorted = sorted(self.date_at)
        self.dates_sorted = [self.date_at[p] for p in self.pos_sorted]  # non-decreasing
        self.source_date = json.load(open(os.path.join(art, "chrono/source_dates.json"), encoding="utf-8"))["dates"]
        self.story = {}  # storyId -> row
        for i, r in enumerate(jl(os.path.join(art, "spoiler.jsonl"))):
            r["_line"] = i  # tiebreak: spoiler.jsonl has storySort ties (act33side, main_6, records)
            self.story[r["storyId"]] = r
        self._key = {}
        for sid in self.story:
            self._key[sid] = self.key(sid)
        ranked = sorted(self.story, key=lambda s: self._key[s])
        self.rank = {s: i + 1 for i, s in enumerate(ranked)}
        self.ranked = ranked
        self.ranked_keys = [self._key[s] for s in ranked]
        self.group_stories = collections.defaultdict(list)
        for s in ranked:
            self.group_stories[self.story[s]["groupId"]].append(s)

    def _dated_key(self, d, sid, sort=0, line=0):
        # first guide position whose (latest) date admits d; the source sits just before that group's stories
        i = bisect.bisect_left(self.dates_sorted, d)
        p = self.pos_sorted[i] if i < len(self.pos_sorted) else 10**6
        return (p, 0, d, sid.rsplit("_story_", 1)[0], sort, line)

    def key(self, sid):
        """Release key of a story id, group id or source id (chunk ids `x#0003` accepted); NEVER if undated."""
        if sid in self._key:
            return self._key[sid]
        r = self.story.get(sid)
        if r is not None:
            g = r["groupId"]
            if g in self.gpos:
                return (self.gpos[g], 1, r["storySort"], "", 0, r["_line"])
            d = self.source_date.get(sid)
            return NEVER if d is None else self._dated_key(d, sid, r["storySort"], r["_line"])
        d = self.source_date.get(sid)
        if d is not None:
            return self._dated_key(d, sid)
        gs = getattr(self, "group_stories", {}).get(sid)
        if gs:  # a whole group: released once its last story is
            return self._key[gs[-1]]
        if "#" in sid:  # a chunk id
            return self.key(sid.rsplit("#", 1)[0])
        return NEVER

    def max_key(self, ids):
        ks = [self.key(i) for i in ids]
        return max(ks) if ks else NEVER

    def pos_of_key(self, k):
        """The smallest gatePos whose story key is at or after k: a fact with key k is visible at horizonPos h iff
        h >= this. len(ranked) + 1 when no story is."""
        return bisect.bisect_left(self.ranked_keys, k) + 1


def eras_of(O, guide_position=False):
    """The provisional era rule (Ian's decision D7, 2026-10-16: the 22 eras): every main episode, plus an event
    whenever 6 or more guide groups pass without a boundary. position = gatePos of the group's last story (contract);
    groupPosition = the reading-guide group position (the prototype's `position`)."""
    eras, last = [], -99
    for g in sorted(O.guide.values(), key=lambda i: i["position"]):
        if g["kind"] == "main" or g["position"] - last >= 6:
            eras.append(g)
            last = g["position"]
    out = []
    for g in eras:
        gs = [s for s in O.group_stories.get(g["groupId"], []) if O._key[s] != NEVER]
        if not gs:  # a group with no scripted story has no gatePos (horizon/mod.rs skips it too)
            continue
        out.append({"id": g["groupId"], "name": g["name"],
                    "position": g["position"] if guide_position else O.rank[gs[-1]],
                    "groupPosition": g["position"], "_key": O.key(g["groupId"])})
    return out


def sha16(path):
    return sha_file(path)[:16] if os.path.exists(path) else None


def gate_order_doc(O, art, eras):
    """horizon/mod.rs's GateOrderFile, field for field."""
    root = os.path.dirname(art)
    never = [s for s in O.ranked if O._key[s] == NEVER]
    dated = [s for s in O.ranked if O._key[s] != NEVER and O._key[s][1] == 0]
    source_pos = {sid: O.pos_of_key(O.key(sid)) for sid in sorted(O.source_date) if sid not in O.story}
    return {
        "schema": GATE_ORDER_SCHEMA,
        "inputs": {"readingGuide": sha16(os.path.join(root, ORDER_INPUTS[0])),
                   "spoiler": sha16(os.path.join(root, ORDER_INPUTS[1])),
                   "sourceDates": sha16(os.path.join(root, ORDER_INPUTS[2]))},
        "storyOrder": O.ranked,
        "never": never,
        "dated": dated,
        "sourcePos": source_pos,
        "eras": [{"id": e["id"], "name": e["name"], "position": e["position"], "groupPosition": e["groupPosition"]}
                 for e in eras],
    }


def gate_pos_map(doc):
    """storyId -> gatePos from a gate order (stories in `never` have none), and sourceId -> sourcePos."""
    never = set(doc.get("never") or [])
    return ({s: i + 1 for i, s in enumerate(doc.get("storyOrder") or []) if s not in never},
            dict(doc.get("sourcePos") or {}))


def write_json_atomic(path, obj, indent=None):
    common.write_json_atomic(path, obj, newline=True, encoding="utf-8", ensure_ascii=False, indent=indent,
                             separators=None if indent else (",", ":"))


def sync_gate_order(O, art, eras, mode, trevor):
    """Apply --gate-order. Returns (sha of the file as used or None, what happened)."""
    if mode == "off":
        return None, "off"
    path = os.path.join(trevor, GATE_ORDER_REL)
    doc = gate_order_doc(O, art, eras)
    if mode in ("auto", "read") and os.path.exists(path):
        cur = json.load(open(path, encoding="utf-8"))
        fresh = cur.get("schema") == GATE_ORDER_SCHEMA and cur.get("inputs") == doc["inputs"]
        # Only the fields both writers share are compared; extra keys are ignored.
        if fresh or mode == "read":
            if cur.get("storyOrder") != doc["storyOrder"]:
                diff = next((i for i, (a, b) in enumerate(zip(cur.get("storyOrder") or [], doc["storyOrder"])) if a != b),
                            min(len(cur.get("storyOrder") or []), len(doc["storyOrder"])))
                sys.stderr.write(f"gate_order.json disagrees with this script's ranking at gatePos {diff + 1} "
                                 f"({GATE_ORDER_REL}); refusing to build panels on two orders. "
                                 f"--gate-order write rewrites it.\n")
                sys.exit(3)
            for k in ("never", "dated", "sourcePos", "eras"):
                a_, b_ = cur.get(k), doc[k]
                if k == "eras":
                    a_, b_ = [(e.get("id"), e.get("position")) for e in a_ or []], [(e["id"], e["position"]) for e in b_]
                if a_ != b_:
                    sys.stderr.write(f"gate_order.json `{k}` disagrees with this script's; --gate-order write rewrites it.\n")
                    sys.exit(3)
            return sha_file(path), "read (fresh)" if fresh else "read (stale inputs, --gate-order read)"
    if mode == "read":
        sys.stderr.write(f"--gate-order read: {GATE_ORDER_REL} is missing\n")
        sys.exit(3)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    common.write_json_atomic(path, doc, encoding="utf-8", ensure_ascii=False, indent=2)  # serde_json::to_vec_pretty's format
    return sha_file(path), "written"


def corpus_version(trevor, main="artifacts/p3b", p4="artifacts/p4x"):
    """serve.rs `corpus_version` for ask's default corpora: "en-" + the first 16 hex of sha256 over
    "<dir> <chunks sha>\\n" for the main corpus (sha of its chunks.jsonl) and P4x (vectors.meta.json chunksSha).
    None when the main corpus is missing."""
    mp = os.path.join(trevor, main, "chunks.jsonl")
    if not os.path.exists(mp):
        return None
    parts = [(main, sha_file(mp))]
    meta = os.path.join(trevor, p4, "vectors.meta.json")
    if os.path.exists(meta):
        s = json.load(open(meta)).get("chunksSha")
        if s:
            parts.append((p4, s))
    h = hashlib.sha256("".join(f"{d} {s}\n" for d, s in parts).encode()).hexdigest()[:16]
    return f"en-{h}"


def read_res_version(trevor):
    """The asset watcher's EN res_version (assets/ArkAssets/en/.version, written after a successful extract)."""
    p = os.environ.get("TREVOR_RES_VERSION_FILE") or os.path.join(trevor, "..", "assets", "ArkAssets", "en", ".version")
    try:
        return open(p, encoding="utf-8").read().strip() or None
    except OSError:
        return None


# ---------------------------------------------------------------- build
def build(a):
    t0 = time.time()
    art = os.path.join(a.trevor, "artifacts")
    O = Order(art)
    eras = eras_of(O, a.era_guide_position)
    gate_sha, gate_how = sync_gate_order(O, art, eras_of(O), a.gate_order, a.trevor)
    t_order = time.time()

    # --- story texts: summaries, synopses, headers
    p2 = {r["storyId"]: r for r in jl(os.path.join(art, "p2/stories.jsonl"))}
    primers = {r["groupId"]: r for r in jl(os.path.join(art, "primers/primers.jsonl"))}
    first_app = json.load(open(os.path.join(art, "chrono/first_appearance.json"), encoding="utf-8"))
    appear = json.load(open(os.path.join(art, "appearances/appearances.json"), encoding="utf-8"))
    app_chars, app_groups = appear["characters"], appear["groups"]

    def group_name(g):
        if g in O.guide:
            return O.guide[g]["name"]
        if g in app_groups and app_groups[g].get("name"):
            return app_groups[g]["name"]
        gs = O.group_stories.get(g)
        if gs and gs[0] in p2:
            return p2[gs[0]]["header"].split(",")[0]
        return g

    # --- bank
    bank = list(jl(os.path.join(art, "bank/bank.jsonl")))
    for b in bank:
        gs = list(b.get("gateStories") or [])
        if a.gate_sources and gs:
            gs += [s for s in b.get("sources") or [] if s not in gs]
        b["_gate"] = O.max_key(gs) if b.get("gateStories") else NEVER
        b["_gateRank"] = max((O.rank.get(s, 0) for s in b.get("gateStories") or []), default=0)
        if a.gate_sources and b["_gate"] != NEVER:
            b["_gateRank"] = max(b["_gateRank"], min(O.pos_of_key(b["_gate"]), len(O.ranked)))
        b["_groups"] = {O.story[s]["groupId"] for s in b.get("gateStories") or [] if s in O.story}
    who = {}
    for b in bank:
        if b["kind"] == "who":
            who[b["question"][len("Who is "):].rstrip("?")] = b
    asks = [b for b in bank if b["kind"] == "ask" and b["_gate"] != NEVER]
    by_id = {b["id"]: b for b in bank}

    # --- operator archives (official game text; dated by source_dates)
    archive = {}
    ground = a.ground if a.ground != "auto" else ("all" if a.extra_sources else "off")
    need_pass = ({p for t in jl(os.path.join(art, "topics/topics.jsonl")) for p in t["passages"]}
                 if ground != "off" else set())
    pass_text = {}  # passage chunk id -> text, for the grounding check
    with open(os.path.join(art, "p3b/chunks.jsonl"), encoding="utf-8") as f:
        for line in f:
            if need_pass:
                i = line.find('"chunkId":"') + 11
                if line[i:line.find('"', i)] in need_pass:
                    d = json.loads(line)
                    pass_text.setdefault(d["chunkId"], d["text"])
            if '"storyId":"archive_' not in line[:160]:
                continue
            d = json.loads(line)
            if d["ordinal"] != 0:
                continue
            m = re.search(r"Profile:\s*(.+)", d["text"])
            n = re.match(r"Operator archive:\s*(.+)", d["text"])
            if m and n:
                archive[n.group(1).strip()] = (d["storyId"], first_sentence(m.group(1)), O.key(d["storyId"]))

    if need_pass and os.path.exists(os.path.join(art, "p4x/chunks.jsonl")):  # outfits and modules live only in P4x
        with open(os.path.join(art, "p4x/chunks.jsonl"), encoding="utf-8") as f:
            for line in f:
                i = line.find('"chunkId":"') + 11
                cid = line[i:line.find('"', i)]
                if cid in need_pass and cid not in pass_text:
                    pass_text[cid] = json.loads(line)["text"]

    # --- topics: per-sentence gate from cited passages
    extra = set(a.extra_sources)
    stats_extra = collections.Counter()
    topics = {}
    for t in jl(os.path.join(art, "topics/topics.jsonl")):
        sents, more, other = [], [], []

        def cited(s):
            cites = [int(n) for g in CITE.findall(s) for n in re.split(r"\s*,\s*", g)]
            if not cites:
                return None
            ids = [t["passages"][n - 1] for n in cites if 0 < n <= len(t["passages"])]
            return ids if len(ids) == len(cites) else None
        paras = (t.get("summary") or "").split("\n")
        for s in SENT.split(paras[0])[:3]:
            ids = cited(s)
            if ids is not None:
                sents.append((O.max_key(ids), strip_cites(s), ids))
        if "topic" in extra:
            # Later sentences of the summary whose subject is the term: the term or an alias ("The Royal Guard is
            # ..."), or "The <noun>" where the noun is a word of the term's name ("The Union" for Truckers' Union) or
            # a head noun of its kind ("The city" for a place), or "It"/"They". Each is still gated by every passage
            # it cites.
            alts = sorted({x for x in (t["topic"], *(t.get("aliases") or [])) if x}, key=len, reverse=True)
            heads = {w.lower() for x in alts for w in tokens(x) if len(w) > 3} | KIND_HEADS.get(t["kind"], set())
            subj = re.compile(r"^(?:The |the )?(?:" + "|".join(re.escape(x) for x in alts) + r")\b")
            rest = SENT.split(paras[0])[3:] + [s for para in paras[1:] for s in SENT.split(para)]
            for s in rest:
                ids = cited(s)
                txt = strip_cites(s)
                if ids is None or len(txt) > 300 or TOPIC_META.search(txt) or "*" in txt:
                    continue
                m = re.match(r"^The ([a-z][a-z'’\-]+)\b", txt)
                if subj.match(txt) or (m and m.group(1) in heads) or re.match(r"^(?:It|They) [a-z]", txt):
                    more.append((O.max_key(ids), txt, ids))
                elif not TOPIC_WEAK_START.match(txt) and (
                        {w.lower() for w in tokens(txt)} & {w.lower() for x in alts for w in tokens(x) if len(w) > 3}):
                    # any other sentence must at least name the term ("Beanstalk is a Metal Crab breeder" under
                    # Reproba is about one member, not the race)
                    other.append((O.max_key(ids), txt, ids))
        topics[t["topic"]] = {"kind": t["kind"], "sents": sents, "more": more + other,
                              "aliases": t.get("aliases") or [], "passages": t["passages"]}
    exact, lower, maxlen = {}, {}, 1
    for name, t in topics.items():
        for al in {name, *t["aliases"]}:
            tk = tuple(tokens(al))
            if not tk:
                continue
            maxlen = max(maxlen, len(tk))
            if al == al.lower():
                lower.setdefault(tuple(x.lower() for x in tk), name)
            else:
                exact.setdefault(tk, name)

    # --- one pass over story chunks: speakers and term mentions per story
    speakers = collections.defaultdict(collections.Counter)
    terms = collections.defaultdict(collections.Counter)
    intro_raw = []  # (story key, chunk ordinal, name, kind, line, storyId), in file order
    mention_raw = []  # (story key, chunk ordinal, name words, phrase, speaker, storyId)
    speakers_seen = set()
    for d in jl(os.path.join(art, "chunks.jsonl")):
        sid = d["storyId"]
        if d["chunkId"] in need_pass:
            pass_text.setdefault(d["chunkId"], d["text"])
        for s in d.get("speakers") or []:
            speakers[sid][s] += 1
        if "mention" in extra:
            for words, phrase, spk in story_mentions(d["text"]):
                mention_raw.append((O.key(sid), d["ordinal"], words, phrase, spk, sid))
        if "intro" in extra:
            speakers_seen.update(d.get("speakers") or [])
            for n, kind, line in story_intros(d["text"], speakers_seen):
                intro_raw.append((O.key(sid), d["ordinal"], n, kind, line, sid))
        tk = tokens(d["text"])
        lk = [x.lower() for x in tk]
        i, n = 0, len(tk)
        while i < n:
            hit = 0
            for L in range(min(maxlen, n - i), 0, -1):
                name = exact.get(tuple(tk[i:i + L])) or lower.get(tuple(lk[i:i + L]))
                if name:
                    terms[sid][name] += 1
                    hit = L
                    break
            i += hit or 1
    term_first = {}
    for sid in O.ranked:
        for name in terms.get(sid, ()):
            term_first.setdefault(name, sid)

    # --- grounding: a model-written summary sentence may carry facts from passages it does not cite ("The current
    # Speaker of Parliament is Vina Victoria [4]", where [4] is an early outfit text and Vina's role comes from a
    # later event). Every name and quoted phrase of a sentence must occur in its cited passages; one that does not
    # is looked up in the topic's other passages and the sentence waits for the earliest that has it (that passage
    # joins its sources). A name in no passage at all drops the sentence.
    def evidenced(t, names, x, what):
        k, txt, ids = x
        cited = " ".join(pass_text.get(i, "") for i in ids)
        if not cited.strip():
            stats_extra[f"ground_{what}_dropped"] += 1
            return None
        ev = []
        for item in ungrounded_items(txt, cited, names):
            best = min(((O.key(p_), p_) for p_ in t["passages"] if has_item(item, _norm(pass_text.get(p_, "")))),
                       default=None)
            if best is None:
                stats_extra[f"ground_{what}_dropped"] += 1
                return None
            ev.append(best)
        if ev:
            stats_extra[f"ground_{what}_delayed" if max(ev)[0] > k else f"ground_{what}_evidenced"] += 1
            k = max(k, max(ev)[0])
            ids = ids + [p_ for _, p_ in ev if p_ not in ids]
        return (k, txt, ids)
    for name, t in topics.items():
        names = [name, *t["aliases"]]
        if ground == "all":
            t["sents"] = [y for y in (evidenced(t, names, x, "first3") for x in t["sents"]) if y]
        if ground != "off":
            t["more"] = [y for y in (evidenced(t, names, x, "later") for x in t["more"]) if y]
        t["sents"] = [(k, s_) for k, s_, *_ in t["sents"]]

    # --- extra identity sources: per name, candidates (visible from key, kind, text, provenance)
    # A candidate is shown on a panel with key K when its key is < K (story lines, synopses: strictly earlier
    # stories) or <= K (an enemy's stage, an operator file). Each name keeps its earliest candidate per kind.
    id_cand = collections.defaultdict(dict)  # name -> kind -> (key, strict, text, prov)

    def offer(name, kind, key, strict, text, prov):
        if key == NEVER or not text:
            return
        cur = id_cand[name].get(kind)
        if cur is None or (key, text) < (cur[0], cur[2]):
            id_cand[name][kind] = (key, strict, tidy(text), prov)
    all_names = {n for c in speakers.values() for n in c}
    if "attrs" in extra:
        for at in jl(os.path.join(art, "entities/operator_attributes.jsonl")):
            src = "archive_" + at["charId"]
            if at["name"] in all_names and src in O.source_date:
                offer(at["name"], "attrs", O.key(src), False, basic_info_line(at), {"kind": "attrs", "source": src})
    if "enemy" in extra:
        ep = os.path.join(art, "chrono/enemy_intros.json")
        enemies = json.load(open(ep, encoding="utf-8"))["enemies"] if os.path.exists(ep) else {}
        stats_extra["enemyIntrosFile"] = os.path.exists(ep)
        for name, rows in enemies.items():
            if name not in all_names:
                continue
            for r in rows:
                for st in r["stages"]:
                    if st["story"] not in O.story:
                        continue
                    k = O.key(st["story"])
                    # after: visible from the story AFTER the stage's _beg story, i.e. on keys > k
                    offer(name, "enemy", k, st["after"], lead_sentences(r["description"]),
                          {"kind": "enemy", "source": st["story"], "after": st["after"], "enemyId": r["enemyId"]})
    # An appositive names a person; a label that is a term (Sankta, Feline) or a role (the Doctor, "Rhodes Island
    # Operator") gets none from a synopsis or a passing mention.
    term_names = {x for name, t in topics.items() for x in (name, *t["aliases"])}
    person = {n for n in all_names if n not in term_names and n.split(" ")[-1] not in ROLE_WORDS}
    if "synopsis" in extra:
        for sid in O.ranked:
            for n, line in synopsis_appositives(p2.get(sid, {}).get("officialSynopsis"), person):
                offer(n, "synopsis", O.key(sid), True, line, {"kind": "synopsis", "source": sid})
    for k, _o, n, kind, line, sid in sorted(intro_raw, key=lambda x: (x[0], x[1])):
        if n in all_names:
            offer(n, "intro", k, True, line, {"kind": "intro", "source": sid})
    for k, _o, words, phrase, spk, sid in sorted(mention_raw, key=lambda x: (x[0], x[1])):
        for i in range(len(words)):  # the longest suffix that is a speaker label
            n = " ".join(words[i:])
            if n in all_names:
                if n != spk and n in person:
                    offer(n, "mention", k, True, f"{n}, {phrase}.", {"kind": "mention", "source": sid})
                break
    EXTRA_ORDER = ("attrs", "enemy", "synopsis", "intro", "mention")  # after the operator file and the bank
    t_load = time.time()

    era_keys = [(e["_key"], e) for e in eras]

    def era_of(k):
        best = None
        for ek, e in era_keys:
            if ek <= k:
                best = e
        return None if best is None else {"id": best["id"], "name": best["name"], "position": best["position"]}

    # ------------------------------------------------------------ per story
    stage = a.out.rstrip("/") + ".tmp"
    shutil.rmtree(stage, ignore_errors=True)
    os.makedirs(os.path.join(stage, "stories"))
    stats = collections.Counter()
    sizes, sizes_noans, bodies = [], [], []
    ctx = {"O": O, "by_id": by_id, "archive": archive, "who": who, "topics": topics, "era_keys": era_keys}
    violations = []
    provenance = {}  # storyId -> {"identity": {name: rec}, "definition": {term: rec}}, extra sources only
    gate_gp, gate_sp = gate_pos_map(gate_order_doc(O, art, eras_of(O)))  # the check's own positions
    injected = 0
    for sid in O.ranked:
        row = O.story[sid]
        g = row["groupId"]
        K = O.key(sid)
        R = O.rank[sid]
        me = p2.get(sid, {})
        gstories = O.group_stories[g]
        idx = gstories.index(sid)
        earlier = gstories[:idx]

        # Recap so far
        recap = {
            "thisStory": {"title": me.get("header"), "synopsis": me.get("officialSynopsis"),
                          "summary": me.get("summary"), "spoiler": True} if me else None,
            "previous": None, "soFar": [], "earlierCount": len(earlier),
            "beforeYouStart": None,
            "buildsOn": list(O.guide.get(g, {}).get("buildsOn") or []),
        }
        if earlier and earlier[-1] in p2:
            pv = p2[earlier[-1]]
            recap["previous"] = {"storyId": pv["storyId"], "title": pv["header"], "summary": pv["summary"]}
        for e in earlier[-(a.max_sofar + 1):-1]:
            if e in p2:
                recap["soFar"].append({"storyId": e, "title": p2[e]["header"], "synopsis": p2[e]["officialSynopsis"]})
        pr = primers.get(g)
        pb = by_id.get(f"primer:{g}")
        if pr and (pb is None or pb["_gate"] <= K):
            if idx == 0:
                recap["beforeYouStart"] = {"primer": pr["primer"], "prior": pr.get("prior") or []}
            else:
                recap["beforeYouStart"] = {"bankId": f"primer:{g}"}
        recap_nonempty = bool(recap["previous"] or recap["soFar"] or (recap["beforeYouStart"] and idx == 0))

        # Who's here
        chars, more = [], 0
        sp = speakers.get(sid, collections.Counter())
        ranked_sp = sorted(sp.items(), key=lambda kv: (-kv[1], kv[0]))
        for name, cnt in ranked_sp:
            known = name in who or name in archive
            if not known and (GENERIC.search(name) or len(name) > 40):
                continue
            if len(chars) >= a.max_chars:
                more += 1
                continue
            ident, src, wid = None, None, None
            if name in archive and archive[name][2] <= K:
                ident, src = archive[name][1], "operatorFile"
            w = who.get(name)
            if w and w["_gate"] <= K:
                wid = w["id"]
                if ident is None:
                    ident, src = first_sentence(w["answer"]), "bank"
            if ident is None and name in id_cand:
                for kind in EXTRA_ORDER:
                    c = id_cand[name].get(kind)
                    if c and (c[0] < K if c[1] else c[0] <= K):
                        ident = c[2]
                        src = "operatorFile" if kind == "attrs" else (EXTRA_LABEL[kind] if a.identity_source_labels
                                                                      else None)
                        provenance.setdefault(sid, {}).setdefault("identity", {})[name] = dict(c[3], text=ident)
                        stats_extra["identity_" + kind] += 1
                        break
            fa = first_app.get(name)
            fa_k = O.key(fa["storyId"]) if fa else NEVER
            seen = []
            for gg in (app_chars.get(name) or {}).get("speaks", {}):
                if gg != g and O.key(gg) <= K:
                    seen.append(gg)
            seen.sort(key=lambda x: O.key(x))
            chars.append({
                "name": name, "chunks": cnt, "identity": ident, "identitySource": src, "whoId": wid,
                "debut": bool(fa and fa["storyId"] == sid),
                "firstSeen": ({"storyId": fa["storyId"], "group": fa["group"]} if fa and fa_k < K else None),
                "seenInGroups": len(seen), "recentGroups": [group_name(x) for x in seen[-3:]],
            })
        stats["chars_total"] += len(chars)
        stats["chars_with_identity"] += sum(1 for c in chars if c["identity"])

        # Terms
        items, tmore = [], 0
        for name, cnt in sorted(terms.get(sid, {}).items(), key=lambda kv: (-kv[1], kv[0])):
            if len(items) >= a.max_terms:
                tmore += 1
                continue
            t = topics[name]
            defn = next((s for k, s in t["sents"] if k <= K), None)
            if defn is None:
                more_hit = next(((s, ids) for k, s, ids in t["more"] if k <= K), None)
                if more_hit:
                    defn = more_hit[0]
                    provenance.setdefault(sid, {}).setdefault("definition", {})[name] = {
                        "kind": "topic", "topic": name, "passages": more_hit[1], "text": tidy(defn)}
                    stats_extra["definition_topic"] += 1
            fs = term_first.get(name)
            items.append({"term": name, "kind": t["kind"], "mentions": cnt, "definition": defn,
                          "firstNamed": None if fs is None or fs == sid else
                          {"storyId": fs, "group": group_name(O.story[fs]["groupId"])}})
        stats["terms_total"] += len(items)
        stats["terms_with_def"] += sum(1 for t in items if t["definition"])

        # Suggested questions
        sp_names = [c["name"] for c in chars[:8]]
        sp_re = re.compile(r"\b(" + "|".join(re.escape(n) for n in sp_names) + r")\b") if sp_names else None
        cand = []
        for b in asks:
            if b["_gate"] > K:
                continue
            score = 0
            if g in b["_groups"]:
                score += 3
            if sp_re:
                score += 2 * len(set(sp_re.findall(b["question"])))
            if score == 0:
                continue
            same = sid in (b.get("gateStories") or [])
            cand.append((-(score - (1 if same else 0)), -b["_gateRank"], b, same))
        cand.sort(key=lambda x: (x[0], x[1], x[2]["id"]))
        picks = [(b, same) for _, _, b, same in cand[:3]]
        if pb and pb["_gate"] <= K and idx <= 2:
            picks.append((pb, False))
        for c in chars:
            if len([p for p in picks if p[0]["kind"] == "who"]) >= 2:
                break
            if c["whoId"]:
                picks.append((by_id[c["whoId"]], False))
        if earlier:
            sb = by_id.get(f"story:{earlier[-1]}")
            if sb and sb["_gate"] <= K:
                picks.append((sb, False))
        for gname in reversed(recap["buildsOn"]):
            gid = next((x for x, i in O.guide.items() if i["name"] == gname), None)
            eb = by_id.get(f"event:{gid}") if gid else None
            if eb and eb["_gate"] <= K:
                picks.append((eb, False))
                break
        seen_ids, sugg = set(), []
        for b, same in picks:
            if b["id"] in seen_ids or len(sugg) >= a.max_suggest:
                continue
            seen_ids.add(b["id"])
            s = {"id": b["id"], "kind": b["kind"], "question": tidy(b["question"]), "gatePos": b["_gateRank"],
                 "fromThisStory": same}
            if not a.no_answers:
                s["answer"] = tidy(strip_cites(b["answer"]))
            sugg.append(s)

        panel = {
            "schema": SCHEMA, "storyId": sid, "groupId": g, "group": group_name(g),
            "title": me.get("header"), "position": R, "era": era_of(K),
            "recap": recap,
            "whosHere": {"characters": chars, "more": more},
            "terms": {"items": items, "more": tmore},
            "suggestions": {"items": sugg},
        }
        if a.inject_late and injected < 2 * a.inject_late and chars and items and R < len(O.ranked):
            # The canary for the check below: a line whose source is the NEXT story (gatePos R + 1) must be caught.
            later = O.ranked[R]
            chars[0]["identity"], chars[0]["identitySource"] = "INJECTED line from a later story", None
            items[0]["definition"] = "INJECTED definition citing a later story"
            pv = provenance.setdefault(sid, {})
            pv.setdefault("identity", {})[chars[0]["name"]] = {"kind": "intro", "source": later,
                                                               "text": chars[0]["identity"]}
            pv.setdefault("definition", {})[items[0]["term"]] = {"kind": "topic", "topic": items[0]["term"],
                                                                 "passages": [later + "#0000"],
                                                                 "text": items[0]["definition"]}
            injected += 2
        body = tidy(dumps_compact(panel)).encode("utf-8")
        written = json.loads(body)
        violations += verify_panel(written, ctx, provenance.get(sid))
        violations += check_new_lines(written, provenance.get(sid), gate_gp, gate_sp)
        with open(os.path.join(stage, "stories", sid + ".json"), "wb") as f:
            f.write(body)
        sizes.append(len(body))
        if not a.no_answers:
            p2c = dict(panel, suggestions={"items": [{k: v for k, v in s.items() if k != "answer"} for s in sugg]})
            sizes_noans.append(len(dumps_compact(p2c).encode("utf-8")))
        bodies.append((sid, body))
        stats["stories"] += 1
        stats["recap"] += recap_nonempty
        stats["recap_incl_thisStory"] += bool(recap_nonempty or recap["thisStory"])
        stats["whosHere"] += bool(chars)
        stats["whosHere_with_identity"] += any(c["identity"] for c in chars)
        stats["terms"] += bool(items)
        stats["terms_any_def"] += any(t["definition"] for t in items)
        stats["suggestions"] += bool(sugg)
        stats["suggestions_ask"] += any(s["kind"] == "ask" for s in sugg)
        stats["all_four"] += bool(recap_nonempty and chars and items and sugg)
    t_build = time.time()

    # ------------------------------------------------------------ sizes, manifest
    all_raw = b"{" + b",".join(json.dumps(s).encode() + b":" + b for s, b in bodies) + b"}"
    all_gz = gzip.compress(all_raw, 6)
    panels_sha = hashlib.sha256(all_raw).hexdigest()
    cv = a.corpus_version or corpus_version(a.trevor)
    version = a.version or f"{cv or 'en-nocorpus'}-p{panels_sha[:8]}"
    built_from = a.built_from or os.environ.get("TREVOR_RES_VERSION") or read_res_version(a.trevor)
    q = lambda xs, p: sorted(xs)[min(len(xs) - 1, int(round(p * (len(xs) - 1))))]
    st = {
        "stories": stats["stories"],
        "panelsNonEmpty": {k: stats[k] for k in ("recap", "recap_incl_thisStory", "whosHere", "whosHere_with_identity",
                                                   "terms", "terms_any_def", "suggestions", "suggestions_ask", "all_four")},
        "items": {"characters": stats["chars_total"], "charactersWithIdentity": stats["chars_with_identity"],
                  "terms": stats["terms_total"], "termsWithDefinition": stats["terms_with_def"]},
        "perStoryBytes": {"min": min(sizes), "median": int(statistics.median(sizes)), "p95": q(sizes, .95),
                          "max": max(sizes), "mean": int(statistics.mean(sizes))},
        "perStoryBytesNoAnswers": ({"min": min(sizes_noans), "median": int(statistics.median(sizes_noans)),
                                    "p95": q(sizes_noans, .95), "max": max(sizes_noans), "total": sum(sizes_noans)}
                                   if sizes_noans else None),
        "totalRawBytes": len(all_raw), "totalGzipBytes": len(all_gz), "gzipRatio": round(len(all_gz) / len(all_raw), 3),
        "gateViolations": len(violations),
        "extraSources": dict(sorted(stats_extra.items()), sources=sorted(extra)),
        "undatedStories": sum(1 for s in O.ranked if O._key[s] == NEVER),
        "gateOrder": gate_how,
        "eras": [{k: e[k] for k in ("id", "name", "position", "groupPosition")} for e in eras],
        "seconds": {"order": round(t_order - t0, 2), "load+scan": round(t_load - t_order, 2),
                    "build+write": round(t_build - t_load, 2), "total": round(time.time() - t0, 2)},
        "args": {k: v for k, v in vars(a).items()},
    }
    manifest = {
        "schema": SCHEMA, "version": version, "server": a.server, "corpusVersion": cv, "builtFrom": built_from,
        "builtAt": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "storyOrder": O.ranked,
        "eras": [{"id": e["id"], "name": e["name"], "position": e["position"]} for e in eras],
        "panels": len(bodies), "panelsSha": panels_sha, "gateOrderSha": gate_sha,
        "answersInline": not a.no_answers,
        "stats": {k: st[k] for k in ("panelsNonEmpty", "items", "totalRawBytes", "totalGzipBytes", "gateViolations")},
    }
    write_json_atomic(os.path.join(stage, "manifest.json"), manifest)
    if extra:
        # Not published: where each extra-source line came from, for the gate check and for review.
        write_json_atomic(os.path.join(stage, "provenance.json"), provenance)
    with open(os.path.join(stage, "stats.json"), "w", encoding="utf-8") as f:
        json.dump(st, f, indent=2, ensure_ascii=False)
    for v in violations[:20]:
        sys.stderr.write(f"VIOLATION {v}\n")
    if a.inject_late:
        sys.stderr.write(f"inject-late: {injected} late lines injected, {len(violations)} violations reported\n")
    if violations:
        sys.stderr.write(f"{len(violations)} gate violations; {a.out} left unchanged, the build is in {stage}\n")
        return 1
    old = a.out.rstrip("/") + ".old"
    shutil.rmtree(old, ignore_errors=True)
    if os.path.exists(a.out):
        os.replace(a.out, old)
    os.replace(stage, a.out)
    shutil.rmtree(old, ignore_errors=True)
    json.dump({k: v for k, v in st.items() if k not in ("eras", "args")}, sys.stdout, indent=2)
    print()
    print(f"version {version} (corpus {cv}, builtFrom {built_from}); {len(bodies)} panels in {a.out}; "
          f"gate order {gate_how}; 0 gate violations")
    return 0


# ---------------------------------------------------------------- the post-build check
def verify_panel(p, ctx, prov=None):
    """Every fact of one written panel checked against the release order from the panel's own content. Returns
    a list of violation strings (empty when the panel is clean)."""
    O, by_id, archive, who, topics = ctx["O"], ctx["by_id"], ctx["archive"], ctx["who"], ctx["topics"]
    sid = p["storyId"]
    K = O.key(sid)
    out = []
    bad = lambda what: out.append(f"{sid}: {what}")
    if p["position"] != O.rank[sid]:
        bad(f"position {p['position']} != gatePos {O.rank[sid]}")
    r = p["recap"]
    if r["previous"] and not O.key(r["previous"]["storyId"]) < K:
        bad(f"previous {r['previous']['storyId']}")
    for e in r["soFar"]:
        if not O.key(e["storyId"]) < K:
            bad(f"soFar {e['storyId']}")
    b = r["beforeYouStart"]
    if b:
        pb = by_id.get(f"primer:{p['groupId']}")
        if pb is not None and not pb["_gate"] <= K:
            bad("primer")
    for c in p["whosHere"]["characters"]:
        if c["firstSeen"] and not O.key(c["firstSeen"]["storyId"]) < K:
            bad(f"firstSeen {c['name']}")
        has_prov = c["name"] in (prov or {}).get("identity", {})  # an extra source: check_new_lines checks it
        if c["identitySource"] == "operatorFile" and not has_prov and not (c["name"] in archive and
                                                                         archive[c["name"]][2] <= K):
            bad(f"operator file {c['name']}")
        if c["identitySource"] == "bank" and not (c["name"] in who and who[c["name"]]["_gate"] <= K):
            bad(f"who {c['name']}")
        if c["whoId"] and not by_id[c["whoId"]]["_gate"] <= K:
            bad(f"whoId {c['whoId']}")
    for t in p["terms"]["items"]:
        if t["firstNamed"] and not O.key(t["firstNamed"]["storyId"]) < K:
            bad(f"firstNamed {t['term']}")
        if t["definition"] is not None and t["term"] not in (prov or {}).get("definition", {}):
            ks = [k for k, s in topics[t["term"]]["sents"] if tidy(s) == t["definition"] or s == t["definition"]]
            if not ks or min(ks) > K:
                bad(f"definition {t['term']}")
    for s in p["suggestions"]["items"]:
        bk = by_id.get(s["id"])
        if bk is None or not bk["_gate"] <= K:
            bad(f"suggestion {s['id']}")
        if s["gatePos"] > p["position"]:
            bad(f"suggestion gatePos {s['id']} {s['gatePos']} > {p['position']}")
    if p["era"] is not None:
        ek = next((k for k, e in ctx["era_keys"] if e["id"] == p["era"]["id"]), NEVER)
        if not ek <= K:
            bad(f"era {p['era']['id']}")
    return out


# ---------------------------------------------------------------- self-test
def self_test():
    """The order rule on a toy release (ties, dated and undated records, a dated source) and the era rule."""
    import tempfile
    d = tempfile.mkdtemp()
    art = os.path.join(d, "artifacts")
    os.makedirs(os.path.join(art, "chrono"))
    guide = {"items": [{"groupId": "main_0", "name": "E0", "kind": "main", "position": 1, "released": "by 2020-01-22"},
                       {"groupId": "act1", "name": "A1", "kind": "event", "position": 2, "released": "2020-03-01"},
                       {"groupId": "main_1", "name": "E1", "kind": "main", "position": 3, "released": "about 2020-07-01"}]}
    json.dump(guide, open(os.path.join(art, "chrono/reading_guide.json"), "w"))
    rows = [("m0_a", "main_0", 1), ("a1_b", "act1", 2), ("a1_a", "act1", 2), ("m1_a", "main_1", 1),
            ("story_x_set_1_story_1", "story_x_set_1", 1), ("story_y_set_1_story_1", "story_y_set_1", 1)]
    with open(os.path.join(art, "spoiler.jsonl"), "w") as f:
        for s, g, n in rows:
            f.write(json.dumps({"storyId": s, "groupId": g, "storySort": n}) + "\n")
    day = parse_day("2020-05-01")
    json.dump({"dates": {"story_x_set_1_story_1": day, "archive_char_x": parse_day("2020-02-01")}},
              open(os.path.join(art, "chrono/source_dates.json"), "w"))
    O = Order(art)
    # Ties at storySort 2 break by file line: a1_b before a1_a. The dated record (2020-05-01) sits before main_1 (the
    # first group whose date admits it); the undated record is last.
    assert O.ranked == ["m0_a", "a1_b", "a1_a", "story_x_set_1_story_1", "m1_a", "story_y_set_1_story_1"], O.ranked
    doc = gate_order_doc(O, art, eras_of(O))
    assert doc["never"] == ["story_y_set_1_story_1"] and doc["dated"] == ["story_x_set_1_story_1"]
    # archive dated 2020-02-01: admitted by act1 (2020-03-01), so visible from act1's first story, gatePos 2.
    assert doc["sourcePos"] == {"archive_char_x": 2}, doc["sourcePos"]
    gp, sp = gate_pos_map(doc)
    assert "story_y_set_1_story_1" not in gp and gp["m1_a"] == 5 and sp == {"archive_char_x": 2}
    eras = doc["eras"]
    assert [e["id"] for e in eras] == ["main_0", "main_1"] and [e["position"] for e in eras] == [1, 5], eras
    assert [e["groupPosition"] for e in eras] == [1, 3]
    # The written file round-trips through --gate-order auto (fresh -> read, verified).
    os.makedirs(os.path.join(d, "artifacts", "chrono"), exist_ok=True)
    sha1, how1 = sync_gate_order(O, art, eras_of(O), "auto", d)
    sha2, how2 = sync_gate_order(O, art, eras_of(O), "auto", d)
    assert how1 == "written" and how2 == "read (fresh)" and sha1 == sha2, (how1, how2)
    shutil.rmtree(d)
    # The extra-source gate: story lines strictly earlier, an enemy stage at or before (after = the next gatePos),
    # cited passages at or before by gatePos or sourcePos, undated never.
    gp2, sp2 = {"s1": 1, "s2": 2, "s3": 3}, {"archive_x": 2}

    def one(kind, src, here, **kw):
        p = {"storyId": "s", "position": here, "terms": {"items": []},
             "whosHere": {"characters": [{"name": "N", "identity": "line", "identitySource": None}]}}
        return check_new_lines(p, {"identity": {"N": dict(kind=kind, source=src, text="line", **kw)}}, gp2, sp2)
    assert not one("intro", "s1", 2) and one("intro", "s2", 2) and one("synopsis", "s3", 2)
    assert not one("mention", "s1", 2) and one("mention", "s2", 2)
    assert not one("enemy", "s2", 2, after=False) and one("enemy", "s2", 2, after=True)
    assert not one("enemy", "s2", 3, after=True) and not one("attrs", "archive_x", 2) and one("attrs", "archive_x", 1)
    assert one("intro", "undated_story", 3) and one("other", "s1", 3)
    p = {"storyId": "s", "position": 2, "terms": {"items": []},
         "whosHere": {"characters": [{"name": "N", "identity": "line", "identitySource": None}]}}
    assert check_new_lines(p, None, gp2, sp2), "an identity with no source and no provenance must be caught"
    for cites, bad in ((["s1#0001", "archive_x#0000"], False), (["s3#0000"], True), (["enemy_y#0000"], True),
                       ([], True)):
        p = {"storyId": "s", "position": 2, "whosHere": {"characters": []},
             "terms": {"items": [{"term": "T", "definition": "d"}]}}
        got = check_new_lines(p, {"definition": {"T": {"kind": "topic", "passages": cites, "text": "d"}}}, gp2, sp2)
        assert bool(got) == bad, (cites, got)
    assert story_intros("Elena: Yes! I'm Elena, a researcher in the Energy Section. Sorry.", set()) == [
        ("Elena", "self", "Elena: “I'm Elena, a researcher in the Energy Section.”")]
    assert story_intros("Talulah: I am Talulah, of course.", set()) == []  # no identity in it
    assert not grounded("The current Speaker of Parliament is Vina Victoria.",
                        ["Gnosis became Speaker of the Parliament of Kjerag."], ["Parliament"])
    assert grounded("The Victorian city is known as the \"Iron City,\" a place.", ["Victoria's iron city"], ["X"])
    assert story_mentions("Raidian: she told Rankin, the foreman, to go.")[0][:2] == (["Rankin"], "the foreman")
    assert story_mentions("Elysium: Miss Alty, the town has become quite dangerous.") == []
    assert story_mentions("Joanna: After greeting Enya, the girl looks aside and falls quiet again.") == []
    assert story_mentions("Chester: As requested by Sir Arctosz, the leader of Rhodes Island, Dr. Ling.") == []
    assert story_mentions("She thinks about the drunken Diane, the departing Glasgow Gang, and the crown.") == []
    assert story_mentions("With Caesar, the Signori dei Lupi surround Vargo, shifting.") == []
    assert synopsis_appositives("After meeting Mostima, the two blend into the crowd, unseen.", {"Mostima"}) == []
    assert synopsis_appositives("Led by Siege, the Exemplars arrive in Gibsonham, where...", {"Siege"}) == []
    assert synopsis_appositives("It turns out that Zuo Le, the youth, and Zheng did it.", {"Zuo Le"}) == []
    assert synopsis_appositives("Wei Yenwu, the head of Lungmen, waits.", {"Wei Yenwu"}) == [
        ("Wei Yenwu", "Wei Yenwu, the head of Lungmen.")]
    assert story_mentions("Amiya: we fight Theresa, the invincible forces of nature, and more.") == []
    assert synopsis_appositives("At dawn, Cecelia, a resident of the outskirts, meets them.", {"Cecelia"}) == [
        ("Cecelia", "Cecelia, a resident of the outskirts.")]
    print("panels.py self-test: ok (order, ties, dated and undated records, sourcePos, eras, gate_order round trip, "
          "extra-source gate and extractors)")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--trevor", default=ROOT, help="the trevor/ directory (reads artifacts/)")
    ap.add_argument("--out", default=None, help="output directory (default <trevor>/artifacts/panels)")
    ap.add_argument("--version", default=None, help="default <corpusVersion>-p<first 8 hex of the panels sha>")
    ap.add_argument("--corpus-version", default=None, help="default: serve.rs's rule over artifacts/p3b and p4x")
    ap.add_argument("--built-from", default=None, help="EN res_version; default $TREVOR_RES_VERSION, else "
                    "../assets/ArkAssets/en/.version ($TREVOR_RES_VERSION_FILE)")
    ap.add_argument("--server", default="en")
    ap.add_argument("--gate-order", choices=("auto", "read", "write", "off"), default="auto")
    ap.add_argument("--era-guide-position", action="store_true",
                    help="kill switch: era.position = reading-guide group position (the prototype), not gatePos")
    ap.add_argument("--no-gate-sources", dest="gate_sources", action="store_false",
                    help="kill switch: gate bank entries by gateStories only (the prototype)")
    ap.add_argument("--no-answers", action="store_true", help="suggestions carry ids only, no answer text")
    ap.add_argument("--extra-sources", default=",".join(EXTRA_SOURCES),
                    help="comma list of the 2026-10-17 identity/definition sources (" + ",".join(EXTRA_SOURCES) +
                    "); 'none' is the kill switch: the panels before them, byte for byte")
    ap.add_argument("--ground", choices=("auto", "all", "new", "off"), default="auto",
                    help="topic definitions must name only what their cited passages name: all sentences, only the "
                    "2026-10-17 ones, or off; auto = all, or off under --extra-sources none")
    ap.add_argument("--identity-source-labels", action="store_true",
                    help="write enemyHandbook/storySynopsis/storyLine as identitySource (needs gates.py's schema "
                    "to accept them); default null")
    ap.add_argument("--inject-late", type=int, default=0, metavar="N",
                    help="canary: put N identities and N definitions from a later story into the build; the gate "
                    "check must report all 2N and the build exits 1 (use with --out on a scratch directory)")
    ap.add_argument("--max-chars", type=int, default=12)
    ap.add_argument("--max-terms", type=int, default=8)
    ap.add_argument("--max-suggest", type=int, default=6)
    ap.add_argument("--max-sofar", type=int, default=8)
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        self_test()
        return 0
    a.extra_sources = [] if a.extra_sources in ("none", "") else a.extra_sources.split(",")
    bad = [x for x in a.extra_sources if x not in EXTRA_SOURCES]
    if bad:
        ap.error(f"--extra-sources: unknown {bad}")
    a.trevor = os.path.abspath(a.trevor)
    a.out = os.path.abspath(a.out or os.path.join(a.trevor, "artifacts", "panels"))
    return build(a)


if __name__ == "__main__":
    sys.exit(main())
