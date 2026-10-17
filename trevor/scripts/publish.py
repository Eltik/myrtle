#!/usr/bin/env python3
"""Publish a gated panel build to the myrtle backend (contract v1 section 3, `POST /api/trevor/versions`).

Reads a panels.py build (default artifacts/panels: manifest.json, stories/*.json, gate_report.json) and sends:
  1. the panels in chunks, each `{version, server, builtAt, manifest, gateReport, panels: {...}, activate: false}`,
     gzip-encoded (`content-encoding: gzip`), every request at most 4 MiB compressed AND 8 MiB decompressed;
  2. one `{version, server, builtAt, manifest, gateReport, panels: {}, activate: true}`: each call overwrites the
     version row's manifest and gate report (ON CONFLICT DO UPDATE), so the activate call resends both.
Each chunk is one transaction on the backend and the flip to active is one more, so a publish that dies part way
leaves an inactive version and the site keeps the last good one. Panel bytes are sent exactly as panels.py wrote them.

Every request carries `x-trevor-worker-key: $TREVOR_WORKER_KEY` and is retried 3 times after the first try (1, 4,
16 s) on a network error, 408, 429 or 5xx; any other 4xx fails at once. A build whose gate report did not pass, or
whose report names another version or panels sha, is refused.

After an activate the state dir ($TREVOR_STATE_DIR, default artifacts/publish) holds:
  active.json                      {version, builtFrom, corpusVersion, activatedAt}: what vps-update.sh compares
  versions/<version>/activate.json the exact activate body (manifest + gate report): what --activate VERSION resends
  history.jsonl                    one line per publish or rollback
Rollback: `publish.py --activate <older version>` resends that version's stored activate body (its panels are still
in trevor_panels: the backend never deletes a version).

Usage:
  publish.py [--panels DIR] [--backend URL] [--dry-run OUTDIR] [--no-gzip] [--no-activate]
  publish.py --activate VERSION [--dry-run OUTDIR]
--no-activate sends the chunks only and stores the activate body (versions/<version>/activate.json) without
touching active.json; `--activate VERSION` then flips it. vps-update.sh uses the pair so the worker restarts on the
new corpus between the two (the backend's active version and the worker's corpus then differ for seconds, not for
the whole upload).
  publish.py --self-test
Env: TREVOR_WORKER_KEY (required unless --dry-run), TREVOR_BACKEND (default http://127.0.0.1:3060),
TREVOR_STATE_DIR.
"""
import argparse, gzip, hashlib, json, os, sys, time, urllib.error, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PATH = "/api/trevor/versions"
MAX_COMPRESSED = 4 * 1024 * 1024
MAX_DECOMPRESSED = 8 * 1024 * 1024
BACKOFF = (1, 4, 16)


# ---------------------------------------------------------------- packing
def envelope(head, panels_items, activate):
    """The request body bytes: `head` (version, server, builtAt, manifest, gateReport as JSON fragments) then the
    panels object built from (id, raw panel bytes) without re-serializing the panels."""
    inner = b",".join(json.dumps(sid).encode() + b":" + body for sid, body in panels_items)
    return (b"{" + head + b',"activate":' + (b"true" if activate else b"false") + b',"panels":{' + inner + b"}}")


def head_bytes(version, server, built_at, manifest, gate_report):
    j = lambda x: json.dumps(x, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    return (b'"version":' + j(version) + b',"server":' + j(server) + b',"builtAt":' + j(built_at)
            + b',"manifest":' + j(manifest) + b',"gateReport":' + j(gate_report))


# Chunk shas for the backend's cache carry-over (contract v1.1 section 3b, backend clarification): every chunk id the
# worker can cite, with the `contentSha` its citations carry, sent in calls of their own before the activate call.
CHUNK_SHA_BATCH = 8000
CHUNK_SHA_CORPORA = "artifacts/p3b,artifacts/p4x,artifacts/p4-art"


def chunk_sha_map(root, corpora):
    """{chunkId: contentSha} over the corpora the worker serves (first corpus wins; a differing sha for the same id is
    counted, not sent twice). A chunk without contentSha gets sha256(text)[:16], the worker's fallback. Missing corpus
    directories are skipped and named."""
    shas, conflicts, missing = {}, 0, []
    for c in [x.strip() for x in corpora.split(",") if x.strip()]:
        path = os.path.join(root, c, "chunks.jsonl")
        if not os.path.exists(path):
            missing.append(c)
            continue
        for line in open(path, encoding="utf-8"):
            r = json.loads(line)
            sha = r.get("contentSha") or hashlib.sha256(r["text"].encode("utf-8")).hexdigest()[:16]
            old = shas.setdefault(r["chunkId"], sha)
            conflicts += old != sha
    return shas, conflicts, missing


def sha_bodies(head, shas, batch=CHUNK_SHA_BATCH):
    """Request bodies carrying the sha map in sorted batches: `{head, activate: false, panels: {}, chunkShas: {...}}`."""
    ids = sorted(shas)
    for i in range(0, len(ids), batch):
        part = {k: shas[k] for k in ids[i:i + batch]}
        yield (b"{" + head + b',"activate":false,"panels":{},"chunkShas":'
               + json.dumps(part, separators=(",", ":")).encode("utf-8") + b"}")


def encode(body, use_gzip):
    return gzip.compress(body, 6, mtime=0) if use_gzip else body


def pack(head, items, max_compressed=MAX_COMPRESSED, max_decompressed=MAX_DECOMPRESSED, use_gzip=True):
    """Greedy chunks of panels, in order, each body within max_decompressed raw bytes and max_compressed bytes on the
    wire (without gzip the wire bytes are the raw bytes, so both limits apply to them). A chunk whose compressed form
    is still too big is split in half until it fits. Returns [(items, raw body, wire bytes)]. Raises ValueError when
    one panel alone cannot fit."""
    base = len(envelope(head, [], False))
    raw_cap = max_decompressed if use_gzip else min(max_decompressed, max_compressed)
    groups, cur, size = [], [], base
    for it in items:
        add = len(json.dumps(it[0]).encode()) + 1 + len(it[1]) + (1 if cur else 0)
        if cur and size + add > raw_cap:
            groups.append(cur); cur, size = [], base
            add -= 1
        cur.append(it); size += add
    if cur:
        groups.append(cur)
    out = []
    stack = list(reversed(groups))
    while stack:
        g = stack.pop()
        body = envelope(head, g, False)
        wire = encode(body, use_gzip)
        if len(body) <= raw_cap and len(wire) <= max_compressed:
            out.append((g, body, wire))
        elif len(g) == 1:
            raise ValueError(f"panel {g[0][0]} alone is {len(body)} raw / {len(wire)} wire bytes, over the limits")
        else:
            mid = len(g) // 2
            stack += [g[mid:], g[:mid]]
    return out


# ---------------------------------------------------------------- sending
class SendError(Exception):
    pass


def post(url, wire, headers, tries=len(BACKOFF) + 1, sleep=time.sleep, timeout=300):
    """POST with the retry schedule; returns the decoded JSON answer."""
    err = None
    for attempt in range(tries):
        req = urllib.request.Request(url, wire, headers, method="POST")
        try:
            with urllib.request.urlopen(req, timeout=timeout) as r:
                txt = r.read()
                return json.loads(txt) if txt.strip() else {}
        except urllib.error.HTTPError as e:
            detail = e.read()[:400].decode("utf-8", "replace")
            err = SendError(f"HTTP {e.code} from {url}: {detail}")
            if e.code < 500 and e.code not in (408, 429):
                raise err
        except (urllib.error.URLError, OSError, ValueError) as e:
            err = SendError(f"{type(e).__name__} from {url}: {e}")
        if attempt < tries - 1:
            sleep(BACKOFF[min(attempt, len(BACKOFF) - 1)])
    raise err


def headers_for(key, use_gzip):
    h = {"content-type": "application/json", "x-trevor-worker-key": key}
    if use_gzip:
        h["content-encoding"] = "gzip"
    return h


def write_dry(out_dir, requests):
    os.makedirs(out_dir, exist_ok=True)
    index = []
    for i, r in enumerate(requests):
        name = f"{i:03d}-{'activate' if r['activate'] else 'chunk'}.json" + (".gz" if r["gzip"] else "")
        with open(os.path.join(out_dir, name), "wb") as f:
            f.write(r["wire"])
        hdr = dict(r["headers"], **{"x-trevor-worker-key": "<redacted>"})
        index.append({"file": name, "method": "POST", "url": r["url"], "headers": hdr, "activate": r["activate"],
                      "panels": r["panels"], "wireBytes": len(r["wire"]), "decompressedBytes": len(r["body"])})
    with open(os.path.join(out_dir, "requests.json"), "w") as f:
        json.dump(index, f, indent=1)
    return index


def store_activate(state, version, activate_body):
    vdir = os.path.join(state, "versions", version)
    os.makedirs(vdir, exist_ok=True)
    p = os.path.join(vdir, "activate.json")
    with open(p + ".tmp", "wb") as f:
        f.write(activate_body)
    os.replace(p + ".tmp", p)


def record(state, version, activate_body, manifest, how):
    store_activate(state, version, activate_body)
    act = {"version": version, "builtFrom": manifest.get("builtFrom"), "corpusVersion": manifest.get("corpusVersion"),
           "activatedAt": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
    with open(os.path.join(state, "active.json.tmp"), "w") as f:
        json.dump(act, f, indent=1)
    os.replace(os.path.join(state, "active.json.tmp"), os.path.join(state, "active.json"))
    with open(os.path.join(state, "history.jsonl"), "a") as f:
        f.write(json.dumps(dict(act, how=how)) + "\n")


# ---------------------------------------------------------------- commands
def publish(a):
    manifest = json.load(open(os.path.join(a.panels, "manifest.json"), encoding="utf-8"))
    report_path = a.gate_report or os.path.join(a.panels, "gate_report.json")
    if not os.path.exists(report_path):
        sys.exit(f"publish: no gate report at {report_path}; run scripts/gates.py first")
    report = json.load(open(report_path, encoding="utf-8"))
    if report.get("passed") is not True:
        sys.exit(f"publish: the gate report did not pass ({report_path}); nothing sent")
    if report.get("version") != manifest["version"] or report.get("panelsSha") != manifest["panelsSha"]:
        sys.exit("publish: the gate report is for another build (version or panelsSha differ); rerun gates.py")
    items = []
    for sid in manifest["storyOrder"]:
        with open(os.path.join(a.panels, "stories", sid + ".json"), "rb") as f:
            items.append((sid, f.read()))
    raw_all = b"{" + b",".join(json.dumps(s).encode() + b":" + b for s, b in items) + b"}"
    if hashlib.sha256(raw_all).hexdigest() != manifest["panelsSha"]:
        sys.exit("publish: panel files do not match manifest.panelsSha; rebuild with panels.py")
    if not manifest.get("builtFrom"):
        print("publish: WARNING manifest.builtFrom is empty; vps-update.sh cannot detect the next asset update from it",
              file=sys.stderr)
    head = head_bytes(manifest["version"], manifest["server"], manifest["builtAt"], manifest, report)
    use_gzip = not a.no_gzip
    chunks = pack(head, items, a.max_compressed, a.max_decompressed, use_gzip)
    url = a.backend.rstrip("/") + PATH
    hdr = headers_for(a.key or "", use_gzip)
    reqs = [{"url": url, "headers": hdr, "activate": False, "panels": len(g), "body": body, "wire": wire,
             "gzip": use_gzip} for g, body, wire in chunks]
    n_shas = 0
    if not a.no_chunk_shas:
        shas, conflicts, missing = chunk_sha_map(a.trevor, a.chunk_sha_corpora)
        n_shas = len(shas)
        if missing:
            print(f"publish: chunk shas: no chunks.jsonl in {', '.join(missing)} (skipped)", file=sys.stderr)
        if conflicts:
            print(f"publish: chunk shas: {conflicts} ids carry different shas in different corpora; the first is sent",
                  file=sys.stderr)
        for body in sha_bodies(head, shas):
            reqs.append({"url": url, "headers": hdr, "activate": False, "panels": 0, "shas": True, "body": body,
                         "wire": encode(body, use_gzip), "gzip": use_gzip})
    act_body = envelope(head, [], True)
    reqs.append({"url": url, "headers": hdr, "activate": True, "panels": 0, "body": act_body,
                 "wire": encode(act_body, use_gzip), "gzip": use_gzip})
    for r in reqs:  # the limits hold for the activate call too
        assert len(r["wire"]) <= a.max_compressed and len(r["body"]) <= a.max_decompressed, "a request is over the limits"
    total = sum(r["panels"] for r in reqs)
    assert total == len(items), (total, len(items))
    summary = (f"{manifest['version']}: {len(items)} panels in {len(chunks)} chunks, {n_shas} chunk shas in "
               f"{sum(1 for r in reqs if r.get('shas'))} calls + 1 activate; wire "
               f"{sum(len(r['wire']) for r in reqs)} B (max {max(len(r['wire']) for r in reqs)}), decompressed "
               f"{sum(len(r['body']) for r in reqs)} B (max {max(len(r['body']) for r in reqs)})")
    if a.dry_run:
        write_dry(a.dry_run, reqs)
        print(f"dry run: {summary}; requests written to {a.dry_run}")
        return 0
    if not a.key:
        sys.exit("publish: TREVOR_WORKER_KEY is not set")
    t0 = time.time()
    if a.no_activate:
        reqs = reqs[:-1]
    for i, r in enumerate(reqs):
        res = post(r["url"], r["wire"], r["headers"])
        if r["activate"] and res.get("active") is not True:
            raise SendError(f"activate call answered {res}")
        if r.get("shas") and res.get("chunkShas") not in (None, len(json.loads(r["body"])["chunkShas"])):
            raise SendError(f"chunk sha call {i} answered chunkShas={res.get('chunkShas')}")
        if not r["activate"] and res.get("panels") not in (None, r["panels"]):
            raise SendError(f"chunk {i} answered panels={res.get('panels')}, sent {r['panels']}")
    if a.no_activate:
        store_activate(a.state, manifest["version"], act_body)
        print(f"uploaded {summary} in {time.time() - t0:.1f} s, NOT activated; `publish.py --activate "
              f"{manifest['version']}` flips it")
        return 0
    record(a.state, manifest["version"], act_body, manifest, "publish")
    print(f"published {summary} in {time.time() - t0:.1f} s; active = {manifest['version']}")
    return 0


def activate_only(a):
    p = os.path.join(a.state, "versions", a.activate, "activate.json")
    if not os.path.exists(p):
        sys.exit(f"publish: no stored activate body for {a.activate} ({p}); known: "
                 + ", ".join(sorted(os.listdir(os.path.join(a.state, "versions"))) if os.path.isdir(os.path.join(a.state, "versions")) else "none"))
    body = open(p, "rb").read()
    d = json.loads(body)
    if d.get("activate") is not True or d.get("panels") != {} or d.get("version") != a.activate:
        sys.exit(f"publish: {p} is not an activate body for {a.activate}")
    use_gzip = not a.no_gzip
    url = a.backend.rstrip("/") + PATH
    r = {"url": url, "headers": headers_for(a.key or "", use_gzip), "activate": True, "panels": 0, "body": body,
         "wire": encode(body, use_gzip), "gzip": use_gzip}
    if a.dry_run:
        write_dry(a.dry_run, [r])
        print(f"dry run: activate {a.activate} written to {a.dry_run}")
        return 0
    if not a.key:
        sys.exit("publish: TREVOR_WORKER_KEY is not set")
    res = post(r["url"], r["wire"], r["headers"])
    if res.get("active") is not True:
        raise SendError(f"activate call answered {res}")
    record(a.state, a.activate, body, d["manifest"] or {}, "rollback")
    print(f"active = {a.activate}")
    return 0


# ---------------------------------------------------------------- self-test
def self_test():
    """Chunk limits on synthetic panels (incompressible and compressible), order and coverage, the retry schedule and
    the no-retry 4xx against a local HTTP server, and the activate body."""
    import http.server, random, threading
    rnd = random.Random(7)
    head = head_bytes("v1", "en", "2026-10-16T00:00:00Z", {"version": "v1", "storyOrder": []}, {"passed": True})
    # 1. Incompressible panels: the compressed limit binds (gzip cannot shrink random hex much below raw/2).
    noisy = [(f"s{i:04d}", json.dumps({"x": "%x" % rnd.getrandbits(4 * rnd.randint(640, 1920))}).encode())
             for i in range(400)]
    for mc, md in ((40_000, 400_000), (100_000, 120_000), (30_000, 1_000_000)):
        ch = pack(head, noisy, mc, md, True)
        flat = [it for g, _, _ in ch for it in g]
        assert flat == noisy, "order or coverage broken"
        for g, body, wire in ch:
            assert len(wire) <= mc and len(body) <= md, (len(wire), len(body), mc, md)
            assert gzip.decompress(wire) == body
            assert json.loads(body)["panels"].keys() == {s for s, _ in g}
        # greedy: no two neighbouring chunks could have been merged under the raw limit
        print(f"  incompressible, limits {mc}/{md}: {len(ch)} chunks, max wire {max(len(w) for _, _, w in ch)}, "
              f"max raw {max(len(b) for _, b, _ in ch)}")
    # 2. Compressible panels: the decompressed limit binds.
    plain = [(f"p{i:04d}", json.dumps({"text": "Amiya and the Doctor " * 200}).encode()) for i in range(300)]
    ch = pack(head, plain, 4_000_000, 300_000, True)
    assert all(len(b) <= 300_000 and len(w) <= 4_000_000 for _, b, w in ch) and len(ch) >= 4
    assert sum(len(g) for g, _, _ in ch) == 300
    print(f"  compressible, limits 4000000/300000: {len(ch)} chunks, max raw {max(len(b) for _, b, _ in ch)}, "
          f"max wire {max(len(w) for _, _, w in ch)}")
    # 3. --no-gzip: the wire is the raw body, both limits on it.
    ch = pack(head, plain, 200_000, 300_000, False)
    assert all(len(w) <= 200_000 and w == b for _, b, w in ch)
    # 4. One panel over the limit is an error, not a silent drop.
    try:
        pack(head, [("big", b'"' + b"x" * 5000 + b'"')], 1000, 2000, True)
        raise AssertionError("an oversized panel was accepted")
    except ValueError:
        pass
    # 5. HTTP: 503 twice then 200 (retried after 1 and 4 s, sleeps recorded, not slept); 400 is not retried.
    seen = []

    class H(http.server.BaseHTTPRequestHandler):
        def do_POST(self):
            n = int(self.headers["content-length"])
            data = self.rfile.read(n)
            seen.append((self.path, {k.lower(): v for k, v in self.headers.items()}, data))
            code = 503 if len(seen) <= 2 else (400 if self.path.endswith("bad") else 200)
            body = json.dumps({"version": "v1", "server": "en", "panels": 1, "active": True}).encode()
            self.send_response(code); self.send_header("content-length", str(len(body))); self.end_headers()
            self.wfile.write(body)

        def log_message(self, *a):
            pass
    srv = http.server.HTTPServer(("127.0.0.1", 0), H)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    url = f"http://127.0.0.1:{srv.server_address[1]}{PATH}"
    slept = []
    body = envelope(head, [], True)
    res = post(url, encode(body, True), headers_for("k123", True), sleep=slept.append)
    assert res["active"] is True and slept == [1, 4] and len(seen) == 3, (slept, len(seen))
    h = seen[-1][1]
    assert h["x-trevor-worker-key"] == "k123" and h["content-encoding"] == "gzip"
    sent = json.loads(gzip.decompress(seen[-1][2]))
    assert sent["activate"] is True and sent["panels"] == {} and sent["manifest"]["version"] == "v1" and sent["gateReport"]
    try:
        post(url + "bad", b"{}", headers_for("k", False), sleep=slept.append)
        raise AssertionError("a 400 was retried or accepted")
    except SendError as e:
        assert "HTTP 400" in str(e) and len(seen) == 4 and slept == [1, 4]
    srv.shutdown()
    print("publish.py self-test: ok (limits, order, coverage, no-gzip, oversized panel, retries 1/4 s then success, "
          "400 not retried, activate body)")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--trevor", default=ROOT)
    ap.add_argument("--panels", default=None, help="panels.py output (default <trevor>/artifacts/panels)")
    ap.add_argument("--gate-report", default=None, help="default <panels>/gate_report.json")
    ap.add_argument("--backend", default=os.environ.get("TREVOR_BACKEND", "http://127.0.0.1:3060"))
    ap.add_argument("--state", default=None)
    ap.add_argument("--dry-run", metavar="OUTDIR", default=None, help="write the requests to OUTDIR, send nothing")
    ap.add_argument("--activate", metavar="VERSION", default=None, help="re-activate a published version (rollback)")
    ap.add_argument("--no-activate", action="store_true", help="upload the chunks only; --activate flips later")
    ap.add_argument("--no-gzip", action="store_true", help="send raw JSON (a backend without request decompression)")
    ap.add_argument("--max-compressed", type=int, default=MAX_COMPRESSED)
    ap.add_argument("--max-decompressed", type=int, default=MAX_DECOMPRESSED)
    ap.add_argument("--no-chunk-shas", action="store_true", default=os.environ.get("TREVOR_CHUNK_SHAS") == "0",
                    help="send no chunk shas (the 2026-10-16 publish; the backend then carries no cached answer over)")
    ap.add_argument("--chunk-sha-corpora", default=os.environ.get("TREVOR_CHUNK_SHA_CORPORA", CHUNK_SHA_CORPORA),
                    help="comma-separated corpus dirs under --trevor whose chunk ids and contentShas are sent")
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        self_test()
        return 0
    a.trevor = os.path.abspath(a.trevor)
    a.panels = os.path.abspath(a.panels or os.path.join(a.trevor, "artifacts", "panels"))
    a.state = os.path.abspath(a.state or os.environ.get("TREVOR_STATE_DIR") or os.path.join(a.trevor, "artifacts", "publish"))
    a.key = os.environ.get("TREVOR_WORKER_KEY")
    os.makedirs(a.state, exist_ok=True)
    try:
        return activate_only(a) if a.activate else publish(a)
    except SendError as e:
        sys.stderr.write(f"publish: FAILED, the active version is unchanged: {e}\n")
        return 1


if __name__ == "__main__":
    sys.exit(main())
