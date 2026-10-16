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
(horizon.rs, the panels, trevor_panels.gate_pos, trevor_answers.gate_pos). The schema is horizon.rs's
(`GateOrderFile`, src/search/horizon.rs, also written by `ask --gate-order`); this script writes the same structure
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
  Rule (the prototype's, which is horizon.rs's old rule plus a strict tie-break): a story in a reading-guide group
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

Usage: python3 scripts/panels.py [--out artifacts/panels] [--version V] [--built-from RES_VERSION]
                                 [--gate-order auto|read|write|off] [--no-answers] [--self-test]
"""
import argparse, bisect, collections, gzip, hashlib, json, os, re, shutil, statistics, sys, time

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


def sha_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def parse_day(s):
    m = re.search(r"(\d{4})-(\d{2})-(\d{2})", s or "")
    if not m:
        return None
    y, mo, d = (int(x) for x in m.groups())
    # days from civil (same as horizon.rs)
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


# ---------------------------------------------------------------- release order
class Order:
    """Total release order over stories and dated sources, equivalent to horizon.rs `allows` (plus the tie-break)."""

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
        if not gs:  # a group with no scripted story has no gatePos (horizon.rs skips it too)
            continue
        out.append({"id": g["groupId"], "name": g["name"],
                    "position": g["position"] if guide_position else O.rank[gs[-1]],
                    "groupPosition": g["position"], "_key": O.key(g["groupId"])})
    return out


def sha16(path):
    return sha_file(path)[:16] if os.path.exists(path) else None


def gate_order_doc(O, art, eras):
    """horizon.rs's GateOrderFile, field for field."""
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
    tmp = path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(obj, f, ensure_ascii=False, indent=indent, separators=None if indent else (",", ":"))
        f.write("\n")
    os.replace(tmp, path)


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
    tmp = path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        f.write(json.dumps(doc, ensure_ascii=False, indent=2))  # serde_json::to_vec_pretty's format
    os.replace(tmp, path)
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
    with open(os.path.join(art, "p3b/chunks.jsonl"), encoding="utf-8") as f:
        for line in f:
            if '"storyId":"archive_' not in line[:160]:
                continue
            d = json.loads(line)
            if d["ordinal"] != 0:
                continue
            m = re.search(r"Profile:\s*(.+)", d["text"])
            n = re.match(r"Operator archive:\s*(.+)", d["text"])
            if m and n:
                archive[n.group(1).strip()] = (d["storyId"], first_sentence(m.group(1)), O.key(d["storyId"]))

    # --- topics: per-sentence gate from cited passages
    topics = {}
    for t in jl(os.path.join(art, "topics/topics.jsonl")):
        sents = []
        for s in SENT.split((t.get("summary") or "").split("\n")[0])[:3]:
            cites = [int(n) for g in CITE.findall(s) for n in re.split(r"\s*,\s*", g)]
            if not cites:
                continue
            ids = [t["passages"][n - 1] for n in cites if 0 < n <= len(t["passages"])]
            if len(ids) != len(cites):
                continue
            sents.append((O.max_key(ids), strip_cites(s)))
        topics[t["topic"]] = {"kind": t["kind"], "sents": sents, "aliases": t.get("aliases") or []}
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
    for d in jl(os.path.join(art, "chunks.jsonl")):
        sid = d["storyId"]
        for s in d.get("speakers") or []:
            speakers[sid][s] += 1
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
        body = tidy(dumps_compact(panel)).encode("utf-8")
        violations += verify_panel(json.loads(body), ctx)
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
    with open(os.path.join(stage, "stats.json"), "w", encoding="utf-8") as f:
        json.dump(st, f, indent=2, ensure_ascii=False)
    for v in violations[:20]:
        sys.stderr.write(f"VIOLATION {v}\n")
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
def verify_panel(p, ctx):
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
        if c["identitySource"] == "operatorFile" and not (c["name"] in archive and archive[c["name"]][2] <= K):
            bad(f"operator file {c['name']}")
        if c["identitySource"] == "bank" and not (c["name"] in who and who[c["name"]]["_gate"] <= K):
            bad(f"who {c['name']}")
        if c["whoId"] and not by_id[c["whoId"]]["_gate"] <= K:
            bad(f"whoId {c['whoId']}")
    for t in p["terms"]["items"]:
        if t["firstNamed"] and not O.key(t["firstNamed"]["storyId"]) < K:
            bad(f"firstNamed {t['term']}")
        if t["definition"] is not None:
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
    print("panels.py self-test: ok (order, ties, dated and undated records, sourcePos, eras, gate_order round trip)")


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
    ap.add_argument("--max-chars", type=int, default=12)
    ap.add_argument("--max-terms", type=int, default=8)
    ap.add_argument("--max-suggest", type=int, default=6)
    ap.add_argument("--max-sofar", type=int, default=8)
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        self_test()
        return 0
    a.trevor = os.path.abspath(a.trevor)
    a.out = os.path.abspath(a.out or os.path.join(a.trevor, "artifacts", "panels"))
    return build(a)


if __name__ == "__main__":
    sys.exit(main())
