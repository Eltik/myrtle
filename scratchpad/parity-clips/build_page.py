#!/usr/bin/env python3
"""Build the parity artifact page with freshly-encoded clips inlined.

⚠️ Clips MUST be rebuilt with the per-skin reference TRIM OFFSET (see rebuild_all8.sh). A clip
built at off=0 shows a correct renderer as if it were out of sync — wis alone is six frames out.
"""
import base64, subprocess, sys
from pathlib import Path

HERE = Path(__file__).parent
EMBED = HERE / "embed11"

# key -> (display name, before, after, note)
SKINS = [
    ("wis",  "Wiš'adel",          33.649, 12.643, "Reference mis-trimmed by six frames."),
    ("eyja", "Eyjafjalla",        22.123, 10.408, "Reference mis-trimmed by three frames."),
    ("mue",  "Muelsyse",          21.918, 17.216, "Renderer fix: settled ground at the transform beat."),
    ("exc",  "Executor",          11.038,  8.919, "Reference mis-trimmed by three frames."),
    ("cet",  "Civilight Eterna",  18.087, 18.087, "Open — worst skin. Deficit localised to one corner."),
    ("mly",  "Mlynar",            18.073, 18.073, "Open — a blue cast, eight mechanisms refuted."),
    ("cel",  "Cello",             18.021, 18.021, "Open — twelve mechanisms refuted; considered exhausted."),
    ("ska",  "Skadi",             10.447, 10.447, "Stable."),
]

def vid(key):
    p = EMBED / f"{key}.mp4"
    probe = subprocess.run(["ffprobe","-v","error","-show_entries","format=duration","-of","csv=p=0",str(p)],
                           capture_output=True, text=True)
    if probe.returncode != 0 or not probe.stdout.strip():
        raise SystemExit(f"{p} does not decode")
    b = base64.b64encode(p.read_bytes()).decode("ascii")
    assert base64.b64decode(b) == p.read_bytes()
    return b

before_mean = sum(s[2] for s in SKINS) / len(SKINS)
after_mean  = sum(s[3] for s in SKINS) / len(SKINS)

rows = "\n".join(
    f'<tr class="{"win" if a < b else "flat"}"><td class="nm">{n}</td>'
    f'<td class="num">{b:.3f}</td><td class="num">{a:.3f}</td>'
    f'<td class="num delta">{"−" if a<b else ""}{abs(b-a):.3f}</td>'
    f'<td class="note">{note}</td></tr>'
    for k, n, b, a, note in SKINS)

cards = "\n".join(f"""
<figure class="card">
  <figcaption>
    <h3>{n}</h3>
    <p class="score"><span class="was">{b:.3f}</span><span class="arrow">→</span><span class="now">{a:.3f}</span></p>
    <p class="cnote">{note}</p>
  </figcaption>
  <video controls loop muted playsinline preload="metadata" src="data:video/mp4;base64,{vid(k)}"></video>
  <p class="legend">Top: our renderer · Bottom: the game capture</p>
</figure>""" for k, n, b, a, note in SKINS)

html = f"""<title>Dynamic Illustration — Entrance Parity</title>
<style>
:root {{
  --ink:#1b1d24; --ink-soft:#4a4f5e; --ink-faint:#767c8c;
  --paper:#f6f5f2; --card:#fffefc; --rule:#e2e0da;
  --accent:#1f5f8b; --accent-soft:#e8f0f6; --win:#2f6b4f; --flat:#8b8578;
  --mono:ui-monospace,SFMono-Regular,"SF Mono",Menlo,monospace;
  --serif:"Iowan Old Style","Palatino Linotype",Palatino,Georgia,serif;
  --sans:system-ui,-apple-system,"Segoe UI",Roboto,sans-serif;
}}
@media (prefers-color-scheme: dark) {{
  :root:not([data-theme="light"]) {{
    --ink:#e8e6e1; --ink-soft:#b0aca4; --ink-faint:#85817a;
    --paper:#15171c; --card:#1c1f26; --rule:#2c3038;
    --accent:#7fb8dd; --accent-soft:#1d2a35; --win:#7bc4a0; --flat:#7d7870;
  }}
}}
:root[data-theme="dark"] {{
  --ink:#e8e6e1; --ink-soft:#b0aca4; --ink-faint:#85817a;
  --paper:#15171c; --card:#1c1f26; --rule:#2c3038;
  --accent:#7fb8dd; --accent-soft:#1d2a35; --win:#7bc4a0; --flat:#7d7870;
}}
* {{ box-sizing:border-box; }}
body {{ margin:0; background:var(--paper); color:var(--ink); font-family:var(--sans);
  line-height:1.6; -webkit-font-smoothing:antialiased; }}
.wrap {{ max-width:78ch; margin:0 auto; padding:clamp(1.5rem,4vw,4rem) clamp(1rem,4vw,2rem) 6rem; }}
header {{ border-bottom:1px solid var(--rule); padding-bottom:2rem; margin-bottom:2.5rem; }}
.eyebrow {{ font:500 .75rem/1 var(--mono); letter-spacing:.14em; text-transform:uppercase;
  color:var(--accent); margin:0 0 1rem; }}
h1 {{ font-family:var(--serif); font-weight:600; font-size:clamp(2rem,5vw,3.1rem); line-height:1.1;
  margin:0 0 .75rem; text-wrap:balance; letter-spacing:-.01em; }}
.sub {{ color:var(--ink-soft); font-size:1.05rem; margin:0; max-width:60ch; }}
.headline {{ display:flex; flex-wrap:wrap; gap:2.5rem; margin:2.5rem 0 0; }}
.stat .k {{ font:500 .7rem/1 var(--mono); letter-spacing:.12em; text-transform:uppercase;
  color:var(--ink-faint); margin:0 0 .4rem; }}
.stat .v {{ font-family:var(--serif); font-size:2.4rem; line-height:1; margin:0;
  font-variant-numeric:tabular-nums; }}
.stat .v .to {{ color:var(--win); }}
.stat .v .from {{ color:var(--ink-faint); font-size:1.5rem; }}
h2 {{ font-family:var(--serif); font-size:1.6rem; margin:3.5rem 0 .5rem; font-weight:600; }}
h2 + .lede {{ color:var(--ink-soft); margin:0 0 1.5rem; }}
.tw {{ overflow-x:auto; border:1px solid var(--rule); border-radius:10px; background:var(--card); }}
table {{ border-collapse:collapse; width:100%; font-size:.92rem; }}
th, td {{ text-align:left; padding:.7rem .9rem; border-bottom:1px solid var(--rule); }}
thead th {{ font:600 .68rem/1.4 var(--mono); letter-spacing:.1em; text-transform:uppercase;
  color:var(--ink-faint); background:var(--accent-soft); }}
tbody tr:last-child td {{ border-bottom:0; }}
.num {{ font-family:var(--mono); font-variant-numeric:tabular-nums; text-align:right; white-space:nowrap; }}
.nm {{ font-weight:600; white-space:nowrap; }}
.delta {{ font-weight:600; }}
tr.win .delta {{ color:var(--win); }}
tr.flat .delta {{ color:var(--flat); }}
td.note {{ color:var(--ink-soft); font-size:.86rem; }}
.card {{ margin:0 0 2.5rem; background:var(--card); border:1px solid var(--rule); border-radius:12px;
  overflow:hidden; }}
.card figcaption {{ padding:1.1rem 1.2rem .8rem; }}
.card h3 {{ font-family:var(--serif); font-size:1.25rem; margin:0 0 .3rem; font-weight:600; }}
.score {{ margin:0 0 .35rem; font-family:var(--mono); font-variant-numeric:tabular-nums;
  display:flex; align-items:baseline; gap:.5rem; }}
.score .was {{ color:var(--ink-faint); text-decoration:line-through; }}
.score .arrow {{ color:var(--ink-faint); }}
.score .now {{ color:var(--win); font-weight:600; font-size:1.15rem; }}
.cnote {{ margin:0; color:var(--ink-soft); font-size:.88rem; }}
.card video {{ display:block; width:100%; background:#000; }}
.legend {{ margin:0; padding:.55rem 1.2rem .9rem; font:.75rem/1.4 var(--mono);
  color:var(--ink-faint); letter-spacing:.02em; }}
.finding {{ border-left:3px solid var(--accent); padding:.2rem 0 .2rem 1.1rem; margin:1.4rem 0; }}
.finding h3 {{ font-size:1rem; margin:0 0 .35rem; font-family:var(--sans); font-weight:650; }}
.finding p {{ margin:0; color:var(--ink-soft); font-size:.94rem; }}
code {{ font-family:var(--mono); font-size:.86em; background:var(--accent-soft);
  padding:.12em .38em; border-radius:4px; }}
ul {{ padding-left:1.2rem; color:var(--ink-soft); }}
li {{ margin:.45rem 0; }}
li strong {{ color:var(--ink); }}
footer {{ margin-top:4rem; padding-top:1.5rem; border-top:1px solid var(--rule);
  color:var(--ink-faint); font-size:.84rem; }}
</style>

<div class="wrap">
<header>
  <p class="eyebrow">Arknights · dynamic illustration · renderer parity</p>
  <h1>Entrance parity against the game capture</h1>
  <p class="sub">A PIXI renderer reproducing each skin's <code>_Start</code> cinematic, scored per
  frame against an Android capture. Lower is closer. The metric is MADC — mean absolute difference
  in luma plus half the mean chroma difference, over a fixed inner crop.</p>
  <div class="headline">
    <div class="stat"><p class="k">Corpus mean</p>
      <p class="v"><span class="from">{before_mean:.3f}</span> <span class="to">{after_mean:.3f}</span></p></div>
    <div class="stat"><p class="k">Skins measured</p><p class="v">8 <span class="from">of 82</span></p></div>
    <div class="stat"><p class="k">Of the 4 fixes</p><p class="v">3 <span class="from">changed no pixels</span></p></div>
  </div>
</header>

<h2>What moved, and why</h2>
<p class="lede">Four skins improved. Three of the four were not renderer bugs at all.</p>
<div class="tw"><table>
<thead><tr><th>Skin</th><th class="num">Before</th><th class="num">After</th><th class="num">Δ</th><th>Cause</th></tr></thead>
<tbody>
{rows}
</tbody></table></div>

<div class="finding">
  <h3>Most of the gain was a measurement error, not a rendering one</h3>
  <p>Each reference clip is trimmed by hand to a shared <code>t0</code>. Sweeping that trim per skin
  found <strong>six of the eight were cut on the wrong frame</strong> — Wiš'adel by six frames. The
  renderer was never wrong there; it was being compared against the wrong frames. Those fixes change
  no pixels. The two largest errors were both in the group an earlier flash-anchor check could not
  validate, so "the trims are verified" had never actually covered them.</p>
</div>

<div class="finding">
  <h3>Muelsyse was a real fix, and it needed both halves at once</h3>
  <p>Past her transform beat the game paints a flat near-white ground and stops showing the static
  art's vista behind the scene. We kept painting a blurred vista over 18.5% of the frame and fell
  through to a grey fill elsewhere. Removing the vista <em>alone</em> is worse (49.1 → 52.6, since
  the vacated region drops to grey); whitening the ground alone leaves the vista painted. Together:
  49.1 → 26.9 on her worst beat. That coupling is why four earlier single-cause attempts all came
  back refuted — each was scored against a frame where the other half still painted the same pixels.</p>
</div>

<h2>Side by side</h2>
<p class="lede">Our renderer on top, the game capture below. Each clip is aligned using that skin's
corrected trim offset — at the old offsets several of these read as out of sync when they are not.</p>
{cards}

<h2>Still open</h2>
<ul>
  <li><strong>Civilight Eterna (18.087)</strong> — now the worst. Her error is broad rather than one
    bad beat, but three of her four worst beats share a −20 to −40 luma deficit in the
    <em>same lower-left corner</em>. In that corner the game carries <strong>1.8× as many bright
    blobs, each about 0.45× the size</strong> — twice the bright area — while our brightest pixels
    already match exactly. Measured out: the terrain plane, the whole scene-layer set, the gap-fill
    backdrop, the HDR pass, all 132 particle systems, framing, gamma and bloom.</li>
  <li><strong>Mlynar (18.073)</strong> — a +1.98 Cb blue cast on his skin alone. Eight mechanisms
    refuted; two colour-space explanations fitted it to ~1% and both turned out wrong.</li>
  <li><strong>Cello (18.021)</strong> — twelve mechanisms refuted and considered exhausted.</li>
  <li><strong>Wiš'adel t=12</strong> — her only bad beat. The capture ramps a darkening where we
    step it on in a single frame.</li>
  <li><strong>The 74 skins with no capture</strong> — only 8 of 82 are measured at all. This is the
    largest remaining source of error and none of the numbers above speak to it.</li>
</ul>

<div class="finding">
  <h3>A broken diagnostic invented a phantom</h3>
  <p>One ablation switch was silently ineffective: the gap-fill sprite is registered for a per-frame
  coverage toggle that re-assigned its visibility every frame, overwriting the ablation one frame
  later. That manufactured an "unexplained base layer" that appeared to survive every ablation on two
  skins — it was the gap-fill wash the whole time, and a second switch had been removing it all
  along. Worth stating plainly because the wrong conclusion was reported before the tool was
  checked: <em>when an ablation shows no effect, verify the ablation before believing the result.</em></p>
</div>

<div class="finding">
  <h3>A clean population split is not evidence that acting on it helps</h3>
  <p>Two of Civilight Eterna's large background planes are dropped because their texture — a 1500²
  atlas of a cathedral, banners, swords and foliage — is classified as a flow/distortion map. The
  misclassification is real, and coverage separates the two populations with an empty gap (127
  textures below 0.85, <em>two</em> in between, 263 filled panels above 0.95). Restoring them still
  measures worse on three skins, including her own target beat. The change was reverted and the
  numbers kept.</p>
</div>

<footer>
  Scored with <code>all8.sh</code> over a fixed per-skin beat set, inner crop, 30 fps.
  Clips rebuilt at each skin's corrected trim offset.
</footer>
</div>
"""
out = HERE / "parity_page.html"
out.write_text(html, encoding="utf-8")
n = len(html.encode("utf-8"))
print(f"wrote {out} — {n/1e6:.2f} MB ({'OK' if n < 16e6 else 'OVER CAP'})")
