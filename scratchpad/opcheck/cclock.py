"""CONTENT CLOCK: when was the content in a reference frame actually rendered?

The `REF_NEW` captures are 30 fps containers whose CONTENT advances irregularly. Measured over the
whole corpus, effective content rates run 17.05 fps (fugue) to 29.80 fps (cetnew), and the gap
histogram between updates is dominated by 1 with a minority of 2 and a tail of 3 to 6. ⛔ That
REFUTES a clean ratio: a 60 fps capture of a 30 fps render would put every gap at exactly 2. It is
irregular frame DROPPING under host load, the same cause already recorded for our own captures.

So a game frame can repeat the previous frame's pixels, and its content is then older than its
container timestamp. Scoring our render at container time against their older content charges us
for the recorder. This maps each beat to the time its game frame's content was last updated:

    content_time(i) = j / fps,  j = the largest index <= i whose frame differs from its predecessor

Prints one time per beat, for `mad.py --ourtimes=`. Where the frame is not a duplicate the value is
the beat itself, so a clean clip is a no-op and its score cannot move.

Usage: python3 cclock.py <ref.mp4> <b1,b2,...> [--fps=30] [--verbose]
"""

import subprocess
import sys

import numpy as np

DUP_PER_S = 15.0


def main():
    ref, times = sys.argv[1], sys.argv[2]
    fps = float(next((a.split("=", 1)[1] for a in sys.argv[3:] if a.startswith("--fps=")), 30))
    verbose = "--verbose" in sys.argv
    p = subprocess.run(
        ["ffprobe", "-v", "error", "-select_streams", "v:0",
         "-show_entries", "stream=width,height", "-of", "csv=p=0", ref],
        capture_output=True, text=True,
    ).stdout.strip().split(",")
    w, h = int(p[0]), int(p[1])
    raw = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", ref, "-f", "rawvideo", "-pix_fmt", "gray", "-"],
        capture_output=True,
    ).stdout
    n = len(raw) // (w * h)
    a = np.frombuffer(raw, np.uint8)[: n * w * h].reshape(n, h, w).astype(np.float32)
    d = np.abs(np.diff(a, axis=0)).mean(axis=(1, 2)) * fps
    # advanced[i] is True when frame i differs from frame i-1. Frame 0 has no predecessor and is
    # treated as an update, so the walk below always terminates.
    advanced = np.concatenate([[True], d >= DUP_PER_S])
    out = []
    for b in (float(x) for x in times.split(",")):
        gi = min(int(round(b * fps)), n - 1)
        j = gi
        while j > 0 and not advanced[j]:
            j -= 1
        out.append(j / fps)
        if verbose:
            print(f"  beat {b:6.2f}  game frame {gi:5d}  content frame {j:5d}  "
                  f"stale {(gi - j) / fps:+.4f}s", file=sys.stderr)
    print(",".join(f"{t:.4f}" for t in out))


if __name__ == "__main__":
    main()
