#!/usr/bin/env python3
"""Swap the artifact's per-skin parity clips for freshly-rendered ones.

Two traps this script exists to avoid, both of which have corrupted the file before:

  1. Replacements MUST be applied strictly back-to-front BY POSITION. Ordering them by
     payload size (or by any other key) invalidates every offset after the first splice
     and produces a file that no longer decodes.
  2. Every payload is validated as base64 AND as a decodable MP4 before the write, so a
     truncated clip can never reach the page.

Usage: refresh_artifact.py <artifact.html> <embed-dir>
The embed dir holds <key>.mp4 for each key in SKIN_ORDER; the Nth video element in the
page carrying a data: URI is matched to a key by VIDEO_INDEX below.
"""
import base64
import re
import subprocess
import sys
from pathlib import Path

# Which video element on the page belongs to which skin, in document order.
# Indices not listed are diagnostic clips that stay as they are.
VIDEO_INDEX = {0: "mly", 1: "cel", 2: "ska", 3: "mue", 7: "exc", 8: "cet", 9: "wis"}
# Superseded by the fresh captures above — dropped to stay under the 16 MB page cap.
DROP = {5, 6}

VID_RE = re.compile(r"data:video/mp4;base64,([A-Za-z0-9+/=]+)")


def encode(path: Path) -> str:
    raw = path.read_bytes()
    # Prove it decodes before it goes anywhere near the page.
    probe = subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", str(path)],
        capture_output=True, text=True,
    )
    if probe.returncode != 0 or not probe.stdout.strip():
        raise SystemExit(f"{path} does not decode: {probe.stderr.strip()}")
    b64 = base64.b64encode(raw).decode("ascii")
    assert base64.b64decode(b64) == raw
    return b64


def main() -> None:
    page, embed_dir = Path(sys.argv[1]), Path(sys.argv[2])
    html = page.read_text(encoding="utf-8")
    matches = list(VID_RE.finditer(html))
    print(f"page has {len(matches)} video payloads")

    # Build (start, end, replacement) triples, then sort by POSITION descending.
    edits = []
    for i, m in enumerate(matches):
        if i in DROP:
            # Blank the payload rather than removing the element: the surrounding markup
            # and prose still reference it, and an empty src degrades to a dead player
            # instead of malformed HTML. The element itself is stripped separately.
            edits.append((m.start(), m.end(), None))
        elif i in VIDEO_INDEX:
            key = VIDEO_INDEX[i]
            b64 = encode(embed_dir / f"{key}.mp4")
            old_kb, new_kb = len(m.group(1)) // 1024, len(b64) // 1024
            print(f"  [{i}] {key:5s} {old_kb:5d}KB -> {new_kb:5d}KB")
            edits.append((m.start(), m.end(), "data:video/mp4;base64," + b64))

    for start, end, rep in sorted(edits, key=lambda e: e[0], reverse=True):
        if rep is None:
            # Drop the whole <video ...> element this payload sits inside.
            open_tag = html.rfind("<video", 0, start)
            close = html.find(">", end)
            close = html.find("</video>", close)
            close = close + len("</video>") if close != -1 else html.find(">", end) + 1
            html = html[:open_tag] + '<p class="note">(superseded clip removed)</p>' + html[close:]
        else:
            html = html[:start] + rep + html[end:]

    page.write_text(html, encoding="utf-8")
    size = len(html.encode("utf-8"))
    print(f"wrote {page} — {size/1e6:.2f} MB ({'OK' if size < 16e6 else 'OVER CAP'})")

    # Re-validate: every remaining payload must still decode.
    bad = [i for i, m in enumerate(VID_RE.finditer(page.read_text(encoding='utf-8')))
           if len(m.group(1)) % 4 or not _decodes(m.group(1))]
    print("payloads failing validation:", bad or "none")


def _decodes(b64: str) -> bool:
    try:
        base64.b64decode(b64)
        return True
    except Exception:
        return False


if __name__ == "__main__":
    main()
