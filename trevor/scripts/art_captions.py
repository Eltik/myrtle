#!/usr/bin/env python3
"""Operator art captions from a local vision model (Gemma 4 12B + mmproj).

Trevor answers from text; questions about what an illustration shows ("which
operator has their height written in their E2 art?") need a description of
the image. This script writes one model-written caption and one transcription
of visible text per illustration. The operator and skin of each image come
from the game data (skin_table + character_table), never from the model.

  python3 scripts/art_captions.py select
      manifest of every E2 art and every outfit with its image file
      -> artifacts/art/manifest.jsonl (no model, seconds)
  SERVER=http://127.0.0.1:8091 python3 scripts/art_captions.py caption [--kind e2|skin|all] [--n 20] [--seed 7]
      caption the images (resumable: rows already in captions.jsonl are skipped)
      -> artifacts/art/captions.jsonl; needs llama-server with --mmproj
  SERVER=... python3 scripts/art_captions.py text [--kind ...] [--n ...] [--first IDS]
      second pass for small lettering: the same transcription prompt on a
      2x2 grid of full-resolution tiles (ART_TILES) -> artifacts/art/text_tiles.jsonl
  python3 scripts/art_captions.py ocr [--kind ...]
      cross-check: macOS Vision text recognition (on-device, no model lock)
      over the same images -> artifacts/art/ocr.jsonl
  python3 scripts/art_captions.py stats
  SERVER=... python3 scripts/art_captions.py design [--ids char_a,char_b | --split split.json --split-part dev,heldout] [--minutes M]
      visual design reading (2026-10-04 night): per operator, what real-world animal, creature, plant or object
      the E2 art (E0 when there is no E2 or the E2 reading is unclear) is modelled on, with the visible features,
      the most specific species and a confidence; the race is the only context given -> artifacts/art/design.jsonl
      (resumable, keyed on operator, art and prompt hash); read by design_infer.py with DESIGN_VISUAL=1
  python3 scripts/art_captions.py units
      one P4 unit per captioned image (kind "art") -> artifacts/art/units.jsonl, and
      artifacts/art/p4_art_units.jsonl = artifacts/sources/all_units.jsonl (P4's units) + the art units, for
      `build-units --units artifacts/art/p4_art_units.jsonl --out artifacts/p4-art` (`ask --art` reads it;
      P4 itself never holds them, so leaving p4-art unbuilt is the kill switch)

Images are cropped to their alpha bounding box, composited on mid grey and
downscaled to ART_MAX_SIDE (default 1536) before encoding as JPEG. The
server's --image-max-tokens bounds the visual tokens per image.
ART_PROMPT=v1 is the only prompt so far; its hash is written into each row.
"""
import argparse, base64, hashlib, io, json, os, random, subprocess, sys, time, urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT.parent / "assets" / "output" / "en"
EXCEL = ASSETS / "gamedata" / "excel"
TEX = ASSETS / "textures"
OUT = ROOT / "artifacts" / "art"
SERVER = os.environ.get("SERVER", "http://127.0.0.1:8091")
MAX_SIDE = int(os.environ.get("ART_MAX_SIDE", "1536"))

SYSTEM = """You describe illustrations from the game Arknights for a lore database. Be factual and concrete: describe only what is visible. Do not name the character, the game, or guess at story context."""

USER = """Describe this character illustration.

Return JSON with:
- "caption": 3 to 6 sentences. The character's appearance (hair, eyes, ears, horns, tail or other features, clothing, colors), pose and expression, what they hold or wear (weapons, tools, accessories), other figures or creatures, and the setting or background.
- "visible_text": every piece of written text you can actually read in the image (words, numbers, letters, signs, labels, measurements, tattoos, logos), each transcribed exactly as written. Do not include text you cannot read clearly; do not guess. Use an empty list when the image has no legible text.
- "text_locations": for each item of visible_text, a few words on where it appears (for example "on the sleeve", "on a sign in the background")."""

SCHEMA = {
    "type": "object",
    "properties": {
        "caption": {"type": "string"},
        "visible_text": {"type": "array", "items": {"type": "string", "maxLength": 120}, "maxItems": 24},
        "text_locations": {"type": "array", "items": {"type": "string", "maxLength": 120}, "maxItems": 24},
    },
    "required": ["caption", "visible_text", "text_locations"],
}
PROMPT_HASH = hashlib.sha256((SYSTEM + USER + json.dumps(SCHEMA, sort_keys=True)).encode()).hexdigest()[:12]


def kv(lst):
    return {x["key"]: x["value"] for x in lst}


def select(_a):
    skins = kv(json.load(open(EXCEL / "skin_table.json"))["CharSkins"])
    chars = kv(json.load(open(EXCEL / "character_table.json"))["Characters"])
    rows, missing = [], {"e2": 0, "skin": 0}
    for sid, v in skins.items():
        ds = v.get("DisplaySkin") or {}
        group, pid, cid = ds.get("SkinGroupId") or "", v.get("PortraitId"), v.get("CharId")
        if not pid or not cid or not cid.startswith("char_"):
            continue
        if group == "ILLUST_2":
            kind, path = "e2", TEX / "chararts" / cid / f"{pid}.png"
        elif not group.startswith("ILLUST") and ds.get("SkinName"):
            kind, path = "skin", TEX / "skinpack" / cid / f"{pid}.png"
        else:
            continue
        if not path.exists():
            missing[kind] += 1
            continue
        c = chars.get(cid, {})
        rows.append({
            "id": sid, "kind": kind, "char_id": cid, "operator": c.get("Name"),
            "skin_name": ds.get("SkinName"), "skin_group": ds.get("SkinGroupName"),
            "illustrators": ds.get("DrawerList") or [], "skin_text": ds.get("Content") or ds.get("Dialog"),
            "path": str(path.relative_to(ASSETS)),
        })
    rows.sort(key=lambda r: (r["kind"], r["id"]))
    OUT.mkdir(parents=True, exist_ok=True)
    with open(OUT / "manifest.jsonl", "w") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    n = {k: sum(r["kind"] == k for r in rows) for k in ("e2", "skin")}
    print(f"manifest: {n['e2']} E2 arts, {n['skin']} outfits; image missing: {missing}")


def manifest(kind):
    rows = [json.loads(l) for l in open(OUT / "manifest.jsonl")]
    return [r for r in rows if kind == "all" or r["kind"] == kind]


def prepare(path):
    from PIL import Image
    im = Image.open(ASSETS / path).convert("RGBA")
    box = im.getchannel("A").point(lambda a: 255 if a > 8 else 0).getbbox()
    if box:
        im = im.crop(box)
    bg = Image.new("RGB", im.size, (128, 128, 128))
    bg.paste(im, mask=im.getchannel("A"))
    bg.thumbnail((MAX_SIDE, MAX_SIDE), Image.LANCZOS)
    buf = io.BytesIO()
    bg.save(buf, "JPEG", quality=90)
    return buf.getvalue(), bg.size


TEXT_USER = """This is a crop of a larger illustration. Transcribe every piece of written text you can actually read in it (words, numbers, letters, signs, labels, measurements), each exactly as written. Do not include text you cannot read clearly; do not guess. Return JSON {"visible_text": [...]}, an empty list when there is no legible text."""
TEXT_SCHEMA = {"type": "object", "properties": {"visible_text": {"type": "array", "items": {"type": "string", "maxLength": 120}, "maxItems": 24}},
               "required": ["visible_text"]}


def tiles(path, grid=2, overlap=0.15):
    """Full-resolution tiles of the alpha-cropped composite, each JPEG-encoded at up to MAX_SIDE."""
    from PIL import Image
    im = Image.open(ASSETS / path).convert("RGBA")
    box = im.getchannel("A").point(lambda a: 255 if a > 8 else 0).getbbox()
    if box:
        im = im.crop(box)
    bg = Image.new("RGB", im.size, (128, 128, 128))
    bg.paste(im, mask=im.getchannel("A"))
    W, H = bg.size
    tw, th = W / grid, H / grid
    out = []
    for gy in range(grid):
        for gx in range(grid):
            x0, y0 = max(0, gx * tw - overlap * tw), max(0, gy * th - overlap * th)
            x1, y1 = min(W, (gx + 1) * tw + overlap * tw), min(H, (gy + 1) * th + overlap * th)
            t = bg.crop((int(x0), int(y0), int(x1), int(y1)))
            t.thumbnail((MAX_SIDE, MAX_SIDE), Image.LANCZOS)
            buf = io.BytesIO()
            t.save(buf, "JPEG", quality=90)
            out.append(((gx, gy), buf.getvalue(), t.size))
    return out


def text(a):
    """Tiled text transcription (ART_TILES grid, default 2x2) for small lettering the whole-image pass misses."""
    grid = int(os.environ.get("ART_TILES", "2"))
    rows = pick(manifest(a.kind), a.n, a.seed)
    if a.first:
        front = [x for x in a.first.split(",") if x]
        rows = [r for r in manifest("all") if r["id"] in front] + [r for r in rows if r["id"] not in front]
    out = OUT / "text_tiles.jsonl"
    done = done_ids(out)
    todo = [r for r in rows if r["id"] not in done]
    print(f"{len(todo)} to transcribe in {grid}x{grid} tiles", flush=True)
    deadline = time.time() + a.minutes * 60 if a.minutes else None
    for i, r in enumerate(todo):
        if deadline and time.time() > deadline:
            print(f"time budget of {a.minutes} min reached after {i}", flush=True)
            break
        t0, per, union = time.time(), [], []
        for (gx, gy), jpg, size in tiles(r["path"], grid):
            body = {"messages": [{"role": "user", "content": [
                        {"type": "image_url", "image_url": {"url": "data:image/jpeg;base64," + base64.b64encode(jpg).decode()}},
                        {"type": "text", "text": TEXT_USER}]}],
                    "temperature": 0, "max_tokens": 600,
                    "response_format": {"type": "json_schema", "json_schema": {"name": "text", "schema": TEXT_SCHEMA}}}
            try:
                vt = json.loads(post(SERVER + "/v1/chat/completions", body)["choices"][0]["message"]["content"]).get("visible_text", [])
            except Exception as e:
                print(f"tile error {r['id']} {gx},{gy}: {e}", flush=True)
                vt = None
            per.append({"tile": [gx, gy], "px": list(size), "visible_text": vt})
            for v in vt or []:
                if v not in union:
                    union.append(v)
        dt = time.time() - t0
        with open(out, "a") as f:
            f.write(json.dumps({"id": r["id"], "kind": r["kind"], "operator": r["operator"], "skin_name": r["skin_name"],
                                "grid": grid, "visible_text": union, "tiles": per, "seconds": round(dt, 2),
                                "prompt": hashlib.sha256((TEXT_USER + json.dumps(TEXT_SCHEMA, sort_keys=True)).encode()).hexdigest()[:12],
                                "source": "model-written transcription"}, ensure_ascii=False) + "\n")
        print(f"tiles {i + 1}/{len(todo)} {r['id']} {dt:.1f}s {union}", flush=True)


def post(url, body, timeout=600):
    req = urllib.request.Request(url, json.dumps(body).encode(), {"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.load(r)


def pick(rows, n, seed):
    """Seeded order; the first n of the shuffled list (n=0: all), so a pilot is a prefix of the full run."""
    rows = list(rows)
    random.Random(seed).shuffle(rows)
    return rows[:n] if n else rows


def done_ids(path):
    if not path.exists():
        return set()
    # a row the model broke (parse_error: a repetition loop past max_tokens) is redone; readers keep the last row per id
    return {r["id"] for r in map(json.loads, filter(str.strip, open(path))) if not r.get("parse_error")}


def caption(a):
    rows = pick(manifest(a.kind), a.n, a.seed)
    out = OUT / "captions.jsonl"
    done = done_ids(out)
    todo = [r for r in rows if r["id"] not in done]
    if a.ids:
        want = set(a.ids.split(","))
        todo = [r for r in manifest("all") if r["id"] in want and r["id"] not in done]
    if a.first:
        front = [x for x in a.first.split(",") if x]
        extra = [r for r in manifest("all") if r["id"] in front and r["id"] not in done and r not in todo]
        todo = sorted(extra + todo, key=lambda r: 0 if r["id"] in front else 1)
    print(f"{len(todo)} to caption ({len(done)} already in captions.jsonl)", flush=True)
    deadline = time.time() + a.minutes * 60 if a.minutes else None
    for i, r in enumerate(todo):
        if deadline and time.time() > deadline:
            print(f"time budget of {a.minutes} min reached after {i}", flush=True)
            break
        jpg, size = prepare(r["path"])
        uri = "data:image/jpeg;base64," + base64.b64encode(jpg).decode()
        body = {
            "messages": [
                {"role": "system", "content": SYSTEM},
                {"role": "user", "content": [
                    {"type": "image_url", "image_url": {"url": uri}},
                    {"type": "text", "text": USER},
                ]},
            ],
            "temperature": 0, "max_tokens": 700,
            "response_format": {"type": "json_schema", "json_schema": {"name": "art", "schema": SCHEMA}},
        }
        t0 = time.time()
        for attempt in range(3):
            try:
                resp = post(SERVER + "/v1/chat/completions", body)
                break
            except Exception as e:  # one failed decode 500s the request; retry
                print(f"retry {r['id']}: {e}", flush=True)
                time.sleep(5)
        else:
            sys.exit(f"three failed requests on {r['id']}; stopping (is the server up?)")
        dt = time.time() - t0
        msg = resp["choices"][0]["message"]["content"]
        try:
            j = json.loads(msg)
        except json.JSONDecodeError:
            j = {"caption": msg, "visible_text": [], "text_locations": [], "parse_error": True}
        u = resp.get("usage", {})
        rec = {**{k: r[k] for k in ("id", "kind", "char_id", "operator", "skin_name", "illustrators", "path")},
               **j, "image_px": list(size), "prompt_tokens": u.get("prompt_tokens"),
               "completion_tokens": u.get("completion_tokens"), "seconds": round(dt, 2),
               "model": "gemma-4-12b-it-qat-q4_0+mmproj", "prompt": PROMPT_HASH, "max_side": MAX_SIDE,
               "source": "model-written description"}
        with open(out, "a") as f:
            f.write(json.dumps(rec, ensure_ascii=False) + "\n")
        print(f"{i + 1}/{len(todo)} {r['id']} {dt:.1f}s pt={u.get('prompt_tokens')} text={j.get('visible_text')}", flush=True)


SWIFT = r'''
import Foundation
import Vision
import AppKit
// usage: ocr <image> ... ; prints one JSON line per image; box = [x, y from top, w, h] as fractions
for p in CommandLine.arguments.dropFirst() {
    guard let img = NSImage(contentsOfFile: p), let cg = img.cgImage(forProposedRect: nil, context: nil, hints: nil) else { print("{\"path\":\"\(p)\",\"error\":true}"); continue }
    let req = VNRecognizeTextRequest()
    req.recognitionLevel = .accurate
    req.usesLanguageCorrection = false
    try? VNImageRequestHandler(cgImage: cg, options: [:]).perform([req])
    var items: [[String: Any]] = []
    for o in req.results ?? [] { if let c = o.topCandidates(1).first { let b = o.boundingBox; items.append(["text": c.string, "conf": c.confidence, "box": [b.minX, 1 - b.maxY, b.width, b.height]]) } }
    let d = try! JSONSerialization.data(withJSONObject: ["path": p, "items": items])
    print(String(data: d, encoding: .utf8)!)
}
'''


def ocr(a):
    """On-device Apple Vision OCR over the full-resolution composite (no LLM)."""
    import tempfile
    from PIL import Image
    binp = OUT / "ocr-bin"
    if not binp.exists():
        src = OUT / "ocr.swift"
        src.write_text(SWIFT)
        subprocess.run(["swiftc", "-O", str(src), "-o", str(binp)], check=True)
    out = OUT / "ocr.jsonl"
    done = done_ids(out)
    rows = [r for r in pick(manifest(a.kind), a.n, a.seed) if r["id"] not in done]
    tmp = Path(tempfile.mkdtemp())
    for i in range(0, len(rows), 16):
        batch = rows[i:i + 16]
        files = []
        for j, r in enumerate(batch):
            im = Image.open(ASSETS / r["path"]).convert("RGBA")
            bg = Image.new("RGB", im.size, (128, 128, 128))
            bg.paste(im, mask=im.getchannel("A"))
            fp = tmp / f"{j}.png"
            bg.save(fp)
            files.append(str(fp))
        res = subprocess.run([str(binp), *files], capture_output=True, text=True, check=True).stdout.splitlines()
        with open(out, "a") as f:
            for r, line in zip(batch, res):
                items = json.loads(line).get("items", [])
                f.write(json.dumps({"id": r["id"], "kind": r["kind"], "operator": r["operator"],
                                    "skin_name": r["skin_name"], "ocr": items}, ensure_ascii=False) + "\n")
        print(f"ocr {min(i + 16, len(rows))}/{len(rows)}", flush=True)
    for fp in tmp.iterdir():
        fp.unlink()
    tmp.rmdir()


def stats(_a):
    p = OUT / "captions.jsonl"
    rows = list({r["id"]: r for r in map(json.loads, open(p))}.values()) if p.exists() else []
    if not rows:
        print("no captions")
        return
    s = sorted(r["seconds"] for r in rows)
    pt = sorted(r["prompt_tokens"] or 0 for r in rows)
    print(f"{len(rows)} captions ({sum(r['kind'] == 'e2' for r in rows)} E2, {sum(r['kind'] == 'skin' for r in rows)} outfits); "
          f"seconds median {s[len(s) // 2]}, total {sum(s):.0f}; prompt tokens median {pt[len(pt) // 2]}; "
          f"with visible text {sum(bool(r.get('visible_text')) for r in rows)}; parse errors {sum(bool(r.get('parse_error')) for r in rows)}")
    t = OUT / "text_tiles.jsonl"
    if t.exists():
        tr = list({r["id"]: r for r in map(json.loads, open(t))}.values())
        ts = sorted(r["seconds"] for r in tr)
        print(f"{len(tr)} tiled transcriptions; seconds median {ts[len(ts) // 2]}; with text {sum(bool(r['visible_text']) for r in tr)}")


def clean(t):
    """Skin flavor text without the game's rich-text tags."""
    import re
    return re.sub(r"</?color[^>]*>|</?[bi]>", "", t or "").strip()


def units(_a):
    """One unit per captioned image. Operator, outfit, illustrators and flavor text come from the game data; the
    description is the model's caption; visible text only from the tiled pass (whole-image text misread Pallas's
    "162cm" as "82m", 2026-10-02), and only for images the tiled pass covered."""
    man = {r["id"]: r for r in map(json.loads, open(OUT / "manifest.jsonl"))}
    caps = {}
    for r in map(json.loads, filter(str.strip, open(OUT / "captions.jsonl"))):
        if not r.get("parse_error") and r.get("caption"):
            caps[r["id"]] = r
    tiles_p = OUT / "text_tiles.jsonl"
    tl = {r["id"]: r for r in map(json.loads, filter(str.strip, open(tiles_p)))} if tiles_p.exists() else {}
    import re
    attrs = ROOT / "artifacts" / "entities" / "operator_attributes.jsonl"
    height = {r["charId"]: r.get("heightCm") for r in map(json.loads, open(attrs))} if attrs.exists() else {}
    out = []
    for i, (iid, m) in enumerate(man.items()):
        c = caps.get(iid)
        if not c:
            continue
        what = "E2 (Elite 2) art" if m["kind"] == "e2" else f"outfit \"{m['skin_name']}\"" + (f" ({m['skin_group']})" if m.get("skin_group") else "")
        head = f"Operator art: {m['operator']}, {what}"
        lines = [head, f"Illustrators: {', '.join(m.get('illustrators') or []) or 'not listed'}."]
        flavor = clean(m.get("skin_text"))
        # the E2 boilerplate every promoted operator shares says nothing about this picture
        if flavor and not flavor.startswith("An outfit that has been improved after the Operator's Promotion."):
            lines.append(f"Outfit text (game text): {flavor}")
        lines.append(f"Description (written by a vision model from the image, not game text): {c['caption'].strip()}")
        t = tl.get(iid)
        if t and t.get("visible_text"):
            lines.append("Text visible in the art (vision-model transcription, may be misread): "
                         + "; ".join(f'"{v}"' for v in t["visible_text"]))
            # a transcribed length equal to the operator file's height (game data) is named as such: the question
            # says "height", the art says "162cm", and no retriever joins the two
            h = height.get(m["char_id"])
            for v in t["visible_text"]:
                if h and any(int(x) == h for x in re.findall(r"(\d{2,3})\s*cm\b", v, re.I)):
                    lines.append(f'The transcribed "{v}" matches {m["operator"]}\'s height in the operator file ({h} cm, game data): '
                                 f"the art has {m['operator']}'s height written in it.")
                    break
        sid = "art_" + iid.replace("#", "_").replace("@", "_")
        out.append({"storyId": sid, "groupId": "art", "title": head, "text": "\n".join(lines), "speakers": [m["operator"]]})
    with open(OUT / "units.jsonl", "w") as f:
        for u in out:
            f.write(json.dumps(u, ensure_ascii=False) + "\n")
    base = (ROOT / "artifacts" / "sources" / "all_units.jsonl").read_text()
    with open(OUT / "p4_art_units.jsonl", "w") as f:
        f.write(base if base.endswith("\n") else base + "\n")
        for u in out:
            f.write(json.dumps(u, ensure_ascii=False) + "\n")
    print(f"{len(out)} art units ({sum(1 for u in out if 'Text visible' in u['text'])} with tiled text) of {len(man)} images "
          f"-> {OUT / 'units.jsonl'}, {OUT / 'p4_art_units.jsonl'}")


# Visual design reading (2026-10-04 night, Ian: Trevor deduces design inspirations from the game data, its art
# included). The captions describe what is visible but never say what the design is modelled on, and Lucilla's
# giant phantom jelly has no tie in the game text (an umbrella with wriggling tentacles, bubbles, Ægir). This pass asks
# the vision model one question per operator: which real-world animal, creature, plant or object the design resembles,
# the visible features that show it, the most specific species they justify, and a confidence. The only context it is
# given is the operator's race from the game data (never the codename, which would lead it); the E2 art is read first
# and the E0 art too when there is no E2 or the E2 reading is low-confidence or names nothing. Rows ->
# artifacts/art/design.jsonl, read by design_infer.py only with DESIGN_VISUAL=1.
DESIGN_SYSTEM = """You study character illustrations from the game Arknights to say what real-world thing each character's design is modelled on. Judge only from what is visible in the image. Never name the character, the game, or an artist."""

DESIGN_USER = """The game gives this character's race as "{race}" (a fictional race of the game; a race name gives at most an animal family, never the species).

Many Arknights characters are people with the features of one specific real animal; others draw on a mythical creature, a plant or an object. Work in two steps and name only what you can actually see.
1. The character's own body: ears (shape, size, color, tips), tail (length, thickness, fur, stripes, spots, rings), horns or antlers, wings or feathers, scales, fins, tentacles, shell, fur colors and markings. These features decide the animal.
2. Everything else: creatures that accompany the character, background figures, emblems, headwear, clothing patterns, accessories, weapon. Use these to choose the exact species only when they match the character's own body features; a background creature alone does not make the character that animal.

Return JSON with:
- "body_features": the character's own animal features from step 1, each a few concrete words (color, shape, where), or an empty list.
- "motifs": the motifs from step 2 that bear on the design, each a few concrete words, or an empty list.
- "subject": the most specific real-world species or thing the evidence justifies, by its common English name, or "none" when nothing beyond an ordinary human shows.
- "broader": the broader group it belongs to, or "none".
- "kind": one of "animal", "mythical creature", "plant", "object", "none".
- "confidence": "high" when several distinct body features agree on this exact species or thing, "medium" when they show the group but not clearly the species, "low" when it is a guess."""

DESIGN_SCHEMA = {
    "type": "object",
    "properties": {
        "body_features": {"type": "array", "items": {"type": "string", "maxLength": 100}, "maxItems": 8},
        "motifs": {"type": "array", "items": {"type": "string", "maxLength": 100}, "maxItems": 6},
        "subject": {"type": "string", "maxLength": 80},
        "broader": {"type": "string", "maxLength": 60},
        "kind": {"type": "string", "enum": ["animal", "mythical creature", "plant", "object", "none"]},
        "confidence": {"type": "string", "enum": ["high", "medium", "low"]},
    },
    "required": ["body_features", "motifs", "subject", "broader", "kind", "confidence"],
}
# v2 (pilot 2026-10-04 night, opt-in ART_DESIGN_PROMPT=v2): one list of features, companions and emblems "decide the
# species". On the 22-operator dev pilot it named a background creature for the character (Cliffheart a wolf from the
# wolf-like heads behind her, Specter the Unchained a whale from a skeleton); v3 above separates the body from motifs.
DESIGN_USER_V2 = """The game gives this character's race as "{race}" (a fictional race of the game; a race name gives at most an animal family, never the species).

Many Arknights characters are people with the features of one specific real animal: ears, tails, horns, antlers, wings, feathers, scales, fins, tentacles, shells, fur colors and patterns. Others draw on a mythical creature, a plant or an object, shown in the hair, headwear, clothing, accessories or weapon. Animals that accompany the character or appear in the background, in emblems, on clothing or on equipment often show the same species as the character's own ears and tail; when they agree with the character's features, they decide the species. Look closely at every such detail and name only features you can actually see, then answer which real-world animal, mythical creature, plant or object this character's design most resembles or is modelled on.

Return JSON with:
- "features": the visible features that point to it, each a few concrete words describing what you see (color, shape, where it is).
- "subject": the most specific real-world species or thing these features justify, by its common English name, or "none" when nothing beyond an ordinary human shows.
- "broader": the broader group it belongs to, or "none".
- "kind": one of "animal", "mythical creature", "plant", "object", "none".
- "confidence": "high" when several distinct features agree on this exact species or thing, "medium" when they show the group but not clearly the species, "low" when it is a guess."""
if os.environ.get("ART_DESIGN_PROMPT") == "v2":
    DESIGN_USER = DESIGN_USER_V2
    DESIGN_SCHEMA = {**DESIGN_SCHEMA, "properties": {"features": {**DESIGN_SCHEMA["properties"]["body_features"], "maxItems": 10},
                     **{k: v for k, v in DESIGN_SCHEMA["properties"].items() if k not in ("body_features", "motifs")}},
                     "required": ["features", "subject", "broader", "kind", "confidence"]}
# the server's --image-max-tokens changes what the model sees, so it is part of the key (ART_IMAGE_TOKENS, default 1120)
IMAGE_TOKENS = os.environ.get("ART_IMAGE_TOKENS", "1120")
DESIGN_HASH = hashlib.sha256((DESIGN_SYSTEM + DESIGN_USER + json.dumps(DESIGN_SCHEMA, sort_keys=True)
                              + f"{MAX_SIDE}/{IMAGE_TOKENS}").encode()).hexdigest()[:12]


def design_images():
    """charId -> {"e2": (skin id, path), "e0": (skin id, path)} from skin_table (ILLUST_2 / ILLUST_0), files that exist."""
    skins = kv(json.load(open(EXCEL / "skin_table.json"))["CharSkins"])
    out = {}
    for sid, v in skins.items():
        ds = v.get("DisplaySkin") or {}
        group, pid, cid = ds.get("SkinGroupId") or "", v.get("PortraitId"), v.get("CharId")
        kind = {"ILLUST_2": "e2", "ILLUST_0": "e0"}.get(group)
        if not kind or not pid or not cid or not cid.startswith("char_"):
            continue
        path = TEX / "chararts" / cid / f"{pid}.png"
        if path.exists():
            out.setdefault(cid, {})[kind] = (sid, str(path.relative_to(ASSETS)))
    return out


def design_unclear(row):
    return row is None or row.get("parse_error") or row.get("confidence") == "low" or \
        (row.get("subject") or "none").strip().lower() in ("none", "null", "")


def design(a):
    attrs = {r["charId"]: r for r in map(json.loads, open(ROOT / "artifacts" / "entities" / "operator_attributes.jsonl"))}
    imgs = design_images()
    ids = [x for x in a.ids.split(",") if x] if a.ids else sorted(imgs)
    if a.split:
        sp = json.load(open(a.split))
        ids = [c for part in a.split_part.split(",") for c in sp[part]]
    out = Path(os.environ.get("ART_DESIGN_OUT") or OUT / "design.jsonl")
    # Incremental: a row is keyed on the source PNG's content hash and the prompt hash, so a rerun (and update.sh)
    # reads only new or changed art; compact rows (no raw model text).
    rows = {r["key"]: r for r in map(json.loads, filter(str.strip, open(out)))} if out.exists() else {}
    deadline = time.time() + a.minutes * 60 if a.minutes else None
    n_done = n_cached = 0
    for cid in ids:
        if cid not in attrs or cid not in imgs:
            print(f"skip {cid}: no attributes or no art", flush=True)
            continue
        first = "e2" if "e2" in imgs[cid] else "e0"
        for art in (first, "e0") if first == "e2" else (first,):
            if art not in imgs[cid]:
                continue
            sid, path = imgs[cid][art]
            key = hashlib.sha256((ASSETS / path).read_bytes()).hexdigest()[:16] + "-" + DESIGN_HASH
            if art == "e0" and first == "e2":
                if not design_unclear(rows.get(e2key)):
                    continue
            if art == "e2":
                e2key = key
            if key in rows:
                n_cached += 1
                continue
            if deadline and time.time() > deadline:
                print(f"time budget of {a.minutes} min reached after {n_done}", flush=True)
                return
            race = attrs[cid].get("race") or "not stated"
            jpg, size = prepare(path)
            body = {"messages": [
                        {"role": "system", "content": DESIGN_SYSTEM},
                        {"role": "user", "content": [
                            {"type": "image_url", "image_url": {"url": "data:image/jpeg;base64," + base64.b64encode(jpg).decode()}},
                            {"type": "text", "text": DESIGN_USER.format(race=race)}]}],
                    "temperature": 0, "max_tokens": 500,
                    "response_format": {"type": "json_schema", "json_schema": {"name": "design", "schema": DESIGN_SCHEMA}}}
            t0 = time.time()
            for _ in range(3):
                try:
                    resp = post(SERVER + "/v1/chat/completions", body)
                    break
                except Exception as e:
                    print(f"retry {cid} {art}: {e}", flush=True)
                    time.sleep(5)
            else:
                sys.exit(f"three failed requests on {cid}; stopping (is the server up?)")
            dt = time.time() - t0
            try:
                j = json.loads(resp["choices"][0]["message"]["content"])
                j = {k: j[k] for k in ("subject", "broader", "kind", "confidence", "body_features", "motifs", "features") if k in j}
            except (json.JSONDecodeError, KeyError):
                j = {"parse_error": True}
            rec = {"key": key, "char_id": cid, "operator": attrs[cid]["name"], "art": art, "id": sid, **j,
                   "seconds": round(dt, 2)}
            with open(out, "a") as f:
                f.write(json.dumps(rec, ensure_ascii=False) + "\n")
            rows[key] = rec
            n_done += 1
            print(f"{n_done} {attrs[cid]['name']} {art} {dt:.1f}s: {j.get('subject')} / {j.get('broader')} "
                  f"({j.get('confidence')}) {j.get('body_features')} | {j.get('motifs')}", flush=True)
    print(f"design: {n_done} images read, {n_cached} reused from design.jsonl", flush=True)


# Tiled design probe (2026-10-05, Ian: deduce designs from what the game shows). The whole-image reading above sees the
# art at about 1120 image tokens, so small creatures (Lucilla's jellies, a companion in a corner) blur away; this pass
# asks the same 12B vision model, per 2x2 full-resolution tile (tiles() above, the text pass's tiling), which creatures,
# animals, plants or objects are depicted and their visible features, then aggregates per image: each subject with the
# tiles it appears in, where it is (the character's body, a companion, clothing, weapon, background) and its features.
# One image per operator: E2, else E0. Rows -> artifacts/art/design_tiles.jsonl keyed on the PNG's sha256 + this
# prompt's hash (incl. the grid, ART_MAX_SIDE and ART_IMAGE_TOKENS), compact (no raw model text), so a rerun reads only
# new or changed art. Read by design_infer.py evidence (the deduction evidence file of ask --design-deduce).
TILE_USER = """This is one part (tile {n} of 4, the {pos}) of a larger illustration of one character. The game gives the character's race as "{race}" (a fictional race of the game; it gives at most an animal family).

List every creature, animal, plant or object in this crop that a character design could be modelled on:
- animal features on the character's own body: ears, tail, horns, antlers, wings, feathers, scales, fins, tentacles, shell, fur markings;
- creatures or animals accompanying the character, or drawn on clothing, emblems or equipment;
- objects or plants used as a motif in headwear, clothing, accessories or the weapon.
Ignore ordinary clothing, plain backgrounds and text. For each, give the most specific real-world common English name the visible features justify (a species when they show it, otherwise the group), and only what you can actually see.

Return JSON {{"subjects": [{{"name": ..., "kind": "animal" | "mythical creature" | "plant" | "object", "where": "body" | "companion" | "clothing" | "weapon" | "background", "features": [a few concrete words each: color, shape, where]}}]}}, an empty list when the crop shows nothing of the kind."""
TILE_SCHEMA = {"type": "object", "properties": {"subjects": {"type": "array", "maxItems": 4, "items": {
    "type": "object", "properties": {
        "name": {"type": "string", "maxLength": 60},
        "kind": {"type": "string", "enum": ["animal", "mythical creature", "plant", "object"]},
        "where": {"type": "string", "enum": ["body", "companion", "clothing", "weapon", "background"]},
        "features": {"type": "array", "items": {"type": "string", "maxLength": 80}, "maxItems": 4}},
    "required": ["name", "kind", "where", "features"]}}}, "required": ["subjects"]}
TILE_POS = {(0, 0): "top left", (1, 0): "top right", (0, 1): "bottom left", (1, 1): "bottom right"}
TILE_HASH = hashlib.sha256((DESIGN_SYSTEM + TILE_USER + json.dumps(TILE_SCHEMA, sort_keys=True)
                            + f"{MAX_SIDE}/{IMAGE_TOKENS}/2x2").encode()).hexdigest()[:12]


def tile_aggregate(per):
    """Per-tile subject lists -> one compact list: a subject per normalized name with its tile count, places, features."""
    agg = {}
    for t, subs in enumerate(per):
        for s in subs:
            name = " ".join((s.get("name") or "").lower().split())
            if not name or name in ("none", "human", "person", "woman", "man", "girl", "boy"):
                continue
            a = agg.setdefault(name, {"name": name, "kind": s.get("kind"), "where": [], "tiles": set(), "features": []})
            a["tiles"].add(t)
            if s.get("where") not in a["where"]:
                a["where"].append(s.get("where"))
            for f in s.get("features") or []:
                if f and f not in a["features"] and len(a["features"]) < 6:
                    a["features"].append(f)
    out = [{**a, "tiles": len(a["tiles"])} for a in agg.values()]
    out.sort(key=lambda a: ("body" not in a["where"], -a["tiles"], a["name"]))
    return out[:8]


def design_tiles(a):
    from concurrent.futures import ThreadPoolExecutor
    attrs = {r["charId"]: r for r in map(json.loads, open(ROOT / "artifacts" / "entities" / "operator_attributes.jsonl"))}
    imgs = design_images()
    ids = [x for x in a.ids.split(",") if x] if a.ids else sorted(imgs)
    if a.split:
        sp = json.load(open(a.split))
        ids = [c for part in a.split_part.split(",") for c in sp[part]]
    first = [x for x in a.first.split(",") if x]
    ids = first + [c for c in ids if c not in first]
    out = Path(os.environ.get("ART_TILES_OUT") or OUT / "design_tiles.jsonl")
    rows = {r["key"]: r for r in map(json.loads, filter(str.strip, open(out)))} if out.exists() else {}
    threads = int(os.environ.get("ART_THREADS", "2"))
    deadline = time.time() + a.minutes * 60 if a.minutes else None
    n_done = n_cached = 0

    def ask(arg):
        (gx, gy), jpg, race = arg
        body = {"messages": [
                    {"role": "system", "content": DESIGN_SYSTEM},
                    {"role": "user", "content": [
                        {"type": "image_url", "image_url": {"url": "data:image/jpeg;base64," + base64.b64encode(jpg).decode()}},
                        {"type": "text", "text": TILE_USER.format(n=gy * 2 + gx + 1, pos=TILE_POS[(gx, gy)], race=race)}]}],
                "temperature": 0, "max_tokens": 400,
                "response_format": {"type": "json_schema", "json_schema": {"name": "tile", "schema": TILE_SCHEMA}}}
        for k in range(3):
            try:
                resp = post(SERVER + "/v1/chat/completions", body)
                return json.loads(resp["choices"][0]["message"]["content"]).get("subjects") or []
            except (json.JSONDecodeError, KeyError):
                return None
            except Exception as e:  # one failed decode 500s every in-flight request; retry
                print(f"retry tile {gx},{gy}: {e}", flush=True)
                time.sleep(5 * (k + 1))
        raise SystemExit("three failed requests on one tile; stopping (is the server up?)")

    if os.environ.get("PLAN"):  # update.sh: how many images would be read
        n = 0
        for cid in ids:
            if cid in attrs and cid in imgs:
                path = imgs[cid]["e2" if "e2" in imgs[cid] else "e0"][1]
                n += (hashlib.sha256((ASSETS / path).read_bytes()).hexdigest()[:16] + "-" + TILE_HASH) not in rows
        print(f"design_tiles: plan {n} images of {len(ids)} operators", flush=True)
        return
    with ThreadPoolExecutor(threads) as pool:
        for cid in ids:
            if cid not in attrs or cid not in imgs:
                print(f"skip {cid}: no attributes or no art", flush=True)
                continue
            art = "e2" if "e2" in imgs[cid] else "e0"
            sid, path = imgs[cid][art]
            key = hashlib.sha256((ASSETS / path).read_bytes()).hexdigest()[:16] + "-" + TILE_HASH
            if key in rows:
                n_cached += 1
                continue
            if deadline and time.time() > deadline:
                print(f"time budget of {a.minutes} min reached after {n_done}", flush=True)
                break
            race = attrs[cid].get("race") or "not stated"
            t0 = time.time()
            per = list(pool.map(ask, [(g, jpg, race) for g, jpg, _size in tiles(path, 2)]))
            dt = time.time() - t0
            rec = {"key": key, "char_id": cid, "operator": attrs[cid]["name"], "art": art, "id": sid,
                   "subjects": tile_aggregate([p or [] for p in per]), "bad_tiles": sum(p is None for p in per),
                   "seconds": round(dt, 2)}
            with open(out, "a") as f:
                f.write(json.dumps(rec, ensure_ascii=False) + "\n")
            rows[key] = rec
            n_done += 1
            print(f"{n_done} {rec['operator']} {art} {dt:.1f}s: " + "; ".join(
                f"{s['name']} ({s['kind']}, {'/'.join(s['where'])}, {s['tiles']} tiles)" for s in rec["subjects"]), flush=True)
    print(f"design_tiles: {n_done} images read, {n_cached} reused from {out.name}", flush=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["select", "caption", "text", "ocr", "stats", "units", "design", "design_tiles"])
    ap.add_argument("--split", default="", help="design: a split file (artifacts/answers/wiki-ref/split.json)")
    ap.add_argument("--split-part", default="dev", help="design: comma-separated parts of --split to read")
    ap.add_argument("--kind", default="e2", choices=["e2", "skin", "all"])
    ap.add_argument("--n", type=int, default=0, help="seeded sample size; 0 = every image")
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--ids", default="", help="comma-separated skin ids to caption instead of a sample")
    ap.add_argument("--first", default="", help="comma-separated skin ids captioned before the sample (acceptance cases)")
    ap.add_argument("--minutes", type=float, default=0, help="stop starting new images after this many minutes")
    a = ap.parse_args()
    {"select": select, "caption": caption, "text": text, "ocr": ocr, "stats": stats, "units": units, "design": design,
     "design_tiles": design_tiles}[a.cmd](a)


if __name__ == "__main__":
    main()
