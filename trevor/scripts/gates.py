#!/usr/bin/env python3
"""Publish gates for a Trevor panel build (design/trevor-vps-runbook.md section 4; contract v1 section 3).

Reads a panels.py build (default artifacts/panels: manifest.json + stories/*.json) and the shared gate order
(artifacts/chrono/gate_order.json), runs four gates and writes the gate report publish.py sends as `gateReport`:
  schema    manifest and every panel match panel schema 1; storyId = file name; position = gatePos of the story in
            gate_order.json; storyOrder and eras equal gate_order.json's; panelsSha recomputed.
  spoiler   0 facts past a panel's horizon, from the panel's own content and gatePos alone: previous, soFar,
            firstSeen and firstNamed strictly earlier than the story; every suggestion's gatePos and the era's
            position at or before it. Canary citations are checked the same way (below).
  recall    BM25 recall@10 on the gold set (`target/release/eval run --mode bm25`, 131 scored items, 0.3 s) is at
            least the previous published value minus 0.02. The previous value is the active version's gate report
            (state dir: active.json -> versions/<v>/activate.json), else --previous-recall; with neither, this run
            is the baseline and passes.
  canary    10 fixed questions (deploy/canary.jsonl, each with a horizon story) through `ask --serve` under the model
            lock: every answer non-empty and every cited passage at or before the question's horizon (gatePos for
            stories, the gate order's `sourcePos` for dated sources, anything else is a violation). --no-canary skips
            it (no model on the box, or a dry run); a skipped canary does not fail the report.
The report passes when every gate that ran passes. Exit 0 = passed, 1 = a gate failed, 2 = could not run.

Env: TREVOR_STATE_DIR (default <trevor>/artifacts/publish), TREVOR_LLM_LOCK (default
/private/tmp/claude-501/trevor-llm.lock, ask's), TREVOR_LLM_URL (a running llama-server for the canary; unset = ask
starts its own model), GATES_ASK_FLAGS (extra ask flags for the canary, e.g. "--router keywords --k 5").
"""
import argparse, hashlib, json, os, re, shlex, signal, socket, subprocess, sys, time, urllib.error, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REPORT_SCHEMA = 1
RECALL_TOLERANCE = 0.02

# ---------------------------------------------------------------- schema 1
S, I, B = "str", "int", "bool"
OPT = lambda t: ("opt", t)
ONE_OF = lambda *ts: ("oneof",) + ts
LIST = lambda t: ("list", t)
ENUM = lambda *vs: ("enum",) + vs
REF = {"storyId": S, "group": S}
PANEL = {
    "schema": ENUM(1), "storyId": S, "groupId": S, "group": S, "title": OPT(S), "position": I,
    "era": OPT({"id": S, "name": S, "position": I}),
    "recap": {
        "thisStory": OPT({"title": OPT(S), "synopsis": OPT(S), "summary": OPT(S), "spoiler": B}),
        "previous": OPT({"storyId": S, "title": OPT(S), "summary": OPT(S)}),
        "soFar": LIST({"storyId": S, "title": OPT(S), "synopsis": OPT(S)}),
        "earlierCount": I,
        "beforeYouStart": OPT(ONE_OF({"primer": S, "prior": LIST(S)}, {"bankId": S})),
        "buildsOn": LIST(S),
    },
    "whosHere": {"characters": LIST({"name": S, "chunks": I, "identity": OPT(S),
                                     "identitySource": OPT(ENUM("operatorFile", "bank", "enemyHandbook", "storySynopsis", "storyLine")), "whoId": OPT(S), "debut": B,
                                     "firstSeen": OPT(REF), "seenInGroups": I, "recentGroups": LIST(S)}),
                 "more": I},
    "terms": {"items": LIST({"term": S, "kind": S, "mentions": I, "definition": OPT(S), "firstNamed": OPT(REF)}),
              "more": I},
    "suggestions": {"items": LIST({"id": S, "kind": ENUM("ask", "who", "primer", "story", "event"), "question": S,
                                   "gatePos": I, "fromThisStory": B, "answer?": S})},
}
MANIFEST = {"schema": ENUM(1), "version": S, "server": ENUM("en"), "corpusVersion": OPT(S), "builtFrom": OPT(S),
            "builtAt": S, "storyOrder": LIST(S), "eras": LIST({"id": S, "name": S, "position": I}), "panels": I,
            "panelsSha": S, "gateOrderSha": OPT(S), "answersInline": B, "stats": dict}


def check(v, spec, path, errs):
    """Append to errs every place where v does not match spec (dict keys are exact; 'k?' marks an optional key)."""
    if len(errs) > 200:
        return
    if spec is dict:
        if not isinstance(v, dict):
            errs.append(f"{path}: not an object")
        return
    if isinstance(spec, dict):
        if not isinstance(v, dict):
            errs.append(f"{path}: not an object"); return
        want = {k.rstrip("?"): k.endswith("?") for k in spec}
        for k in v:
            if k not in want:
                errs.append(f"{path}.{k}: unexpected key")
        for k, optional in want.items():
            if k not in v:
                if not optional:
                    errs.append(f"{path}.{k}: missing")
                continue
            check(v[k], spec[k + "?" if optional else k], f"{path}.{k}", errs)
        return
    if isinstance(spec, tuple):
        kind = spec[0]
        if kind == "opt":
            if v is not None:
                check(v, spec[1], path, errs)
        elif kind == "list":
            if not isinstance(v, list):
                errs.append(f"{path}: not a list"); return
            for i, x in enumerate(v):
                check(x, spec[1], f"{path}[{i}]", errs)
        elif kind == "enum":
            if v not in spec[1:] or isinstance(v, bool) != isinstance(spec[1], bool):
                errs.append(f"{path}: {v!r} not in {spec[1:]}")
        elif kind == "oneof":
            if not any(not _errs(v, s) for s in spec[1:]):
                errs.append(f"{path}: matches none of {len(spec) - 1} shapes")
        return
    ok = {S: lambda x: isinstance(x, str), I: lambda x: isinstance(x, int) and not isinstance(x, bool),
          B: lambda x: isinstance(x, bool)}[spec](v)
    if not ok:
        errs.append(f"{path}: expected {spec}, got {type(v).__name__}")


def _errs(v, spec):
    e = []
    check(v, spec, "", e)
    return e


# ---------------------------------------------------------------- loading
def gate_maps(order):
    """storyId -> gatePos (stories in `never` have none) and sourceId -> sourcePos, from gate_order.json (schema 1,
    horizon.rs's GateOrderFile)."""
    order = order or {}
    never = set(order.get("never") or [])
    return ({s: i + 1 for i, s in enumerate(order.get("storyOrder") or []) if s not in never},
            dict(order.get("sourcePos") or {}))


def load_build(panels_dir):
    manifest = json.load(open(os.path.join(panels_dir, "manifest.json"), encoding="utf-8"))
    raw = {}
    for sid in manifest.get("storyOrder") or []:
        p = os.path.join(panels_dir, "stories", sid + ".json")
        if os.path.exists(p):
            raw[sid] = open(p, "rb").read()
    extra = sorted(f[:-5] for f in os.listdir(os.path.join(panels_dir, "stories"))
                   if f.endswith(".json") and f[:-5] not in raw)
    return manifest, raw, extra


def panels_sha(manifest, raw):
    """panels.py's panelsSha: sha256 of {"id":panel,...} in storyOrder, the panel bytes as written."""
    body = b"{" + b",".join(json.dumps(s).encode() + b":" + raw[s] for s in manifest["storyOrder"] if s in raw) + b"}"
    return hashlib.sha256(body).hexdigest()


# ---------------------------------------------------------------- gates
def gate_schema(manifest, raw, extra, order):
    errs = _errs(manifest, MANIFEST)
    so = manifest.get("storyOrder") or []
    if len(set(so)) != len(so):
        errs.append("manifest.storyOrder: duplicate ids")
    if manifest.get("panels") != len(raw):
        errs.append(f"manifest.panels {manifest.get('panels')} != {len(raw)} panel files")
    missing = [s for s in so if s not in raw]
    if missing:
        errs.append(f"{len(missing)} stories in storyOrder have no panel file (first {missing[0]})")
    if extra:
        errs.append(f"{len(extra)} panel files not in storyOrder (first {extra[0]})")
    if order is None:
        errs.append("artifacts/chrono/gate_order.json missing")
    else:
        if order.get("storyOrder") != so:
            errs.append("manifest.storyOrder != gate_order.json storyOrder")
        if [(e["id"], e["position"]) for e in order.get("eras") or []] != [(e.get("id"), e.get("position")) for e in manifest.get("eras") or []]:
            errs.append("manifest.eras != gate_order.json eras")
    pos = [e.get("position") for e in manifest.get("eras") or []]
    if pos != sorted(pos) or any(not isinstance(p, int) or p < 1 or p > len(so) for p in pos):
        errs.append(f"manifest.eras positions not ascending within 1..{len(so)}")
    if raw and not missing and manifest.get("panelsSha") != panels_sha(manifest, raw):
        errs.append("manifest.panelsSha does not match the panel files")
    gp = gate_maps(order)[0]
    bad_panels = 0
    for sid, b in raw.items():
        try:
            p = json.loads(b)
        except ValueError as e:
            errs.append(f"{sid}: not JSON ({e})"); bad_panels += 1; continue
        pe = _errs(p, PANEL)
        if p.get("storyId") != sid:
            pe.append(f"storyId {p.get('storyId')!r} != file name")
        if gp and p.get("position") != gp.get(sid):
            pe.append(f"position {p.get('position')} != gatePos {gp.get(sid)}")
        if pe:
            bad_panels += 1
            errs += [f"{sid}{e}" if e.startswith(".") else f"{sid}: {e}" for e in pe[:5]]
    return {"pass": not errs, "panels": len(raw), "badPanels": bad_panels, "errors": errs[:30], "errorCount": len(errs)}


def panel_violations(p, gp):
    """Facts of one panel past its horizon, from the panel and the gate order alone."""
    sid, here = p["storyId"], p["position"]
    out = []

    def earlier(ref, what):
        q = gp.get(ref)
        if q is None or not q < here:
            out.append(f"{sid}: {what} {ref} at {q} is not before {here}")
    r = p["recap"]
    if r.get("previous"):
        earlier(r["previous"]["storyId"], "recap.previous")
    for e in r.get("soFar") or []:
        earlier(e["storyId"], "recap.soFar")
    for c in p["whosHere"]["characters"]:
        if c.get("firstSeen"):
            earlier(c["firstSeen"]["storyId"], f"firstSeen({c['name']})")
    for t in p["terms"]["items"]:
        if t.get("firstNamed"):
            earlier(t["firstNamed"]["storyId"], f"firstNamed({t['term']})")
    for s in p["suggestions"]["items"]:
        if not s["gatePos"] <= here:
            out.append(f"{sid}: suggestion {s['id']} gatePos {s['gatePos']} > {here}")
    if p.get("era") and not p["era"]["position"] <= here:
        out.append(f"{sid}: era {p['era']['id']} at {p['era']['position']} > {here}")
    return out


def gate_spoiler(raw, order):
    gp = gate_maps(order)[0]
    v, checked = [], 0
    for sid, b in raw.items():
        try:
            p = json.loads(b)
            v += panel_violations(p, gp)
            checked += 1
        except (ValueError, KeyError, TypeError) as e:
            v.append(f"{sid}: unreadable for the spoiler check ({e})")
    return {"pass": not v and bool(gp), "checked": checked, "violations": len(v), "examples": v[:20]}


def previous_recall(state, goldset, corpus):
    try:
        act = json.load(open(os.path.join(state, "active.json")))
        rep = json.load(open(os.path.join(state, "versions", act["version"], "activate.json")))["gateReport"]
        r = rep["gates"]["recall"]
        if r.get("goldset") == goldset and r.get("corpus") == corpus and isinstance(r.get("value"), (int, float)):
            return r["value"], act["version"]
    except (OSError, KeyError, ValueError, TypeError):
        pass
    return None, None


def gate_recall(a, version):
    out_dir = os.path.join(a.state, "gates", "eval")
    os.makedirs(out_dir, exist_ok=True)
    log = os.path.join(out_dir, "last.log")
    cmd = [a.eval_bin, "run", "--goldset", a.goldset, "--corpus", a.corpus, "--mode", "bm25",
           "--label", f"gate-{version}", "--out", out_dir]
    with open(log, "w") as f:
        rc = subprocess.call(cmd, cwd=a.trevor, stdout=f, stderr=subprocess.STDOUT)
    res = {"goldset": a.goldset, "corpus": a.corpus, "mode": "bm25", "metric": "recall@10", "exit": rc}
    if rc != 0:
        res.update({"pass": False, "error": f"eval exited {rc}; see {log}"})
        return res
    last = None
    for line in open(os.path.join(out_dir, "runs.jsonl")):
        if line.strip():
            last = json.loads(line)
    value = last["metrics"]["recall@10"]
    prev, prev_version = previous_recall(a.state, a.goldset, a.corpus)
    if prev is None and a.previous_recall is not None:
        prev, prev_version = a.previous_recall, "--previous-recall"
    res.update({"value": round(value, 6), "nScored": last.get("nScored"), "runId": last.get("runId"),
                "previous": prev, "previousFrom": prev_version, "tolerance": RECALL_TOLERANCE,
                "pass": prev is None or value >= prev - RECALL_TOLERANCE - 1e-12})
    if prev is None:
        res["note"] = "no previous published value: this run is the baseline"
    return res


# ---------------------------------------------------------------- the model lock (ask's convention)
def pid_alive(pid):
    try:
        os.kill(pid, 0)
        return True
    except ProcessLookupError:
        return False
    except PermissionError:
        return True


def ancestors():
    """This process and its ancestors' pids (ask's rule: a lock held by an ancestor is ours)."""
    out, p = [], os.getpid()
    for _ in range(64):
        out.append(p)
        try:
            pp = int(subprocess.run(["ps", "-o", "ppid=", "-p", str(p)], capture_output=True, text=True).stdout.strip())
        except (OSError, ValueError):
            break
        if pp <= 1 or pp == p:
            break
        p = pp
    return out


def take_lock(path, label, wait_s):
    """ask's model lock: a file holding "<pid> <label>". A holder that is this process or one of its ancestors (the
    script that started this one) is ours, and `ask` children run under it the same way. A lock whose pid is dead is
    taken over. Waits up to wait_s seconds for a live holder. Returns True when taken or already ours."""
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    deadline = time.time() + wait_s
    while True:
        try:
            fd = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o644)
            os.write(fd, f"{os.getpid()} {label}\n".encode())
            os.close(fd)
            return True
        except FileExistsError:
            try:
                pid = int(open(path).read().split()[0])
            except (OSError, ValueError, IndexError):
                pid = None
            if pid is not None and pid in ancestors():
                return True
            if pid is None or not pid_alive(pid):
                try:
                    os.remove(path)
                except FileNotFoundError:
                    pass
                continue
            if time.time() >= deadline:
                return False
            time.sleep(10)


def release_lock(path):
    try:
        if open(path).read().split()[0] == str(os.getpid()):
            os.remove(path)
    except (OSError, IndexError):
        pass


def mac_unsafe():
    """On the Mac (house rule): no model run with free memory under 25% or a recorded thermal warning."""
    if sys.platform != "darwin":
        return None
    try:
        free = subprocess.run(["memory_pressure"], capture_output=True, text=True, timeout=60).stdout.strip().splitlines()[-1]
        pct = int(re.search(r"(\d+)%", free).group(1))
        therm = subprocess.run(["pmset", "-g", "therm"], capture_output=True, text=True, timeout=30).stdout
    except (OSError, AttributeError, ValueError, IndexError, subprocess.TimeoutExpired) as e:
        return f"could not read memory/thermal state ({e})"
    if pct < 25:
        return f"free memory {pct}% < 25%"
    if re.search(r"warning level", therm, re.I) and not re.search(r"No thermal warning level", therm):
        return "thermal warning recorded"
    return None


# ---------------------------------------------------------------- canary
def http_json(url, body=None, timeout=60):
    data = None if body is None else json.dumps(body).encode()
    req = urllib.request.Request(url, data, {"content-type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return r.status, json.load(r)
    except urllib.error.HTTPError as e:
        try:
            return e.code, json.load(e)
        except ValueError:
            return e.code, {"error": str(e)}


def free_port():
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    p = s.getsockname()[1]
    s.close()
    return p


def citation_violations(res, horizon_pos, order):
    gp, src = gate_maps(order)
    out = []
    for c in res.get("citations") or []:
        sid = c.get("storyId")
        q = gp.get(sid, src.get(sid)) if sid else None
        if q is None or q > horizon_pos:
            out.append(f"cites {sid or c.get('chunkId')} at {q} past horizon {horizon_pos}")
    return out


def ask_one(base, q, sid, pos, mode):
    """POST one canary question; the horizon as contract v1's horizonPos, or as the story id when this `ask` predates
    it (serve.rs refuses unknown fields with a 400). Returns (status, body, mode used)."""
    for m in ([mode] if mode else ["pos", "story"]):
        body = {"question": q, "horizonPos": pos} if m == "pos" else {"question": q, "horizon": sid}
        code, res = http_json(f"{base}/v1/ask", body, timeout=1800)
        if code == 400 and m == "pos" and "unknown field" in json.dumps(res):
            continue
        return code, res, m
    return code, res, mode


def gate_canary(a, manifest, order):
    if a.no_canary:
        return {"skipped": True, "reason": "--no-canary", "pass": None}
    why = mac_unsafe()
    if why:
        return {"skipped": True, "reason": f"refused on this Mac: {why}", "pass": None}
    qs = [json.loads(l) for l in open(a.canary, encoding="utf-8") if l.strip()]
    gp = gate_maps(order)[0]
    lock = os.environ.get("TREVOR_LLM_LOCK", "/private/tmp/claude-501/trevor-llm.lock")
    if not take_lock(lock, "gates-canary", a.lock_wait):
        return {"pass": False, "error": f"model lock {lock} held by another job for {a.lock_wait} s"}
    port = free_port()
    cmd = [a.ask_bin, "--serve", "--port", str(port), "--lock", lock]
    if os.environ.get("TREVOR_LLM_URL"):
        cmd += ["--server", os.environ["TREVOR_LLM_URL"], "--no-spawn"]
    cmd += shlex.split(os.environ.get("GATES_ASK_FLAGS", ""))
    log = open(os.path.join(a.state, "gates", "canary-ask.log"), "w")
    proc = subprocess.Popen(cmd, cwd=a.trevor, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    old = {s: signal.getsignal(s) for s in (signal.SIGTERM, signal.SIGINT)}

    def stop(*_):
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(60)
            except subprocess.TimeoutExpired:
                proc.kill(); proc.wait()
        release_lock(lock)
    for s in old:
        signal.signal(s, lambda *_: (stop(), sys.exit(130)))
    rows, mode, health = [], None, None
    try:
        base = f"http://127.0.0.1:{port}"
        for _ in range(900):
            if proc.poll() is not None:
                return {"pass": False, "error": f"ask --serve exited {proc.returncode} before it was ready; see {log.name}"}
            try:
                code, health = http_json(f"{base}/health", timeout=5)
                if code == 200:
                    break
            except (urllib.error.URLError, OSError):
                pass
            time.sleep(2)
        else:
            return {"pass": False, "error": "ask --serve not ready in 1800 s"}
        for q in qs:
            pos = gp.get(q["horizonStory"])
            t = time.time()
            code, res, mode = ask_one(base, q["question"], q["horizonStory"], pos, mode)
            ans = (res.get("answer") or "").strip() if code == 200 else ""
            v = citation_violations(res, pos, order) if code == 200 and pos is not None else ["unknown horizon story"]
            rows.append({"id": q["id"], "status": code, "horizonPos": pos, "nonEmpty": bool(ans),
                         "citations": len(res.get("citations") or []) if code == 200 else 0, "violations": v,
                         "seconds": round(time.time() - t, 1), "error": None if code == 200 else res.get("error")})
    finally:
        stop()
        for s, h in old.items():
            signal.signal(s, h)
    viol = sum(len(r["violations"]) for r in rows)
    empty = sum(1 for r in rows if not r["nonEmpty"])
    cv = (health or {}).get("corpusVersion")
    res = {"pass": bool(rows) and viol == 0 and empty == 0 and (cv == manifest.get("corpusVersion")),
           "n": len(rows), "nonEmpty": len(rows) - empty, "violations": viol, "horizonField": mode,
           "model": (health or {}).get("model"), "corpusVersion": cv, "rows": rows}
    if cv != manifest.get("corpusVersion"):
        res["error"] = f"ask serves corpus {cv}, the manifest says {manifest.get('corpusVersion')}"
    return res


# ---------------------------------------------------------------- main
def run(a):
    manifest, raw, extra = load_build(a.panels)
    gpath = os.path.join(a.trevor, "artifacts", "chrono", "gate_order.json")
    order = json.load(open(gpath, encoding="utf-8")) if os.path.exists(gpath) else None
    t0 = time.time()
    gates = {"schema": gate_schema(manifest, raw, extra, order), "spoiler": gate_spoiler(raw, order)}
    gates["recall"] = gate_recall(a, manifest.get("version")) if not a.no_recall else {"skipped": True, "pass": None}
    gates["canary"] = gate_canary(a, manifest, order or {})
    ran = [g for g in gates.values() if not g.get("skipped")]
    report = {"schema": REPORT_SCHEMA, "version": manifest.get("version"), "corpusVersion": manifest.get("corpusVersion"),
              "builtFrom": manifest.get("builtFrom"), "panelsSha": manifest.get("panelsSha"),
              "gateOrderSha": manifest.get("gateOrderSha"),
              "at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()), "seconds": round(time.time() - t0, 1),
              "passed": all(g.get("pass") is True for g in ran), "gates": gates}
    out = a.out or os.path.join(a.panels, "gate_report.json")
    tmp = out + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(report, f, ensure_ascii=False, indent=1)
    os.replace(tmp, out)
    summary = {k: ("skipped" if g.get("skipped") else ("pass" if g.get("pass") else "FAIL")) for k, g in gates.items()}
    print(json.dumps({"passed": report["passed"], "gates": summary,
                      "spoilerViolations": gates["spoiler"]["violations"],
                      "recall": gates["recall"].get("value"), "previousRecall": gates["recall"].get("previous"),
                      "schemaErrors": gates["schema"]["errorCount"], "report": out}))
    return 0 if report["passed"] else 1


def self_test():
    """The schema and spoiler gates on a real-shaped panel, clean and with injected faults."""
    p = {"schema": 1, "storyId": "s3", "groupId": "g", "group": "G", "title": "T", "position": 3,
         "era": {"id": "main_0", "name": "E", "position": 2},
         "recap": {"thisStory": {"title": "T", "synopsis": None, "summary": "x", "spoiler": True},
                   "previous": {"storyId": "s2", "title": "t", "summary": "y"},
                   "soFar": [{"storyId": "s1", "title": "t", "synopsis": "z"}], "earlierCount": 2,
                   "beforeYouStart": {"bankId": "primer:g"}, "buildsOn": []},
         "whosHere": {"characters": [{"name": "A", "chunks": 2, "identity": None, "identitySource": None, "whoId": None,
                                      "debut": False, "firstSeen": {"storyId": "s1", "group": "G"}, "seenInGroups": 0,
                                      "recentGroups": []}], "more": 0},
         "terms": {"items": [{"term": "Ursus", "kind": "nation", "mentions": 1, "definition": None,
                              "firstNamed": {"storyId": "s2", "group": "G"}}], "more": 0},
         "suggestions": {"items": [{"id": "ask:b1", "kind": "ask", "question": "q", "gatePos": 3, "fromThisStory": True,
                                    "answer": "a"}]}}
    gp = {"s1": 1, "s2": 2, "s3": 3, "s4": 4}
    assert _errs(p, PANEL) == [], _errs(p, PANEL)
    assert panel_violations(p, gp) == []
    import copy
    cases = [
        (lambda q: q["recap"]["previous"].update(storyId="s4"), "recap.previous"),
        (lambda q: q["recap"]["soFar"][0].update(storyId="s3"), "recap.soFar"),
        (lambda q: q["whosHere"]["characters"][0]["firstSeen"].update(storyId="nope"), "firstSeen"),
        (lambda q: q["terms"]["items"][0]["firstNamed"].update(storyId="s4"), "firstNamed"),
        (lambda q: q["suggestions"]["items"][0].update(gatePos=4), "suggestion"),
        (lambda q: q["era"].update(position=4), "era"),
    ]
    for mut, what in cases:
        q = copy.deepcopy(p); mut(q)
        v = panel_violations(q, gp)
        assert len(v) == 1 and what in v[0], (what, v)
    for mut, what in [(lambda q: q.pop("terms"), "missing"), (lambda q: q.update(position="3"), "expected int"),
                      (lambda q: q["suggestions"]["items"][0].update(kind="faq"), "not in"),
                      (lambda q: q.update(extra=1), "unexpected key"),
                      (lambda q: q["recap"].update(beforeYouStart={"bankId": 3}), "matches none")]:
        q = copy.deepcopy(p); mut(q)
        e = _errs(q, PANEL)
        assert e and what in e[0], (what, e)
    q = copy.deepcopy(p); q["suggestions"]["items"][0].pop("answer")
    assert _errs(q, PANEL) == []  # answer is optional (--no-answers)
    order = {"storyOrder": ["s1", "s2", "s3", "s4"], "never": [], "sourcePos": {"archive_x": 2}}
    assert gate_maps(order)[0] == gp
    assert citation_violations({"citations": [{"storyId": "s2"}, {"storyId": "archive_x"}]}, 2, order) == []
    assert len(citation_violations({"citations": [{"storyId": "s3"}, {"chunkId": "topic:x"}, {"storyId": "archive_x"}]},
                                   1, order)) == 3
    print("gates.py self-test: ok (schema accepts the panel, 5 schema faults and 6 spoiler faults each caught once, "
          "citation check)")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--trevor", default=ROOT)
    ap.add_argument("--panels", default=None, help="panels.py output (default <trevor>/artifacts/panels)")
    ap.add_argument("--out", default=None, help="gate report path (default <panels>/gate_report.json)")
    ap.add_argument("--state", default=None, help="publish state dir (default $TREVOR_STATE_DIR or <trevor>/artifacts/publish)")
    ap.add_argument("--goldset", default="eval/goldset_v2.jsonl")
    ap.add_argument("--corpus", default="artifacts/p3b", help="the corpus the worker serves from")
    ap.add_argument("--previous-recall", type=float, default=None, help="used when no version is active yet")
    ap.add_argument("--no-recall", action="store_true", help="skip the recall gate (testing only)")
    ap.add_argument("--no-canary", action="store_true", help="skip the model canary (no model on this box)")
    ap.add_argument("--canary", default=None, help="questions (default <trevor>/deploy/canary.jsonl)")
    ap.add_argument("--lock-wait", type=int, default=3600, help="seconds to wait for the model lock")
    ap.add_argument("--ask-bin", default=None, help="the ask binary (default <trevor>/target/release/ask)")
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        self_test()
        return 0
    a.trevor = os.path.abspath(a.trevor)
    a.panels = os.path.abspath(a.panels or os.path.join(a.trevor, "artifacts", "panels"))
    a.state = os.path.abspath(a.state or os.environ.get("TREVOR_STATE_DIR") or os.path.join(a.trevor, "artifacts", "publish"))
    a.canary = a.canary or os.path.join(a.trevor, "deploy", "canary.jsonl")
    a.eval_bin = os.path.join(a.trevor, "target", "release", "eval")
    a.ask_bin = a.ask_bin or os.path.join(a.trevor, "target", "release", "ask")
    os.makedirs(os.path.join(a.state, "gates"), exist_ok=True)
    try:
        return run(a)
    except (OSError, ValueError, KeyError) as e:
        sys.stderr.write(f"gates: could not run: {e}\n")
        return 2


if __name__ == "__main__":
    sys.exit(main())
