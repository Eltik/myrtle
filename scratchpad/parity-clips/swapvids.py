#!/usr/bin/env python3
"""Replace the artifact's embedded parity clips, IN CARD ORDER.

Cards are Mlynar / Virtuosa / Skadi / Muelsyse; only the first three are rebuilt here, so the
4th `data:video/mp4` src is left untouched. Verify order by the heading preceding each src —
they are matched positionally, not by name.
"""
import re, pathlib, base64, sys
SP = pathlib.Path(sys.argv[1])
html = SP / "entrance-parity.html"
h = html.read_text()
new = ["clip_mly.mp4", "clip_cel.mp4", "clip_ska.mp4"]
spans = [(m.start(), m.end()) for m in re.finditer(r'src="data:video/mp4;base64,[A-Za-z0-9+/=]+"', h)]
assert len(spans) >= len(new), f"expected >= {len(new)} videos, found {len(spans)}"
for i in range(len(new) - 1, -1, -1):                       # back-to-front: offsets stay valid
    data = (SP / "clips" / new[i]).read_bytes()
    b64 = base64.b64encode(data).decode()
    a, b = spans[i]
    h = h[:a] + f'src="data:video/mp4;base64,{b64}"' + h[b:]
    print(f"  card {i} <- {new[i]} ({len(data)//1024} KB)")
html.write_text(h)
print("  new size:", len(h))
