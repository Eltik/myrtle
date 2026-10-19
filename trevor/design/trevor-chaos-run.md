# Trevor chaos run: the step 8 runbook (2026-10-19)

Step 8 of `trevor-self-contained-deploy.md`: run the nightly update on a development machine against a scratch
layout, inject four failures (an unreachable source, a failed gate, a skipped table, a full disk) and check that each
leaves the last publish serving and posts exactly one Discord embed. The eight publish-path gaps found by the read of
2026-10-19 13:15 are fixed in the same commit as this doc; the last table maps each injection to the gap it proves.
Budget: under two hours, most of it the full builds (run 0, (c), (a) twice and (d) five times, about 9 builds).

## Layout and environment

Nothing here writes under the real `artifacts/`: `init` moves the scratch copy into a scratch release.

```sh
R=~/Documents/Coding/myrtle/trevor; C=~/trevor-chaos      # not /tmp: abspath resolves /tmp to /private/tmp
mkdir -p $C/trevor $C/home $C/tmpfs $C/assets/ArkAssets/en $C/assets/output/en/gamedata
for d in scripts prompts data deploy; do rsync -a $R/$d/ $C/trevor/$d/; done   # real copies (python resolves imports)
for d in target models eval; do ln -s $R/$d $C/trevor/$d; done                  # the built binaries, no cargo
du -sh $R/artifacts && rsync -a $R/artifacts/ $C/trevor/artifacts/              # a real copy
rsync -a $R/../assets/output/en/gamedata/excel/ $C/assets/output/en/gamedata/excel/
rsync -a $R/../assets/output/en/gamedata/story/ $C/assets/output/en/gamedata/story/   # gametext.py reads it (run log)
echo chaos-res-0 > $C/assets/ArkAssets/en/.version                               # fresh mtime: no watcher alert
```

The backend and Discord stub, one process on :3999 (`$C/stub.py`); it answers every POST with `{"active": true}` and
logs one line per request, with the embed title for `/hook`:

```python
import json, sys, http.server
class H(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        b = self.rfile.read(int(self.headers.get("content-length", 0)))
        t = json.loads(b)["embeds"][0]["title"] if self.path == "/hook" else f"{len(b)} B"
        open(sys.argv[1], "a").write(f"{self.path}\t{t}\n")
        self.send_response(200); self.send_header("content-length", "16"); self.end_headers()
        self.wfile.write(b'{"active": true}')
    def log_message(self, *a): pass
http.server.HTTPServer(("127.0.0.1", 3999), H).serve_forever()
```

```sh
python3 $C/stub.py $C/posts.log & echo $! > $C/stub.pid        # stop it at the end: kill $(cat $C/stub.pid)
export PATH=/opt/homebrew/bin:$PATH TREVOR_DIR=$C/trevor TREVOR_HOME=$C/home ASSETS_DIR=$C/assets \
  FORCE=1 SKIP_FETCH=1 EMBED=0 CANARY=0 TREVOR_WARM_TOP=0 WORKER_SETTLE=0 TREVOR_WORKER_KEY=x PRUNE=0 \
  TREVOR_LLM_LOCK=$C/llm.lock TREVOR_BACKEND=http://127.0.0.1:3999 DISCORD_ALERT_WEBHOOK=http://127.0.0.1:3999/hook \
  RUN_TAIL_DIR=$C/tmpfs                                           # the Mac has no /run/trevor or /dev/shm
U=$C/trevor/scripts/trevor-vps-update.sh
snap() { cat $C/home/state/active.json; echo; readlink $C/home/current; wc -l < $C/home/state/releases.jsonl
         ls $C/home/releases; tail -3 $C/posts.log; }
```

`/opt/homebrew/bin` first gives `flock`. PRUNE=0 keeps retention's own output out of the post log (step 9 has its
tests). `/bin/bash` on macOS is 3.2, where a shell error (an unbound variable) exits 0 through the EXIT trap; every
exit below comes from `on_error`'s explicit `exit`, which 3.2 keeps, but a 0 where a failure is expected means: read
run.log for "unbound variable". Each run's log: `ls -t $C/home/logs | head -1`.

## Run 0: the baseline

`bash $U init; bash $U daily; echo $?; snap`. Expect exit 0 and one post title, "Trevor update (daily)" (published
<v0>), after the chunk and activate POSTs. Record v0, `readlink home/current` (R0), releases.jsonl (1 line) and the
build time: it sets how many of the runs below fit the two hours. Every injection below checks, unless it says
otherwise: active.json still v0, `readlink home/current` still R0, releases.jsonl unchanged, exactly one new `/hook`
line with the title given.

## (b) Unreachable source

1. `env -u SKIP_FETCH TREVOR_BACKEND=http://127.0.0.1:9 bash $U daily`: exit 1, title "Trevor update failed:
   build-corpus --refresh against http://127.0.0.1:9", no `/api/trevor/versions` line.
2. Gap 3: the same with `TREVOR_BACKEND=http://10.255.255.1` under `time`: the same title, done within 15 s
   (`--connect-timeout 10`); before the fix curl waited for the OS connect timeout.
3. Gap 3, the watcher: `touch -t 202609010000 $C/assets/ArkAssets/en/.version; FORCE=0 bash $U daily` twice: exit 0
   both times, "up to date" both times, title "Trevor: the asset watcher has not updated in N days" on the first run
   only (`home/state/watcher-alerted.json` holds the mtime). Then `echo chaos-res-0 > .../.version` (fresh mtime).

## (c) Failed gate

Copy `home/state/versions/<v0>/activate.json` aside, set its `gateReport.gates.recall.value` to 0.999 (gates.py's
previous recall), run `daily`: exit 1, title "Trevor update failed: gates", no versions POST, and
`home/stage/trevor/artifacts/panels/gate_report.json` says `"passed": false`. Restore the copy. The alternative,
`GATES_ARGS="--goldset eval/nope.jsonl"`, gives the same title with exit 1 or 2 (gates.py: 2 = could not run).

## (a) Skipped table

1. Write `$C/assets/output/en/gamedata/.skipped.json` as `{"tables": [{"table": "activity_table", "since":
   "2026-10-19", "resVersion": "chaos-res-a", "previousOutput": true}]}` and `echo chaos-res-a > .../.version`.
   `daily`: exit 0, title "Trevor: a game table is stale", log "stale: en/activity_table stale since 2026-10-19".
   Gap 4: the build makes v0 again, so the log says "already active; nothing to publish" and then "active.json:
   builtFrom chaos-res-0 -> chaos-res-a"; active.json keeps version v0 with the new builtFrom.
2. `daily` again: exit 0, no new post, the stale line still logged.
3. The missing variant: move `excel/activity_table.json` aside, `daily`: exit 0, "keeping last" for
   `main_release.json` (reading_guide.py dates) and the event units (gametext.py), no new post unless the content
   changed (then one "Trevor update (daily)"). Restore the table and remove `.skipped.json`.
4. Optional, gap 7: move `character_table.json` aside: deaths.py build logs "keeping last" and goes on. build-archives
   (Rust) and dossiers.py sets also read that table and have no guard: record where the run stops, if it does.

## (d) Disk

Each proxy needs a version that is not active, so pass `PANELS_ARGS="--version chaos-dN"`; restore every mode after.

1. `DISK_MIN_FREE_GB=100000 DISK_MIN_FREE_PCT=100 bash $U daily`: exit 0, title "Trevor: disk space is low", the run
   goes on (no publish: v0 is active).
2. `chmod -w $C/home/releases`, version chaos-d2: exit 1, title "Trevor update failed: promote the stage to a
   release", no `*.tmp` left, the versions POSTs only inactive ones. `chmod u+w`.
3. Gap 2: `chmod -w $C/home/state`, version chaos-d3: exit 1, the same title, `home/current` still R0 and
   `state/worker-restart` unchanged (its temp file could not be written, so nothing moved); releases.jsonl +1 (the
   promoted release stays a rollback candidate). `chmod u+w`. `chmod -w $C/home` fails the symlink itself the same way.
4. Gap 1: `chmod a-w $C/home/state/history.jsonl`, version chaos-d4: exit 5, title "Trevor update failed: activate
   chaos-d4", a versions POST for the activate, `home/current` MOVED to the chaos-d4 release (the backend activated
   it: no flip back), active.json still v0. `chmod u+w`, run the same again: exit 0, active.json chaos-d4, a second
   chaos-d4 release, history.jsonl +1: the record converges.
5. Gaps 5 and 6, the crash proxies: `rm $C/home/stage/.complete`, then `mkdir -p
   $C/home/releases/20990101T000000Z-x.tmp/artifacts`, `daily`: the log says "no completion marker: the last copy did
   not finish" (a fresh copy) and "removing the partial release ...x.tmp".
6. Gap 8 rides on every failure above: the embed's code block holds the last 20 lines from `$C/tmpfs`, and
   `ls $C/tmpfs` is empty after each run.
7. Optional, a real ENOSPC: `hdiutil create -size 4.5g -fs APFS -volname trevorchaos $C/disk.dmg; hdiutil attach
   $C/disk.dmg -mountpoint $C/vol`, then do this whole runbook with `TREVOR_HOME=$C/vol/home` from `init` on (the
   image must hold the release and the stage: twice `du -sh artifacts` plus 1 GB). Fill it with `dd if=/dev/zero
   of=$C/vol/fill bs=1m` until 200 MB are left, run `RESET_STAGE=1 bash $U daily`: exit 1, title "Trevor update failed:
   start" with a non-empty tail (from tmpfs, not the full disk). `rm $C/vol/fill`, run again: "no completion marker",
   exit 0. `hdiutil detach $C/vol` at the end.

If one build takes more than 10 minutes, run (d) 2 to 5 on a stub tree instead (binaries that `exit 0`, stage
scripts that `sys.exit(0)`, a `panels.py` stub writing a manifest and a passing gate report, the real publish.py and
common.py): the publish path is the same code, and each run takes about 1 s.

## What the run proves

| injection | expected | gap it closes |
|---|---|---|
| (b) 1 | one "update failed" embed, nothing published | the posture of section 7 |
| (b) 2, 3 | a dead host fails in 15 s; an idle watcher alerts once per crossing | 3: no curl timeout; "up to date" for ever |
| (c) | gate failure, no versions POST, stage kept with `passed: false` | the gates refuse a worse publish |
| (a) 1, 2 | one stale embed, none on the rerun; builtFrom recorded | 4: an unchanged build rebuilt nightly |
| (a) 3, 4 | missing tables keep the last output, exit 0 | 7: deaths.py, chrono.py events unguarded |
| (d) 2 | failed promote leaves no partial release | 6: a partial release took a prune slot |
| (d) 3 | `current` never moves without the restart marker | 2: the marker written after the flip |
| (d) 4 | exit 5, backend and worker agree, record converges next run | 1: the record failure flipped back (split brain) |
| (d) 5, 7 | an unfinished stage copy is never continued | 5: a half-copied stage reused |
| every failure | the alert quotes the tail from tmpfs | 8: the tail read from the full disk |

## Run log 2026-10-19 (partial)

Run 0, (b) 1 to 3, (c) both ways, and (a) 1 and 2, from 13:36 to 13:52 JST. Not run today: (a) 3 as written (see
mismatch 3), (a) 4 and all of (d).

Setup. `C` was the session scratchpad (`/private/tmp/claude-501/.../scratchpad/chaos`, already a realpath, on the same
APFS volume as the repo), not `~/trevor-chaos`. `artifacts/`, `excel/`, `story/` and `eval/` were copied with
`cp -cR` (clones, under 1 s); `eval/` was cloned rather than linked so the recall gate's run records stay out of the
real `eval/runs`; `target` and `models` were symlinks as above. Every command ran under `nice -n 19`. The stub ran as
PID 90189 and was killed at 13:56; `pgrep` for the stub, the script, the scratch binaries and llama-server then
matched nothing, and nothing listened on :3999. No model or llama-server started (EMBED=0, CANARY=0,
TREVOR_WARM_TOP=0; `answer_bank.py seed` and `merge` make no model call). `bash` here is `/bin/bash` 3.2.57 (no
Homebrew bash), so both `bash $U` and the script's `#!/usr/bin/env bash` re-exec ran 3.2. The volume had 60.5 GB free
of 994.6 GB (6.1%): above the 10 GB floor, so no disk embed is expected. A find for files newer than the stub under
the real `artifacts/` and `eval/` returned nothing.

| run | command (on top of the exported environment) | exit | time | new posts | invariants after | as expected |
|---|---|---|---|---|---|---|
| init | `bash $U init` | 0 | under 1 s | none | `current` -> `releases/20261019T043651Z-initial` | yes |
| dry | `PLAN=1 bash $U daily` | 0 | under 1 s | none | `state/` holds only `stage-base` and `update.lock` (from init) | yes |
| 0, first | `bash $U daily` | 3 | 33 s | "Trevor update failed: start", "Trevor: disk space is low" | no `active.json`, `current` on the initial release | NO: mismatches 1 and 2 |
| 0 | `RESET_STAGE=1 bash $U daily`, after the `story/` copy and 41c521d9 | 0 | 132 s | 8 `/api/trevor/versions` POSTs, then "Trevor update (daily)" | v0 `en-402153b596a4740c-p6ca7e346`, builtFrom chaos-res-0; R0 `releases/20261019T043939Z-en-402153b596a4740c-p6ca7e346/artifacts`; releases.jsonl 1, history.jsonl 1; gate report passed, recall 0.624682 | yes |
| (b) 1 | `env -u SKIP_FETCH TREVOR_BACKEND=http://127.0.0.1:9 bash $U daily` | 1 | 1 s | "Trevor update failed: build-corpus --refresh against http://127.0.0.1:9" | unchanged; no versions POST | yes |
| (b) 2 | the same with `http://10.255.255.1` | 1 | 11 s | the same title with that host | unchanged; no versions POST | yes (under 15 s) |
| (b) 3, first | `touch -t 202609010000 .version; FORCE=0 bash $U daily` | 0 | under 1 s | "Trevor: the asset watcher has not updated in 48 days" | unchanged; "up to date"; `watcher-alerted.json` mtime 1788188400 | yes |
| (b) 3, second | the same | 0 | under 1 s | none | unchanged; "up to date" | yes |
| (c) | recall value 0.999 in `state/versions/<v0>/activate.json`, `bash $U daily` | 1 | 127 s | "Trevor update failed: gates" | unchanged; no versions POST; stage `gate_report.json` passed false (recall 0.624682 against 0.999, tolerance 0.02); activate.json restored, `cmp` equal | yes |
| (c) alternative | `GATES_ARGS="--goldset eval/nope.jsonl" bash $U daily` | 1 | 127 s | "Trevor update failed: gates" | unchanged; no versions POST; gate report passed false, recall error "eval exited 1" | yes (1 of the allowed 1 or 2) |
| (a) 1 | `.skipped.json` as above, `.version` chaos-res-a, `bash $U daily` | 0 | 128 s | "Trevor: a game table is stale" | active.json v0 with builtFrom chaos-res-0 -> chaos-res-a; `current` R0; releases.jsonl 1, history.jsonl 1; logs "stale: en/activity_table stale since 2026-10-19" and "already active; nothing to publish" | yes |
| (a) 2 | `bash $U daily` again (FORCE=1) | 0 | 126 s | none | unchanged; the stale line logged again | yes |
| (a) 2, gap 4 | `FORCE=0 SETTLE_MIN=0 bash $U daily` (not in the runbook) | 0 | under 1 s | none | "up to date: EN res_version chaos-res-a is the active version's builtFrom" | yes: the same content is not rebuilt |

Every run left `$C/tmpfs` empty (gap 8's ring file removed). No run log holds "unbound variable", so no exit above is
bash 3.2's masked 0: the 3 and the 1s come from explicit exits.

What did not match:

1. The first run 0 exited 3 at `gametext.py`: the layout above copied `gamedata/excel` only, and gametext.py also
   reads `gamedata/story` (story intros, unfetched scripts, record notes). Its units fell from 1441 to 607,
   `write_keep_last` refused the write and the stage exited 3 (0f8dfa62). The guard worked: nothing was published.
   The layout block now copies `story/` too (76 MB; a clone with `cp -cR`).
2. The same run posted "Trevor update failed: start" and "Trevor: disk space is low" with 60.5 GB free, before any
   build step, and posted nothing for the real failure. This is the bash 3.2 caveat in a form the runbook's note
   does not cover: in `disk_check`, `low="$(... | grep 'WARNING: disk low' | sed ...)" || true` runs the inherited
   ERR trap inside the substitution when grep matches nothing (3.2 ignores the outer `|| true` there). The trap
   posted the failure with STEP "start", its log line landed in `$low` and posted the disk embed, and it created the
   run's alert marker (keyed on `$$`, the same in the subshell), so gametext's exit 3 later sent no alert. Every run
   on this Mac would have posted the two extra embeds and swallowed its own failure alert, so no row's one-post check
   could hold. Fixed in 41c521d9 (`|| true` inside the substitution, a test that fails with 3 posts against 1
   without it). Not checked on bash 5, which the VPS runs.
3. EN has no `excel/activity_table.json` on this machine (jp, kr and cn have one), so every run, run 0 included,
   already ran the (a) 3 missing variant: "dates: activity_table.json is missing from the game data; keeping the last
   main_release.json" and "gametext: activity_table.json is missing; kept 80 event-text units from the last build",
   exit 0. (a) 3 cannot be done as written ("move it aside"); run 0 covers it.
4. Timing: a full build takes 126 to 132 s here (EMBED=0, the stage copy about 1 s), so the 10-minute stub-tree
   fallback for (d) is not needed; (d)'s five builds fit in about 12 minutes.

State left for the next session: the stub stopped, `.skipped.json` removed, `activate.json` restored, `.version`
chaos-res-a (matching active.json's builtFrom), two releases (initial and R0), the stage continued from R0. The
scratch tree is 4.0 GB by `du` (clones count in full there; the unique blocks are fewer) and is kept for (d).
