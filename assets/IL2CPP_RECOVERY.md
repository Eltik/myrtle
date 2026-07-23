# IL2CPP Camera-Follow Recovery — Runbook (macOS / Apple Silicon)

Recover the Arknights **dynchar entrance camera-follow** algorithm from the client binary.
Background: the follow is compiled `Torappu.*` C# — *an algorithm, not asset data* — and the
client's `global-metadata.dat` is **runtime-encrypted** (anti-tamper), so a static file dump
fails. You must dump **decrypted** metadata from a **running** client's memory, then decompile
one method. See `docs/DYNCHAR_ENTRANCE_INVESTIGATION.md` §5 for why nothing else works.

This runbook targets **Arknights EN** (`com.YoStarEN.Arknights`, Unity 2021.3, IL2CPP metadata
v27) and runs entirely on an Apple-Silicon Mac.

Two tools in this repo bracket the manual work:
- `assets/downloader` → `client-extract` — triage a package, confirm encryption, pull the binary.
- `assets/il2cpp-locate` — find the exact method + Ghidra/IDA offset in the dump.

```
obtain client → client-extract (triage) → RUN client + memory-dump (Frida) → Il2CppDumper
   → il2cpp-locate dump.cs → Ghidra decompile the offset → port the algorithm
```

---

## Automated driver — `assets/il2cpp-recover.sh`

A staged bash driver wraps the mechanical parts of the runbook below (macOS / Apple Silicon).
It does **not** replace the manual steps — it automates everything *around* the one wall that
can't be scripted (the live dump) and is loud about where that wall is. All artifacts land in
`$WORK` (default `assets/il2cpp-work/`, gitignored). Config is overridable by env:
`PKG`, `WORK`, `AVD_NAME`, `SYS_IMAGE`, `IL2CPPDUMPER`, `DUMP_JS`.

```
./assets/il2cpp-recover.sh <stage>
```

| Stage | What it does | Automation level |
|-------|--------------|------------------|
| `check`    | Report ✓/✗ install status of every tool (never fails). | full |
| `setup`    | Idempotently install apkeep, adb + emulator SDK + the arm64 **google_apis** image, dotnet, frida-tools. | full |
| `download` | `apkeep -d apk-pure` the EN package into `$WORK/apk`. | full |
| `extract`  | Run the in-repo `client-extract`; surfaces the `LIKELY ENCRYPTED` verdict that confirms a live dump is required. | full |
| `avd`      | Create + boot a rooted **google_apis** arm64 AVD (`-writable-system`), wait for boot. | **best-effort** |
| `dump`     | `adb root`, push a version-matched arm64 `frida-server`, install the game, spawn it under `il2cpp_dump.js`, pull the decrypted artifacts. | **best-effort — may be BLOCKED by anti-tamper** |
| `analyze`  | Run desktop Il2CppDumper if only a raw `.so` + decrypted metadata exist, then `il2cpp-locate` → ranked method + Ghidra offset. | full |
| `all`      | `download` → `extract`, then **STOP** and print instructions for the interactive `avd` + `dump`. | full (stops before the wall) |

**The honest ceiling.** `dump` is best-effort by nature: Arknights runtime-encrypts
`global-metadata.dat` and ships emulator/root/Frida detection, so the live dump may fail even
with every tool correct. No flag in this script removes that — the fallbacks are a **physical
rooted device** (Magisk DenyList + Shamiko to hide root, Zygisk on; the Zygisk-Il2CppDumper
module is often more robust than the Frida script) or a **jailbroken iOS device** (Path B).
`IL2CPPDUMPER` (path to `Il2CppDumper.dll`) and `DUMP_JS` (path to `il2cpp_dump.js`) must be
supplied by the user — the script points to their GitHub releases when unset. Ghidra
decompilation of the located offset (Step E) is manual.

---

## Path A — Android Studio emulator (recommended)

Apple-Silicon Macs run **arm64 Android system images natively** (fast), and Arknights ships
arm64 libs. No jailbreak, no FairPlay. This is the route to prefer.

### A0. Prereqs
- Android Studio (includes the emulator + `adb`/`sdkmanager` under `~/Library/Android/sdk`).
- A **rootable** AVD: create one on a **"Google APIs"** image (NOT "Google Play") — only the
  Google-APIs/AOSP images allow `adb root`. Pick **arm64-v8a**, API 33/34.
- `frida` + `frida-tools` on the Mac: `pipx install frida-tools` (or `pip install`).

### A1. Get the APK and triage it
1. Download the **Arknights EN** arm64 APK/XAPK (APKPure / APKMirror — pick the arm64-v8a
   variant). This is a different channel from the game's asset CDN, so the repo downloader
   can't fetch it — grab it manually.
2. Triage it with our tool (confirms you'll need a memory dump, and pulls the `.so` for later
   offset cross-referencing):
   ```
   cd assets/downloader
   cargo run -- client-extract -i ~/Downloads/arknights-en.xapk -o ./il2cpp-input
   ```
   Expect: `global-metadata.dat … LIKELY ENCRYPTED — use the Frida memory-dump runbook`.
   That's the confirmation that A2+ (a live dump) is required.

### A2. Boot the emulator with root + Magisk/Zygisk
The blocker isn't the dump tool — it's that Arknights' anti-tamper decrypts metadata only at
runtime and may resist a plain emulator. Give yourself root + Zygisk so the on-device dumper
can read decrypted memory:
1. Launch the AVD (`emulator -avd <name> -writable-system`), then `adb root`.
2. Patch the AVD with Magisk so Zygisk modules work — use **rootAVD**
   (https://github.com/newbit1/rootAVD): it installs Magisk into the AVD's ramdisk. Enable
   **Zygisk** in the Magisk app afterward.
3. Install Arknights: `adb install-multiple *.apk` (or drag the APK onto the emulator).

### A3. Dump decrypted metadata from memory
Two options — the Zygisk module is the most reliable against the anti-tamper:

**A3a. Zygisk-Il2CppDumper (preferred).**
- Install the module (https://github.com/Perfare/Zygisk-Il2CppDumper) via Magisk → reboot.
- Launch Arknights, let it reach the title screen (metadata is decrypted by then).
- The module writes the dumped, **decrypted** artifacts to
  `/data/data/com.YoStarEN.Arknights/files/` — including `dump.cs`, a rebuilt `libil2cpp.so`,
  and `global-metadata.dat`. Pull them:
  ```
  adb shell "su -c 'cp /data/data/com.YoStarEN.Arknights/files/* /sdcard/il2cpp-dump/'"
  adb pull /sdcard/il2cpp-dump ./dump
  ```
  If it emits `dump.cs` directly, skip to **Step C**.

**A3b. Frida script (fallback).**
- Push arm64 `frida-server` matching your `frida` version, run it as root:
  ```
  adb push frida-server /data/local/tmp/ && adb shell "su -c 'chmod 755 /data/local/tmp/frida-server && /data/local/tmp/frida-server &'"
  ```
- Spawn the game under Il2CppDumper's dump script
  (`il2cpp_dump.js`, https://github.com/Perfare/Il2CppDumper):
  ```
  frida -U -f com.YoStarEN.Arknights -l il2cpp_dump.js --no-pause
  ```
  It writes `dump.cs` + a memory dump. Pull it. If you get only raw `libil2cpp.so` +
  decrypted `global-metadata.dat`, run desktop Il2CppDumper (**Step C**).

---

## Path B — Jailbroken iOS device over USB (fallback)

Only if your device is jailbreakable (tell me the model + iOS version and I'll confirm). This
uses the app already installed on the phone; a memory dump beats **both** FairPlay and the
anti-tamper at once.
1. Jailbreak (palera1n for checkm8 A9–A11; Dopamine for arm64e up to supported iOS 15–16.x).
2. Install `frida` from the JB (add https://build.frida.re to your package manager, install the
   `frida` package), connect over USB.
3. `frida -U -f com.YoStarEN.Arknights -l il2cpp_dump.js --no-pause` — same as A3b; the decrypted
   metadata is read straight from memory. Pull the `dump.cs`.

(A non-jailbroken iOS device cannot run Frida against the App Store binary and its Mach-O is
FairPlay-encrypted, so it's a dead end for this — the emulator path is easier than jailbreaking.)

---

## Step C — Il2CppDumper → dump.cs (only if you have raw binary + decrypted metadata)

If A3/B already produced `dump.cs`, skip this. Otherwise, on the Mac (Il2CppDumper is .NET;
`dotnet` runs on Apple Silicon):
```
dotnet Il2CppDumper.dll <dumped libil2cpp.so> <decrypted global-metadata.dat> ./out
```
→ `out/dump.cs` (+ `script.json`, DummyDll/). Use the **decrypted** metadata from the dump, not
the encrypted one from the APK.

---

## Step D — Locate the method

```
cd assets/il2cpp-locate
cargo run -- --input /path/to/dump.cs            # ranked table
cargo run -- --input /path/to/dump.cs --json     # machine-readable
```
It ranks candidates by `Torappu`/`CharWord`/`L2D`/`DynIllust`/`Entrance`/`Camera` naming **and**
boosts any class whose body references the rig anchors we already recovered (`Dummy002`,
`static_offset`, `start_animation_02`, `_mainCamera`, `charVoiceOffset`). The top hit is almost
certainly the entrance-camera updater. Each row prints a ready
`Ghidra: go to file offset 0x…` / `IDA: jump to VA 0x…`.

---

## Step E — Decompile and port

1. Open the dumped `libil2cpp.so` (or the Mach-O) in **Ghidra** (free), go to the offset from
   Step D, decompile that one method.
2. Read the algorithm — expect it to compute the frame centre / ortho size from the character's
   runtime bounds/bones during the reform (there is no per-skin data; it's pure math). Grab the
   whole call chain if it delegates.
3. Hand me the decompiled pseudocode. I'll port it into the renderer's entrance tick
   (`SceneIllust.tsx`, where the gamedata `entranceCamCenterCurve` is replayed) to replace the
   interim heuristic with the real algorithm.

---

## Notes & gotchas
- **The version isn't the blocker; the anti-tamper is.** metadata v27 is well within
  Il2CppDumper/Il2CppInspector support — the whole reason for the live dump is decryption.
- **Emulator/root detection.** If Arknights refuses to launch, hide root (Magisk DenyList /
  Shamiko) and ensure Zygisk is on. The game only needs to reach a screen where metadata is
  decrypted — a few seconds in.
- **Split APKs.** If `libil2cpp.so` isn't in the base APK, it's in `split_config.arm64_v8a.apk`;
  the Zygisk/Frida dump reads it from the running process regardless, so this only matters for
  the static `client-extract` cross-reference.
- **Client updates** shift every offset — re-run Steps A3→D after any game update; the tooling
  is built to be re-run.
