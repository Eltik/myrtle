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
