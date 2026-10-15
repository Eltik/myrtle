# FlatBuffer schema store

This directory is the ONLY source of FlatBuffer schemas for the unpacker. `generate-fbs`
reads it and nothing else. It replaces two git submodules, `assets/OpenArknightsFBS` and
`assets/ArknightsFlatbuffers`, removed 2026-10-15.

```
assets/schemas/
  cn/*.fbs             the CN schemas, generated from the official CN client by
                       `unpacker schema-gen` and accepted by `schema-sync`
  yostar/*.fbs         one file per table whose live EN/JP/KR binary needs a layout
                       other than cn/; a table with no file here decodes with cn/
  history/<stem>/<label>.fbs
                       every distinct schema version of every table, newest label
                       last in lexical order; the candidates `schema-sync` searches
  roots.json           table file -> root type (and the key/value wrapper field for
                       SimpleKVTable tables); file names are not in the client, so this
                       map is kept by hand and `schema-gen --discover` lists table types
                       it does not cover yet
  schemas.lock.json    provenance: the CN client, server versions, per-table choice
                       and per-server verification verdicts
  README.md            this file
```

## Where schemas come from

Hypergryph ships gamedata as FlatBuffers with no public schema. The schemas are the
layouts of the client's C# classes (`Torappu.*`), which live in the client's IL2CPP
metadata. `unpacker schema-gen` reads them straight from the official CN Windows client:
`downloader -s cn client-download` asks Hypergryph's launcher API for the current client
version and downloads `GameAssembly.dll` (218,523,112 bytes at 2.7.81) and
`global-metadata.dat` (45,805,368 bytes) from the launcher CDN, per file and without
authentication; `schema-gen` parses the metadata (a v29 file; the CN build uses 92-byte
type rows instead of 88, which the reader detects), resolves field types through the
binary's metadata registration, applies the generation rules recorded in
`unpacker/src/schema/il2cpp/mod.rs` and `gen.rs`, and writes one `.fbs` per entry of
`roots.json` (table file -> root type; file names are not in the client). Nothing in
this path depends on a third party: no schema repository, no clone, no device, no running
game. The output format follows the conventions MooncellWiki established in
OpenArknightsFBS, against whose files the generator was validated (59 of 61 identical
after normalisation on 2026-10-15; `audio_data` differed by one property the reference
omits, and `sandbox_table`'s root type no longer exists in the client).

EN/JP/KR run older client builds than CN, so for some tables the Yostar binary matches an
OLDER CN schema. Yostar clients are only distributed through Google Play, so their
schemas are not generated; instead `schema-sync` searches `history/`, which holds every
schema version this store has ever had (seeded once from upstream's full history on
2026-10-15, grown by every regeneration since), and keeps the newest version the
production verifier accepts on the live binary. Candidates are judged with the SAME
verifier and the same options the production decoder uses, so a schema chosen here is a
schema `extract` will accept.

## history/

`history/<stem>/<label>.fbs` holds one file per DISTINCT version of each table's
schema, distinct after the normalisation `schema-sync` uses everywhere (comments and
whitespace stripped). A label is `<YYYY-MM-DD>_<version-or-sha7>`, and where one day
holds several versions of one table, `<YYYY-MM-DD>T<hhmmss>_<version-or-sha7>`, so
that lexical order is chronological and descending lexical order is newest first.
When two files normalise to the same text, the OLDER label is kept.

It was seeded ONCE on 2026-10-15 from a full-history clone of
[OpenArknightsFBS](https://github.com/MooncellWiki/OpenArknightsFBS) (all branches, `main`
`dbbc983`, `YoStar` `6c35802`) by a one-off script outside the repository: every blob
of `FBS/<stem>.fbs`, labelled with the committer date (committer time zone) of the
earliest commit that introduced it and that commit's `Update X.Y.Z` version, else its
7-hex sha. The clone held 1,072 distinct blobs, 1,111 (table, blob) pairs (some blobs
sit under several file names), 22,142,248 bytes over those pairs; normalisation
reduced them to 821 files over 61 tables, 17,510,315 bytes (1 version for
`char_master_table` up to 45 for `activity_table`). 67 files carry the time part. The
seed was the last time anything read that repository; the store is self-contained, and
nothing in `unpacker` runs git or reaches a network to sync schemas. Every `cn/` file
at the seed equals the newest version of its table; every `yostar/` file equals a
history version.

From then on `schema-sync --generated <dir>` adds each generated CN schema whose
normalised text is new as `<today UTC>_<client version>` (a same-day clash takes a
`_2`, `_3` suffix, which still sorts last). The Rust side is
`unpacker::schema::store::history` (`list` newest first, `add`, `find`, `dedupe`).

## The sync

```
unpacker schema-sync \
  --generated <dir>           `unpacker schema-gen` output: <stem>.fbs per table,
                              client.json, schema-gen.json. Without it CN is the
                              stored cn/ (a lock-only or Yostar-only run)
  --store assets/schemas      defaults to the directory holding this README
  --bundles cn=<dir> --bundles en=<dir> --bundles jp=<dir> --bundles kr=<dir>
                              gamedata-profile downloads (anon/*.ab + .idx); any subset
  --report <file.md>          the before/after matrix, written for the PR body
  --levels-sample 20          how many level buffers must verify for prts___levels
  --cache <dir>               binary-schema cache (default: unpacker-bfbs-cache in the
                              system temp directory)
  --self-check                instead of syncing, verify every buffer through the
                              generated decoder AND the reflection verifier and
                              print disagreements (exit 1 on any)
```

Exit codes: 0 resolved; 2 REGRESSION, a table and server pair that the previous lock
recorded as verified resolves under no candidate now (a server not supplied in a run never
counts); 1 error (including a `--generated` directory without `client.json`,
`schema-gen.json`, or any `.fbs`, or whose two `client_version` fields disagree). A
table that was already unresolved stays a warning, so one lagging server never blocks
a fix for the others.

**CN.** The tables are the stems of the generated directory plus those of `cn/`. For
each, the head candidate is the generated schema (source `client@<version>`), or the
stored `cn/` file for a stem the client no longer generates; history follows newest
first. The first candidate that verifies on the CN bundle is written to `cn/`. When
that is not the head, the table is a HISTORY FALLBACK: the lock's `note` and the
report say so. Because the stored `cn/` file was itself the newest verifying version,
the fallback normally lands on it, so a client whose schema does not verify leaves
`cn/` as it was. When nothing verifies, the stored file is kept unverified. A stem in
`cn/` that the generated directory lacks (on 2.7.81, `sandbox_table`, whose root type
the client no longer has) is NOT REGENERATED: kept as it is and listed in the report.
A stem only the generated directory has is added. Without a CN bundle, the stored
file stands (a generated schema reaches `cn/` only through CN verification), and a
new stem takes the generated schema unverified.

**Field type changes.** The verifier cannot see one class of generator error: an
existing field whose TYPE changed at the same size (`int` to `float`, an enum to `int`,
one table type to another). Such a schema verifies and decodes garbage silently;
measured 2026-10-15, a generated `item_table` with one `sortId: int` edited to `float`
verified on CN and was written to `cn/`. So with `--generated`, before anything is
written, the sync compares every generated stem against the stored `cn/<stem>.fbs`
(`schema/fields.rs`, a line-based reader of `name: type;` fields and `enum NAME : type`
underlying types) and lists each change as `stem: table.field: <old> -> <new>` or
`stem: enum NAME: <old> -> <new>`. Field additions and removals are normal patch
evolution and are not listed. The console always prints `field-type-changes: N`; when
N > 0 the report opens, right after the client line, with a `## Field type changes (N)`
section. The exit code stays 0, so the pipeline never stalls on it: the workflow reads
the report and withholds auto-merge on N > 0, leaving the PR for a human.

**Yostar.** For each table, if the CN choice verifies on every supplied Yostar server
the table needs no variant and any file in `yostar/` is removed. Otherwise the
candidates are tried in this order:

1. the CN choice;
2. the generated schema (it differs from the CN choice only after a CN fallback),
   source `client@<version>`;
3. every history version of the table, newest first;
4. the file already in `yostar/`, LAST, as the fallback of record.

A candidate whose normalised text an earlier one already had is skipped. The first
candidate verifying on every supplied Yostar server wins, otherwise the one verifying
on the most, EN breaking ties.

Newest first is deliberate. FlatBuffers is forward compatible: a buffer written with
appended trailing fields still verifies under an older schema that lacks them, so
verification cannot tell the older schema from the right one, and the NEWEST schema
that verifies is the one that decodes the most fields. The previous order put the
stored file first ("stability") and kept data out silently. Its measured symptom: EN
`item_table` decoded under the dead `YoStar` branch head (equal to `2025-06-05_2.5.80`)
carried 46 `UNKNOWN_85..92` enum leaves. Measured 2026-10-15 on the downloads listed
below, the switch moved 4 of the 8 variants to a newer version (`buff_table`,
`retro_table` 2.5.80 -> 2.7.51, `item_table` 2.5.80 -> 2.7.61, `building_data` 2.5.80
-> 2.7.31) and in all 12 EN/JP/KR outputs it only ADDED: every file grew, no key was
lost, every changed leaf was an `UNKNOWN_n` becoming its enum name, and U+FFFD and
`\u0000` stayed at 0. Whether a newer schema can verify and still misread a buffer is
not settled by the verifier; the gate for that is the extraction diff.

"Verifies" means the FlatBuffers verifier accepts the buffer under the candidate schema
with `verifier_opts()`: `max_alignment: 4` because Hypergryph 4-aligns 8-byte scalars,
`ignore_missing_null_terminator: false` because that flag is what separates EN
`roguelike_topic_table` and `building_data` from the CN schema (see
`unpacker/vendor/flatbuffers/MYRTLE-PATCH.md`). Candidates are compiled to binary schemas
with `flatc -b --schema` and verified through the vendored `flatbuffers-reflection`
crate, so no Rust is generated or compiled per candidate. The per-server verdicts land
in the lock, which is how a lagging server becomes visible instead of silent.

`prts___levels` (`level_data`) is one schema for ~2,900 buffers: the first
`--levels-sample` level files in sorted path order must all verify (sorted, not random,
so two runs on the same bundles agree).

## The lock

```json
{
  "client": {
    "version": "2.7.81",
    "generated_at": "<RFC3339>",
    "game_assembly_sha256": "<sha256 of GameAssembly.dll>",
    "metadata_sha256": "<sha256 of global-metadata.dat>"
  },
  "type_changes": [],
  "flatc": "25.12.19",
  "servers": { "cn": { "resVersion": "..." }, "en": {...}, "jp": {...}, "kr": {...} },
  "tables": {
    "item_table": {
      "cn":     { "source": "client@2.7.81", "sha256": "...", "verified": { "cn": true } },
      "yostar": { "source": "history/item_table/2026-08-01_2.7.61.fbs", "sha256": "...",
                  "verified": { "en": true, "jp": true, "kr": true } }
    },
    "character_table": {
      "cn":     { "source": "client@2.7.81", "sha256": "...", "verified": { "cn": true } },
      "yostar": { "source": "same-as-cn", "verified": { "en": true, "jp": true, "kr": true } }
    },
    "sandbox_table": {
      "cn":     { "source": "history/sandbox_table/2023-09-05_2.0.81.fbs", "sha256": "...",
                  "verified": {}, "note": "absent from the cn bundles" },
      "yostar": null
    }
  }
}
```

`client` comes from the generated directory's `client.json` (version, the two file
hashes) and `schema-gen.json`; it is `null` for a run without `--generated`.
`generated_at` is kept from the previous lock while the version and both hashes are
unchanged (else `schema-gen.json` `generated_at`, else `client.json` `downloaded_at`),
so a rerun on a re-downloaded copy of the same client leaves the lock byte-identical.
`type_changes` is the list the run printed as `field-type-changes` (empty when none).
A run without `--generated` keeps the previous array. A rerun on the same client finds
the stored file already equal to the generated one, so for such a stem the entries are
carried over from the previous lock while the client (version and both hashes) is
unchanged; a rerun on the same generated directory therefore reproduces the list.
`source` is `client@<version>` (the generated schema), `history/<stem>/<label>.fbs` (a
history version, also used for a stored file whose text a history version holds),
`same-as-cn` (Yostar only), `store` (a stored file no history version holds) or
`unresolved`. `sha256` is of the file as stored. An optional `note` explains an empty
`verified` (table absent from the supplied bundles, no CN bundle supplied, nothing
verifies) or a CN history fallback. A table with `"yostar": null` was present in no
Yostar bundle. A table with `"verified": {}` on the Yostar side resolved under no
candidate anywhere; the previous file (if any) is kept and `extract` keeps skipping it.
A run that changes nothing writes no file and leaves the lock byte-identical.

## Automation

`.github/workflows/schema-watch.yml` runs every six hours. It first checks, without
downloading anything, whether the launcher's CN client version or any server's
`resVersion` differs from the lock. If nothing changed and the run was not forced it stops
there. Otherwise, when the client moved, it downloads the client (about 264 MB) and runs
`schema-gen`; then it downloads the reachable servers' gamedata bundles, verifies them
with the pinned schemas, runs `schema-sync --generated`, `generate-fbs`, `cargo fmt`,
rebuilds, verifies again (a regression fails the job), smoke extracts every server, and
either commits a lock-only change to main or opens a PR with the matrix and squash-merges
it when Assets CI is green (kill switch: repository variable
`SCHEMA_WATCH_AUTOMERGE=false`). A server whose version endpoint does not answer is
skipped for that run with a warning. `schema-gen` exit 3 (the client changed shape) and
`schema-sync` exit 2 (a table lost every candidate) fail the job and leave the store at
its last state; both need a human.

Locally the same path is:

```bash
# the CN client, ~264 MB
assets/downloader/target/release/downloader -s cn client-download -d /tmp/ark/client --platform windows
cargo run --release --manifest-path assets/unpacker/Cargo.toml --bin unpacker -- schema-gen \
  --client /tmp/ark/client --out /tmp/ark/gen --roots assets/schemas/roots.json
# gamedata-only downloads, ~150 MB CN / ~110 MB EN
for s in cn en jp kr; do assets/downloader/target/release/downloader -s $s -d /tmp/ark/$s download --all --profile gamedata; done
cargo run --release --manifest-path assets/unpacker/Cargo.toml --bin unpacker -- schema-sync \
  --generated /tmp/ark/gen --bundles cn=/tmp/ark/cn --bundles en=/tmp/ark/en \
  --bundles jp=/tmp/ark/jp --bundles kr=/tmp/ark/kr --report /tmp/schema-report.md
cargo run --release --manifest-path assets/unpacker/Cargo.toml --bin generate-fbs
cargo fmt --manifest-path assets/unpacker/Cargo.toml
cargo build --release --manifest-path assets/unpacker/Cargo.toml --bin unpacker
```

To check the generator against the store without touching it:
`unpacker schema-gen --client /tmp/ark/client --out /tmp/ark/gen --compare assets/schemas/cn`.

## Hand patches: none, and why

`generate-fbs` once carried a `patch_schemas()` table of string replacements applied to
upstream schemas. Every patch it ever held was REFUTED on the live binary and the table is
gone. The record stays here because the next person to see a misaligned table will reach
for the same fix.

Measured on CN 26-09-03-04-06-00_ed95a2 against pristine OpenArknightsFBS `b24069f`
(2.7.71), generating twice and diffing the extraction:

- `activity_table`: the patch removed `defaultEnemyTag` from
  `clz_Torappu_ActivityEnemyDuelConstData`. The binary HAS the field (3 occurrences
  pristine, 0 patched). Patched output carried 167,669 escaped NUL bytes and was not valid
  UTF-8; pristine had zero. Patched verify FAILS (`u32 @2354602 unaligned`), pristine
  PASSES.
- `open_server_table`: the patch removed `compensateEndDay`. Patched
  `CheckInRewardDict` started with a valueless `{"key": 0}` and the file was not valid
  UTF-8. Patched verify FAILS (`u32 @56739 unaligned`), pristine PASSES.
- `display_meta_table`: the patch reverted `limitId`, `keyCodeType` and the dict-wrapped
  `timeLimitInfoList`. `KeyCodes` 4 occurrences patched vs 95 pristine, `LimitId` 0 vs 40,
  `KeyCodeType` 0 vs 95. Patched verify FAILS (`i64 @121364 unaligned`), pristine PASSES.
- `ep_breakbuff_table`: the patch removed `enemyElementBreakDuration`. Patched verify
  FAILS out of bounds (`Range [1092616552, 1092616556)`, a float read as an offset),
  pristine PASSES.
- `stage_table`: the patch truncated `clz_Torappu_CGGalleryGroupData` to two fields.
  Both verify on 2.7.71; the patch only discarded two real fields.
- `skin_table` (2.7.41, `6121fa9`): a patch removed `spAvatarId`/`spPortraitId`. Right at
  2.7.21, wrong once the binary shipped them: 18 fields instead of 20 shifted every slot
  from index 14, and `skin_table.json` became unparseable.
- `roguelike_topic_table` (2.7.71): a patch swapped `relicTipsData` and `activity`.
  Right for 2.7.41, which lacked `legacyItems`; on 2.7.71 it read the legacy-item vector
  through the RelicTips schema and the pre-rebase schema decoded `rogue_6` CharBuffData
  into 1.6 GB of vtable garbage.

The pattern is the same every time: a patch written against one binary outlives the
mismatch it corrected and then corrupts the decode itself. If a table verifies under no
schema, the fix is a schema that verifies, found by the sync above, never an edit. A
verifier failure alone does not prove a schema wrong either: before the `max_alignment`
patch, CN `activity_table` and `roguelike_topic_table` failed "unaligned" under schemas
whose decode was known good. What discriminates a good decode from a bad one is the
OUTPUT: valid UTF-8, no escaped NULs, size in line with the other server's.

## Measured 2026-10-15

Inputs: fresh gamedata-profile downloads, CN `26-10-08-04-51-28-56071834`, EN
`26-10-03-16-06-39_0a6acf`, JP `26-10-03-16-46-28_95dc60`, KR
`26-10-03-16-45-32_7cab75`; OpenArknightsFBS `main` `dbbc983` (2.7.81),
`YoStar` `6c35802`; flatc 25.12.19.

**The reflection verifier agrees with the generated decoder on every buffer.**
`unpacker schema-sync --self-check` verifies each gamedata FlatBuffer through
the generated code and through `flatbuffers-reflection` with the bfbs of the
same `.fbs`, both sides where a Yostar variant exists. Against the schemas the
committed decoder was generated from (the old submodules): CN 2,980 buffers,
3,011 verdicts (3,003 pass, 8 fail); EN, JP, KR 2,663 buffers and 2,694
verdicts each (2,686 pass, 8 fail). Disagreements: ZERO on all four, failing
verdicts included. Against the final store and routing: CN 2,989 verdicts, EN,
JP, KR 2,671 each, zero disagreements. The vendored reflection verifier needed
no patch; why is in `unpacker/vendor/flatbuffers/MYRTLE-PATCH.md`.

**The sync reproduces the upstream Yostar result on 35 of 57 tables and differs
on 22, all by design.** `cn/` is byte-identical to the old
`OpenArknightsFBS/FBS` (`diff -r` empty, 61 files). `yostar/` holds 8 files:
`buff_table`, `building_data`, `item_table`, `retro_table` from the `YoStar`
head `6c35802`; `activity_table`, `display_meta_table`, `stage_table` from
`b251597` (2.7.51); `roguelike_topic_table` from `e671930` (2.7.11). All 8 are
the same schema as the old ArknightsFlatbuffers file after normalisation, and
all 8 verify on EN, JP and KR. The 22 that differ (`audio_data`,
`campaign_table`, `char_meta_table`, `charm_table`, `charword_table`,
`checkin_table`, `climb_tower_table`, `cooperate_battle_table`,
`crisis_table`, `crisis_v2_table`, `enemy_database`, `gacha_table`,
`handbook_info_table`, `medal_table`, `replicate_table`,
`sandbox_perm_table`, `skill_table`, `story_review_meta_table`,
`story_review_table`, `story_table`, `uniequip_table`, `zone_table`) share one
reason: the old file is the `YoStar` head, which verifies on EN, JP and KR, and
the CN `main` head ALSO verifies on all three. Upstream tries the `YoStar` head
first; this sync drops the variant once the CN choice verifies everywhere. The
old binary's `verify` already chose CN for all 22 on all three servers, because
the decoder tries CN first, so those 22 variants were compiled and never used.
The other 27 of the 35 matches are tables whose old Yostar file already equalled
CN. 4 tables have no Yostar entry at all: `arkvent_table` (CN only),
`bake_muzzle_data`, `resource_manifest` and `sandbox_table` (no FlatBuffer of
theirs in any of the four downloads, so `"yostar": null` and an empty CN
`verified` with a note). A second run on the same inputs writes 0 files and
leaves the lock byte-identical. One sync over four servers takes 0.96 to 1.97 s
wall with a warm bfbs cache.

**`generate-fbs` from the store changes no extracted byte.** The CN modules,
`fb_json_auto.rs` and the 8 kept Yostar modules regenerate byte-identical. The
22 dropped Yostar modules go (179,458 lines), `fb_json_auto_yostar.rs` loses
25,758 lines and gains none (`diff -d`; git's numstat shows +34,060 / -59,818
from diff alignment), and `flatbuffers_decode.rs` loses its `verifier_opts`
copy for a `use` of `unpacker::schema::opts` plus the 22 Yostar arms. `unpacker
verify` with the new binary: the CN verdict and the `chosen` column are
identical to the old binary's for all 55 schema types on all four servers;
fixed 0, regressed 0; 22 rows per server change only their Yostar column from
PASS to n/a. `extract --gamedata` of all four servers diffs EMPTY against the
old binary's (12,631 CN files, 8,880 each EN, JP, KR).

**Deriving the routing fixes two tables and moves nothing else.** The schema
map and `guess_root_type` are now generated from the store (longest contained
stem wins, then two aliases, then the `level_` prefix). Against the phase above,
`diff -rq` of all four extractions differs in exactly two files.
`legion_mode_buff_table` was matched by the `buff_table` arm and decoded under
that schema (9,052 bytes of `Buffs`); it now decodes under its own (CN 45,846,
EN 46,011, JP 46,349, KR 46,104 bytes of `Legion_mode_buffs`). CN
`arkvent_table` had no arm and was written as a 112,104-byte string listing; it
now decodes to 1,012,080 bytes (`ArkventDataMap`, `OdcDataMap`). `main_text` now
routes to `main_text.fbs` instead of `init_text.fbs`; the two schemas differ
only in the stem and its output is byte-identical. `verify` now labels levels
`prts___levels` and the language tables `init_text` / `main_text` (formerly
`level_data` and `language_data`). CN reads 58 schema types, 2,981 buffers, 0
unrecognised (was 55, 2,980, 1).

Cost: a clean `cargo build --release --bin unpacker` (separate fresh target
directories, run one after the other, one run each, so noise is unmeasured)
takes 385 s for the old tree and 217 s for the new one; the binary goes from
41,558,656 to 38,981,504 bytes (-2,577,152). The vendored reflection crate is
the only new dependency besides `sha2`. `--self-check` over all four servers
takes 7.48 s.

Ruled out by measurement: a reflection-verifier alignment patch (zero
disagreements without one, every alignment check already goes through the
patched `Verifier`); upstream's `YoStar`-head-first order (it kept 22 variants
the decoder never selected); a hand-maintained table map (it misrouted
`legion_mode_buff_table` and could not see `arkvent_table`).

Still open, and what would settle each. Verification cannot tell a schema
missing APPENDED trailing fields from the right one, so candidate 1 (stability)
can keep an older Yostar file after a newer blob also verifies; diffing the
extraction under both would show the lost fields. The CN history fallback did
not run (`main` head verified all 58 CN tables). The regression exit (2) was
exercised only on a synthetic upstream. The `char_table` and `language_data`
aliases matched no filename in these downloads. `bakemuzzledata/` payloads are
not recognised as FlatBuffers by `flatbuffer_payload`, so `bake_muzzle_data`
is untested.

## Measured 2026-10-15, second pass: no upstream in the loop

**History seed.** Upstream's full history held 1,072 distinct blobs (1,111 table and
blob pairs, 22,142,248 bytes). After normalisation `history/` holds 821 files over 61
tables, 17,510,315 bytes, from 1 version (`char_master_table`) to 45 (`activity_table`).
Every `cn/` file equals the newest history version of its table. Nothing reads the
upstream clone after the seed.

**Newest-verifying order, measured on the same four downloads as above.** `cn/`
byte-identical (61 files). 4 of the 8 Yostar variants moved to a newer layout:
`buff_table` and `retro_table` 2.5.80 -> 2.7.51, `item_table` 2.5.80 -> 2.7.61,
`building_data` 2.5.80 -> 2.7.31; `activity_table`, `display_meta_table`, `stage_table`
(2.7.51) and `roguelike_topic_table` (2.7.11) stay because no newer version verifies.
`verify`: 58/57/57/57 rows identical before and after, 0 fixed, 0 regressed. Extraction:
CN diff empty (12,631 files); EN, JP, KR differ in exactly the 4 moved tables each, and
every change is a gain: EN `item_table` +79,292 bytes with `UNKNOWN_` enum leaves 46 -> 0
and 3,070 `StageDropList[].SortId` gained; `buff_table` +5,188 bytes, 4 -> 0 unknown
leaves; `retro_table` +61,933 bytes, 21 -> 11 (the 11 left are the same values before
and after, 1023, 58, 581, 634, probably flag combinations); `building_data` +491 bytes.
No key lost, U+FFFD and `\u0000` 0 before and after. Sync 1.03 s; a second run writes 0
files. This is the measurement that retired the stability-first order: a verifying
candidate that is NEWER carries fields the older one lacks, and both verify.
