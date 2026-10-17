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
import argparse, base64, hashlib, io, json, os, random, subprocess, sys, time
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common

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
    common.write_jsonl(OUT / "manifest.jsonl", rows)
    n = {k: sum(r["kind"] == k for r in rows) for k in ("e2", "skin")}
    print(f"manifest: {n['e2']} E2 arts, {n['skin']} outfits; image missing: {missing}")


def manifest(kind):
    rows = [json.loads(l) for l in open(OUT / "manifest.jsonl")]
    return [r for r in rows if kind == "all" or r["kind"] == kind]


def composite(path):
    """The art cropped to its visible pixels (alpha above 8) and laid on mid gray, full resolution."""
    from PIL import Image
    im = Image.open(ASSETS / path).convert("RGBA")
    box = im.getchannel("A").point(lambda a: 255 if a > 8 else 0).getbbox()
    if box:
        im = im.crop(box)
    bg = Image.new("RGB", im.size, (128, 128, 128))
    bg.paste(im, mask=im.getchannel("A"))
    return bg


def jpeg(im):
    """The image shrunk to fit MAX_SIDE, as JPEG bytes (quality 90), and its size."""
    from PIL import Image
    im.thumbnail((MAX_SIDE, MAX_SIDE), Image.LANCZOS)
    buf = io.BytesIO()
    im.save(buf, "JPEG", quality=90)
    return buf.getvalue(), im.size


def prepare(path):
    return jpeg(composite(path))


def vision_body(jpg, text, max_tokens, name, schema, system=None):
    """A vision request: [system,] user = (image, text), greedy, the answer bound to a JSON schema."""
    user = {"role": "user", "content": [
        {"type": "image_url", "image_url": {"url": "data:image/jpeg;base64," + base64.b64encode(jpg).decode()}},
        {"type": "text", "text": text}]}
    return {"messages": ([{"role": "system", "content": system}] if system is not None else []) + [user],
            "temperature": 0, "max_tokens": max_tokens,
            "response_format": {"type": "json_schema", "json_schema": {"name": name, "schema": schema}}}


def post_or_exit(body, retry_label, exit_label):
    """post() up to three times, 5 s apart (one failed decode 500s the request); then stop the run."""
    for attempt in range(3):
        try:
            return post(SERVER + "/v1/chat/completions", body)
        except Exception as e:  # one failed decode 500s the request; retry
            print(f"retry {retry_label}: {e}", flush=True)
            time.sleep(5)
    sys.exit(f"three failed requests on {exit_label}; stopping (is the server up?)")


TEXT_USER = """This is a crop of a larger illustration. Transcribe every piece of written text you can actually read in it (words, numbers, letters, signs, labels, measurements), each exactly as written. Do not include text you cannot read clearly; do not guess. Return JSON {"visible_text": [...]}, an empty list when there is no legible text."""
TEXT_SCHEMA = {"type": "object", "properties": {"visible_text": {"type": "array", "items": {"type": "string", "maxLength": 120}, "maxItems": 24}},
               "required": ["visible_text"]}


def tiles(path, grid=2, overlap=0.15):
    """Full-resolution tiles of the alpha-cropped composite, each JPEG-encoded at up to MAX_SIDE."""
    bg = composite(path)
    W, H = bg.size
    tw, th = W / grid, H / grid
    out = []
    for gy in range(grid):
        for gx in range(grid):
            x0, y0 = max(0, gx * tw - overlap * tw), max(0, gy * th - overlap * th)
            x1, y1 = min(W, (gx + 1) * tw + overlap * tw), min(H, (gy + 1) * th + overlap * th)
            jpg, size = jpeg(bg.crop((int(x0), int(y0), int(x1), int(y1))))
            out.append(((gx, gy), jpg, size))
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
            body = vision_body(jpg, TEXT_USER, 600, "text", TEXT_SCHEMA)
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
        common.append_jsonl(out, {"id": r["id"], "kind": r["kind"], "operator": r["operator"], "skin_name": r["skin_name"],
                                  "grid": grid, "visible_text": union, "tiles": per, "seconds": round(dt, 2),
                                  "prompt": hashlib.sha256((TEXT_USER + json.dumps(TEXT_SCHEMA, sort_keys=True)).encode()).hexdigest()[:12],
                                  "source": "model-written transcription"})
        print(f"tiles {i + 1}/{len(todo)} {r['id']} {dt:.1f}s {union}", flush=True)


def post(url, body, timeout=600):
    return common.post(url, json.dumps(body).encode(), timeout)


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
        body = vision_body(jpg, USER, 700, "art", SCHEMA, system=SYSTEM)
        t0 = time.time()
        resp = post_or_exit(body, r['id'], r['id'])
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
        common.append_jsonl(out, rec)
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
    rows = list({r["id"]: r for r in common.read_jsonl(p)}.values())
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
    height = {r["charId"]: r.get("heightCm") for r in common.read_jsonl(attrs)}
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
    common.write_jsonl(OUT / "units.jsonl", out)
    base = (ROOT / "artifacts" / "sources" / "all_units.jsonl").read_text()
    with open(OUT / "p4_art_units.jsonl", "w") as f:
        f.write(base if base.endswith("\n") else base + "\n")
        for u in out:
            f.write(json.dumps(u, ensure_ascii=False) + "\n")
    print(f"{len(out)} art units ({sum(1 for u in out if 'Text visible' in u['text'])} with tiled text) of {len(man)} images "
          f"-> {OUT / 'units.jsonl'}, {OUT / 'p4_art_units.jsonl'}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["select", "caption", "text", "ocr", "stats", "units"])
    ap.add_argument("--kind", default="e2", choices=["e2", "skin", "all"])
    ap.add_argument("--n", type=int, default=0, help="seeded sample size; 0 = every image")
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--ids", default="", help="comma-separated skin ids to caption instead of a sample")
    ap.add_argument("--first", default="", help="comma-separated skin ids captioned before the sample (acceptance cases)")
    ap.add_argument("--minutes", type=float, default=0, help="stop starting new images after this many minutes")
    a = ap.parse_args()
    {"select": select, "caption": caption, "text": text, "ocr": ocr, "stats": stats, "units": units}[a.cmd](a)


if __name__ == "__main__":
    main()
