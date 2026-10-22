# myrtle-storage

S3-compatible object storage (Cloudflare R2 first) for Myrtle's served asset
roots, and the `asset-sync` binary that mirrors them. This file is the
contract shared by the backend, the frontend URL helper and the asset
pipeline; it describes what this crate actually does.

## Why

`assets/output/{server}/` (EN 29 GB, 200,415 files; CN 33 GB) is produced by
the unpacker on the VPS and served by the axum handlers in
`backend/src/app/routes/assets.rs`. The backend also reads that tree for data
(gamedata, indexes built by walking `textures/`, `portraits/`, `spine/`,
`audio/`) and writes `derived/`. So:

* The local tree stays the source of truth and the backend keeps reading it.
* The object store is the delivery tier for the browser-served roots:
  `textures, spine, audio, video, portraits`.
* `asset-sync` mirrors those roots into the bucket after every extract and
  deletes what the orphan sweep deleted. Every file is rewritten on every
  extract, so mtime is useless; the diff is content-hashed (MD5, which equals
  the S3/R2 ETag of a single-part upload).
* The backend answers a request for a synced path by redirecting (or
  proxying) to the object; everything else (derived thumbs, `?format=png`
  conversions, gamedata) stays local and unchanged.
* With `S3_TYPE` unset, every byte served and every URL built is identical to
  serving without this crate. That is the kill switch.

## Env vars

Home: `backend/.env` or the repo root `.env`. The backend loads
`backend/.env` first and `../.env` (the root file, by exact path, no ancestor
search) second; dotenv never overrides a variable that is already set, so the
real environment beats `backend/.env`, which beats the root file. dotenv stops
at the first line it cannot parse (an unquoted value with a space) and drops
every key after it, so the backend logs a warning naming the file for any
error other than a missing file, and `asset-sync` exits 1 naming the file. **An empty value counts as unset** for every
variable (`S3_ROOTS=""` is the default roots, `S3_TYPE=""` is disabled).

| var | required | meaning / default |
|---|---|---|
| `S3_TYPE` | no | `cloudflare` (alias `r2`), `aws`, `s3` (generic: MinIO, Backblaze, Garage...), `local` (alias `fs`). Unset, empty, or `off` = storage disabled. Unknown value = startup error naming the accepted values. |
| `S3_BUCKET` | yes unless `local` | bucket name |
| `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY` | `cloudflare`, `s3`: yes. `aws`: optional (both or neither), falls back to the SDK default credential chain. | static credentials |
| `S3_ACCOUNT_ID` | `cloudflare`: yes unless an endpoint is given | derives endpoint `https://{account}.r2.cloudflarestorage.com` |
| `S3_ENDPOINT` | `s3`: yes. others: optional override | endpoint URL; normalized to `scheme://host[:port]`, so a trailing slash or a path (R2's per-bucket page appends `/<bucket>`) is dropped |
| `S3_REGION` | no | `cloudflare` -> `auto`, `aws` -> `us-east-1`, `s3` -> `us-east-1` |
| `S3_FORCE_PATH_STYLE` | no | `1`/`0`; default `s3` -> 1, `cloudflare` -> 0, `aws` -> 0 |
| `S3_PREFIX` | no | key prefix; leading/trailing slashes are trimmed, e.g. `myrtle` |
| `S3_PUBLIC_BASE_URL` | no (yes for `redirect`) | public base for GETs, e.g. `https://assets.eltik.net` (R2 custom domain / r2.dev / CloudFront). A trailing slash is trimmed. `S3_PUBLIC_URL` is accepted as an alias; the canonical name wins when both are set. |
| `S3_LOCAL_DIR` | `local`: yes | directory that stands in for the bucket |
| `S3_ROOTS` | no | comma list of top-level dirs under each server that live in the store. Default `textures,spine,audio,video,portraits`. |
| `S3_CACHE_CONTROL` | no | `Cache-Control` on every uploaded object. Default `public, max-age=604800`. |
| `S3_SYNC_CONCURRENCY` | no | concurrent uploads/deletes, default 16 |
| `S3_SYNC_HASH_THREADS` | no | concurrent hashing tasks, default = available CPUs |
| `S3_MULTIPART_THRESHOLD_BYTES` | no | default 268435456 (256 MiB); part size 64 MiB |
| `S3_EVICT_LOCAL` | no | `1`/`true`/`yes`/`on` (any case): `sync` truncates every file the bucket holds to a 0-byte stub (see "Freeing local disk"). Unset or any other value = off, and nothing is ever truncated. `--evict` / `--no-evict` override it. |
| `S3_EVICT_KEEP` | no | comma list of extra rel-path prefixes never evicted, added to the built-in keep list (which env cannot shrink). A leading `/` is trimmed; a backslash is a startup error. Default empty. |
| `ASSET_DELIVERY` | no | `local` \| `redirect` \| `proxy`. Default: `local` when storage is disabled (the variable is then not read at all); otherwise `redirect` when a public base is set, else `proxy`. `local` with storage enabled is allowed. `redirect` without a public base is a startup error. |

Cloudflare-native names, read only for `S3_TYPE=cloudflare` and only when the
generic name is unset:

| generic | Cloudflare fallback |
|---|---|
| `S3_ACCOUNT_ID` | `CLOUDFLARE_ACCOUNT_ID` |
| `S3_ENDPOINT` | `CLOUDFLARE_S3_API` |
| `S3_ACCESS_KEY_ID` | `CLOUDFLARE_API_ACCESS_KEY` |
| `S3_SECRET_ACCESS_KEY` | `CLOUDFLARE_API_SECRET_KEY` |

`CLOUDFLARE_API_TOKEN` is the dashboard API token, not an S3 credential, and is
never read. A non-positive number or an unparseable switch is a startup error
naming the variable.

Example (values quoted; dotenv strips the quotes, so the comma and space in
`Cache-Control` survive):

```
S3_TYPE="cloudflare"
S3_BUCKET="assets"
S3_PREFIX="myrtle"
S3_PUBLIC_BASE_URL="https://assets.eltik.net"
S3_CACHE_CONTROL="public, max-age=3600"
S3_ROOTS=""
CLOUDFLARE_ACCOUNT_ID="..."
CLOUDFLARE_API_ACCESS_KEY="..."
CLOUDFLARE_API_SECRET_KEY="..."
```

With these, the frontend's `VITE_ASSET_BASE_URL` is
`https://assets.eltik.net/myrtle`.

Provider client settings (in `providers/s3.rs`, not env):

* `cloudflare`: region `auto`; request checksum calculation and response
  checksum validation both `WhenRequired` (R2 rejects the SDK's default
  aws-chunked CRC32 trailers); never an ACL header; `Content-MD5` on
  single-part puts.
* `aws`: SDK defaults; `Content-MD5` set.
* `s3`: path-style by default, checksums `WhenRequired`, `Content-MD5` set.
* `local` (`providers/local.rs`): plain filesystem under `S3_LOCAL_DIR`, same
  key layout. ETag = MD5 of the file. A sidecar under `.meta/<key>.json` keeps
  the `Content-Type`, `Cache-Control` and ETag an upload set (so a multipart
  upload keeps its `-N` ETag); without it the type is guessed and the MD5
  computed on demand. This is the test double for the sync engine.

The store is constructed without I/O (`open_store`), so the backend builds it
synchronously at startup; the AWS default credential chain, when used, is
resolved on the first request.

## Key layout and URLs (`key.rs`)

* Object key = `[{S3_PREFIX}/]{server}/{rel}`, `rel` the path under
  `assets/output/{server}/` with forward slashes, raw (not percent-encoded):
  `myrtle/en/spine/DynIllust/char_1012_skadi2/[scene].json`.
* Public URL = `{S3_PUBLIC_BASE_URL}/{key with each segment percent-encoded}`
  with the `encodeURIComponent` set (`[`, `]`, space, `#`, `?`, `%`, `+`, `,`
  encoded; `-_.!~*'()` not).
* Frontend: `VITE_ASSET_BASE_URL` = `S3_PUBLIC_BASE_URL` + `/` + `S3_PREFIX`,
  no trailing slash. Unset = today's backend URLs.
* `rel` is normalized (leading `/` and `.` segments dropped); `..`, NUL and an
  empty path are rejected, matching the backend's `validate_asset_path`. Only
  a `rel` under one of `S3_ROOTS` is a store request.

## Backend delivery

Every former `serve_file` call site (generic route, indexed routes, story
routes) goes through `deliver_file(state, base_dir, rel, headers)`:

1. Storage disabled or `ASSET_DELIVERY=local`: `serve_file`, unchanged.
2. `rel` not under `S3_ROOTS` (or failing the lexical path rules):
   `serve_file`.
3. Otherwise the local file is the existence oracle (`validate_asset_path` +
   `tokio::fs::metadata`): missing is a 404 and the caller's fallback chain
   `server -> default -> cn` moves on exactly as before. Then:
   * `redirect`: `302 Found`, `Location` = public URL of the key, no body,
     `Cache-Control: public, max-age=3600`. An hour, not a week: the object
     keeps its own long cache, and a week-cached redirect would keep browsers
     on the bucket for a week after storage is switched off.
   * `proxy`: GET from the store forwarding `Range` and `If-None-Match`;
     replies with the store's status (200/206/304/416), `ETag`,
     `Content-Type`, `Content-Length`, `Content-Range`, `Accept-Ranges:
     bytes`, `Cache-Control: public, max-age=604800`; the body is streamed.
     Any store error, including a store 404 for a file that exists on disk
     (the sync has not reached it yet), logs at warn and falls back to
     `serve_file`, except when the local file is 0 bytes: it may be an
     evicted stub, so the store's error stands (404 for a missing object,
     503 otherwise) rather than a 200 with an empty body. `redirect` is
     unchanged, since a stub exists and the object holds the bytes.

The key's server segment is the last component of the tree the file was found
in, so a request that fell back to `cn/` redirects to the `cn/` key, and a
server reading the default server's art (`art_dir_for`) redirects to that
server's key.

Startup logs one line:
`asset delivery: <mode> (S3_TYPE=<kind>, bucket=<b>, prefix=<p>, public=<url|none>, roots=<..>)`.

## `asset-sync` CLI

```
asset-sync sync   --root <assets/output> --server <en|cn|jp|kr> [--roots a,b] [--no-delete]
                  [--dry-run] [--env-file <path>] [--concurrency N] [--hash-threads N]
                  [--allow-shrink] [--json] [--evict | --no-evict] [--force-evict]
asset-sync restore --root <..> --server <..> [--dry-run] [--only <rel-prefix>]... [--concurrency N]
                  [--json] [--env-file ..]                                          # stubs back from the store
asset-sync verify --root <..> --server <..> [--roots a,b] [--json] [--env-file ..]   # dry-run diff, exit 2 on drift
asset-sync ls     [--server <..>] [--prefix <..>] [--env-file ..]                     # size<TAB>etag<TAB>key
asset-sync cors   [--allow-origin <url>...] [--env-file ..]
```

`--env-file` is repeatable on every command.

`cors` sets one rule: GET and HEAD from the given origins (default `*` when
none is given), all request headers, `ETag`, `Content-Range`,
`Content-Length`, `Accept-Ranges` exposed, max-age 86400. `*` is the default
because a CORS fetch that the backend redirects to the bucket arrives with
`Origin: null` (the origin is tainted by the cross-origin redirect), which an
origin list rejects; the objects are public and fetched without credentials.

Env files: every `--env-file` is loaded in the order given (each must exist,
else exit 1). With none given, `.env` in the cwd then `../.env`, each if
present. dotenv never overrides a variable already set, so the first file to
set a variable wins and the real environment beats every file; passing
`--env-file ../backend/.env --env-file ../.env` from `assets/` reproduces what
the backend sees. Each file loaded is logged at info; a parse error is exit 1
naming the file. Storage disabled
prints `asset-sync: storage disabled (S3_TYPE unset), nothing to do` and
exits 0. Logs go to stderr (`RUST_LOG`, default `info`); the summary goes to
stdout.

`sync`:

* Lists the remote under `[{prefix}/]{server}/` (ListObjectsV2, paginated)
  into key -> (size, etag).
* Walks the local roots under `--root/<server>` (no symlinks followed or
  synced), skipping dot entries (files and directories), `*.tmp`, and files
  whose extension is not in the served set (`png jpg jpeg webp svg mp3 ogg wav
  m4a mp4 webm skel atlas json txt`, case-sensitive as in the backend). A root
  missing locally is logged and its remote keys are not deleted. A missing
  `--root/<server>` directory is exit 1.
* Per file: the MD5 is computed (`spawn_blocking`, bounded by
  `--hash-threads`). Remote key with the same size and an ETag equal to the
  MD5 => unchanged. An ETag containing `-` is multipart: unchanged only when
  the manifest (`<root>/<server>/.asset-sync.json`, key -> {size, md5, etag})
  has a record with the same size, the remote ETag and the local MD5; no
  record => re-upload. Anything else => upload.
* Upload: `Content-Type` from `mime_guess` with `.m4a` as `audio/mp4`,
  `Cache-Control` = `S3_CACHE_CONTROL`, `Content-MD5` on single-part puts,
  multipart at or above the threshold (64 MiB parts, aborted on failure).
  Uploads finish before any delete starts. Each object is retried 3 times
  (500 ms, 1 s, 2 s) on top of the SDK's retries; concurrency is bounded by
  a semaphore.
* Delete: remote keys under the walked roots with no local file, 1000 per
  `DeleteObjects`. Safety: when the local file count is below 90 % of the
  remote count under the walked roots, deletes are refused (exit 3) unless
  `--allow-shrink`; uploads still run. `--no-delete` skips deletes and the
  check. Keys outside the roots are never touched.
* Manifest (version 2): written at the end (temp file, fsync, rename), never
  in a dry run. Every key that ends the run unchanged or uploaded has a
  record `{size, md5, etag, evicted}` (md5 as hashed locally, etag from the
  listing or the put; `evicted` is left out when false, and a version 1 file
  loads with every record not evicted). Records under roots the run did not
  walk (a narrower `--roots`, a root missing locally) are carried forward.
  The manifest is correctness-relevant once anything is evicted: it is the
  only thing that tells a stub from an empty asset. An unreadable manifest
  therefore stops `sync` (exit 2) and `restore` (exit 1) instead of being treated as
  empty; repair it, or remove it only if nothing was ever evicted. An older
  binary ignores `evicted` and would upload stubs as empty objects: run
  `restore` before downgrading. The manifest also records the `bucket` and
  `prefix` its keys belong to (empty in files written before that). When any
  record is evicted and the recorded bucket or prefix (non-empty) differs
  from the current settings, or an evicted key is not under the current
  `[{prefix}/]{server}/`, `sync` (exit 2) and `restore` (exit 1) stop with a
  message naming both settings: restore under the old settings first.
* Lock: a writing `sync` and `restore` take `<root>/<server>/.asset-sync.lock`
  (an advisory `flock`, non-blocking) before reading the manifest and hold
  it past the final save, so two runs never overwrite each other's evicted
  marks. A held lock fails at once (`another asset-sync is running on this
  server: the lock <path> is held`; `sync` exit 2, `restore` exit 1). A dry
  run, `verify` and `ls` take no lock. The lock file is a dot file, never
  synced, and never removed.
* Output: progress at info every 1000 files, then one line:
  `asset-sync: server=en scanned=N unchanged=N uploaded=N (B bytes) deleted=N failed=N elapsed=S.s`
  with ` (dry run)` appended in a dry run (counts are then what would
  happen). With eviction on, ` evicted=N (B bytes freed)` sits between
  `deleted` and `failed`; with it off the line is unchanged. `--json` prints
  the report as one JSON object instead (`stubs`, `evict`, `evicted`,
  `freed_bytes`, `evict_failed` included).
* Exit: 0 all good, 1 usage or config (bad flags, bad env, missing env file,
  missing server dir, eviction refused by the delivery guard), 2 one or more
  objects failed or the remote could not be listed (and, for `verify`, any
  drift), 3 delete safety refusal (takes precedence over 2). A failed
  truncation never changes the exit code.

### Freeing local disk: stubs

With `S3_EVICT_LOCAL` on (or `--evict`), a file the bucket is confirmed to
hold (unchanged, or uploaded and the put returned) is TRUNCATED IN PLACE to
0 bytes: same path, same inode, the mtime it had (preserved), never unlinked. Its record gets
`evicted: true`. That file is a stub. It is not deleted because the backend
builds its indexes from the NAMES under the served roots and uses local
existence as the redirect oracle, `run.mjs` re-extracts when the file count
drops under 98 % of the last extract, and the unpacker skips re-encodes when
an output exists and is newer than its source; a 0-byte file satisfies all of
them, and the mtime the extract gave it is already newer than the source. The
mtime is kept rather than set to now because the orphan sweep
(`orphans.mjs` `subtreeWasWritten`) treats a fresh mtime as "this subtree was
written by the current extract", which during a filtered extract would sweep
every older file under it, stubs and keep-list files included.

* Keep list (never evicted; prefixes of the rel path, built in, not
  removable): `textures/avg/`, `textures/spritepack/cutin_char_`,
  `textures/spritepack/ui_home_act_banner_gacha_`,
  `textures/spritepack/ui_zone_home_theme_`,
  `textures/arts/ui/stage/[uc]homeentry/`, `audio/m4a_sources.json`, and
  any `textures/[[<tag>]]/arts/ui/homebanners/` (the `home_banner_gacha_hub`
  gacha banners; only `[[en]]` exists today). Each is read for its bytes by
  the backend (story sprites, release art) or the unpacker.
  `core/release/art.rs` also refuses to archive a 0-byte source, so a stub
  never blanks `derived/release-art`. `S3_EVICT_KEEP` adds prefixes.
* The manifest is written with every candidate marked evicted BEFORE any
  file is truncated, so an interrupted run never leaves an unmarked stub. A
  file that changed since it was hashed (an extract running alongside) is
  left alone. A truncation that fails is logged at warn, counted in
  `evict_failed`, unmarked, and does not change the exit code.
* A 0-byte file with an evicted record is a stub: when the listing has the
  key with the record's size and etag it counts as unchanged (and in
  `stubs`) with no hashing and no upload; when the key is gone the file
  fails with `evicted stub but the object is gone from the store;
  re-extract to recover`, and when the object differs from the record it
  fails too. Either failure keeps the record, so the stub never becomes an
  empty upload. A 0-byte file without an evicted record is an ordinary
  empty asset, uploaded as before.
* Delivery guard: eviction refuses to run (exit 1) when `ASSET_DELIVERY`
  from the same environment resolves to `local`, because the backend would
  then serve the stubs. `--force-evict` overrides the guard (it does not turn
  eviction on). `S3_TYPE=local` is allowed.
* `--dry-run` with eviction counts what would be evicted and freed and
  touches nothing.
* The peak disk use during an extract is not reduced: every extract rewrites
  the tree, and the next sync stubs it again.
* `verify` never evicts; a stub whose object matches its record is in sync.

`restore` is the way back, and must run before storage is switched off (or
`ASSET_DELIVERY` set to `local`): for every record marked evicted whose local
file exists with 0 bytes (narrowed by `--only <rel-prefix>`, repeatable), it
GETs the object into `<file>.<pid>.tmp`, checks the size and the MD5 against
the record, re-checks that the target is still 0 bytes, renames it over the
stub (then fsyncs the directory), and clears `evicted`. A target that gained
bytes meanwhile (an extract rewrote it) keeps them: the download is dropped
and counted in `skipped`. A record whose file is missing is not restored
(the orphan sweep deleted it on purpose; the next sync deletes the object):
it stays marked and is counted in `missing`.
Concurrency is `S3_SYNC_CONCURRENCY` (or `--concurrency`); each object is
retried 3 times except a 404. The manifest is saved once at the end, so a run
that dies part way is finished by running it again. It works with
`S3_EVICT_LOCAL` unset. Output:
`asset-sync: restore server=en restored=N (B bytes) failed=N elapsed=S.s`
with ` missing=N skipped=N` appended when either is above 0, and
` (dry run)` appended with `--dry-run`, which only counts; exit 0, or 1 on
any failure. At startup the backend reads every `<ASSETS_DIR>/<server>/`
manifest. With delivery `local` it logs one warning per server whose
manifest marks stubs:
`<N> evicted asset stubs under <server> but delivery is local; run: asset-sync restore --server <server>`.
Otherwise it warns once per server for stubs whose key is outside
`S3_ROOTS` or the current prefix (`Manifest::evicted_outside_roots`), since
those would be served empty from disk.

## Pipeline hook (owned by `assets/run.mjs`)

* After `runUnpack` (sweep included) and before the backoff clear, `.version`
  and `.last_extract` stamps, run
  `binaries/asset-sync sync --root <outputDir parent> --server <region> --env-file <WS_S3_ENV_FILE>`
  with `WS_S3_ENV_FILE` defaulting to `../backend/.env` when that file exists.
  `WS_S3_SYNC=0` skips the stage. A non-zero exit does not discard the
  extract: the stamps are still written, a `.sync-pending` marker
  (`{region, version, attempts, lastCode, lastSummary, at}`) is written beside
  them, `error` is broadcast with the summary, and `update_complete` is
  withheld. Every later tick (and `force_update`) first retries the sync
  alone while the marker exists; success removes the marker and broadcasts
  `update_complete` for the pending version.
* `node run.mjs sync --server <region>` (and `--dry-run`) runs the stage
  standalone, for the first seed and for manual repair.
* Build: `cargo build --release -p myrtle-storage --bin asset-sync` in
  `backend/`, installed to `assets/binaries/asset-sync`.

## Code

* `src/config.rs`: `StorageKind`, `StorageConfig::from_env` /
  `from_lookup`, `DeliveryMode::from_env` / `from_lookup`.
* `src/key.rs`: `normalize_rel`, `object_key`, `server_prefix`,
  `is_under_roots`, `public_url`.
* `src/store.rs`: the `ObjectStore` trait, `StorageError`,
  `content_type_for`, `md5_file`.
* `src/providers/s3.rs`, `src/providers/local.rs`: the two providers.
* `src/sync.rs`: `run_sync`, `SyncOptions`, `SyncReport`, `Manifest`
  (`evicted_count` included).
* `src/evict.rs`: `EvictPolicy` (`S3_EVICT_LOCAL`, `S3_EVICT_KEEP`),
  `BUILTIN_KEEP`, `truncate_in_place`.
* `src/restore.rs`: `run_restore`, `RestoreOptions`, `RestoreReport`.
* `src/bin/asset-sync.rs`: the CLI.
* Tests: unit tests per module, `tests/sync_local.rs` (the engine, eviction
  and restore against `LocalStore`), `tests/s3_wire.rs` (`S3Store` against a canned HTTP server:
  R2 put headers, GET 200/206/304/404 mapping).
