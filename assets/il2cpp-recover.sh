#!/usr/bin/env bash
#
# il2cpp-recover.sh — staged driver for the Arknights IL2CPP camera-follow recovery
# pipeline on macOS / Apple Silicon.
#
# THE PRIZE: the compiled `Torappu.*` dynchar entrance-camera method — an *algorithm,
# not asset data* — that frames the L2D entrance cinematic. See
# `docs/DYNCHAR_ENTRANCE_INVESTIGATION.md` §5 and `assets/IL2CPP_RECOVERY.md`.
#
# HONEST CEILING: Arknights RUNTIME-ENCRYPTS its global-metadata.dat (anti-tamper), so a
# static file dump ALWAYS fails — the decrypted metadata only exists in the memory of a
# LIVE running client. Dumping it needs a rooted Android emulator + Frida (or a jailbroken
# iOS device). Arknights ships strong emulator/root/Frida detection, so the `dump` stage is
# BEST-EFFORT and MAY BE BLOCKED. No script removes that wall. If the emulator route is
# refused, fall back to a PHYSICAL rooted device (Magisk DenyList + Shamiko to hide root) or
# a jailbroken iOS device. This driver automates everything up to and around that wall, and
# is loud about where the wall is.
#
# Fully-automatic : check, setup, download, extract, analyze
# Best-effort     : avd, dump      (subject to anti-tamper / detection)
# Manual handoff  : Ghidra decompile of the located offset (Step E in the runbook)

set -euo pipefail

# --- Configuration (override via environment) --------------------------------------------
# Absolute path to this script's directory (the repo's `assets/` dir).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Android package id for Arknights EN.
PKG="${PKG:-com.YoStarEN.Arknights}"

# Working directory for all downloaded / extracted / dumped artifacts (gitignored).
WORK="${WORK:-${SCRIPT_DIR}/il2cpp-work}"

# AVD name and system image. MUST be a "google_apis" (NOT "google_apis_playstore") arm64
# image so that `adb root` is permitted — Play images are locked down.
AVD_NAME="${AVD_NAME:-ak_il2cpp}"
SYS_IMAGE="${SYS_IMAGE:-system-images;android-34;google_apis;arm64-v8a}"

# Path to Il2CppDumper.dll (the .NET desktop dumper). Empty → the `analyze` stage explains
# how to obtain it. Download: https://github.com/Perfare/Il2CppDumper/releases
IL2CPPDUMPER="${IL2CPPDUMPER:-}"

# Path to the Frida spawn+dump script (Il2CppDumper's `il2cpp_dump.js`).
# https://github.com/Perfare/Il2CppDumper/blob/master/il2cpp_dump.js
DUMP_JS="${DUMP_JS:-${WORK}/il2cpp_dump.js}"

# Manifests for the two in-repo Rust tools this driver integrates.
CLIENT_EXTRACT_MANIFEST="${SCRIPT_DIR}/downloader/Cargo.toml"
IL2CPP_LOCATE_MANIFEST="${SCRIPT_DIR}/il2cpp-locate/Cargo.toml"

# --- Output helpers ----------------------------------------------------------------------
_bold=$'\033[1m'; _red=$'\033[31m'; _grn=$'\033[32m'; _ylw=$'\033[33m'; _rst=$'\033[0m'

info()  { printf '%s==>%s %s\n'  "${_bold}" "${_rst}" "$*"; }
ok()    { printf '%s  ✓%s %s\n'  "${_grn}"  "${_rst}" "$*"; }
warn()  { printf '%s  ⚠%s %s\n'  "${_ylw}"  "${_rst}" "$*" >&2; }
err()   { printf '%s  ✗%s %s\n'  "${_red}"  "${_rst}" "$*" >&2; }
die()   { err "$*"; exit 1; }

has() { command -v "$1" >/dev/null 2>&1; }

# --- Android SDK tool discovery ----------------------------------------------------------
# The SDK CLI tools (sdkmanager/avdmanager/emulator/adb) are not always on PATH after a
# Homebrew cask install; resolve them from the usual SDK roots too.
android_sdk_root() {
  if [[ -n "${ANDROID_SDK_ROOT:-}" ]]; then printf '%s\n' "${ANDROID_SDK_ROOT}"; return; fi
  if [[ -n "${ANDROID_HOME:-}" ]]; then printf '%s\n' "${ANDROID_HOME}"; return; fi
  printf '%s\n' "${HOME}/Library/Android/sdk"
}

# find_android_tool <name> — echo an absolute path to the tool, or empty if not found.
find_android_tool() {
  local name="$1" root; root="$(android_sdk_root)"
  if has "${name}"; then command -v "${name}"; return; fi
  local c bp; bp="$(brew --prefix 2>/dev/null || printf '/opt/homebrew')"
  for c in \
    "${root}/platform-tools/${name}" \
    "${root}/emulator/${name}" \
    "${root}/cmdline-tools/latest/bin/${name}" \
    "${root}/cmdline-tools/bin/${name}" \
    "${root}/tools/bin/${name}" \
    "${bp}/bin/${name}" \
    "${bp}/share/android-commandlinetools/cmdline-tools/latest/bin/${name}"; do
    if [[ -x "${c}" ]]; then printf '%s\n' "${c}"; return; fi
  done
  printf '%s\n' ""
}

require_tool() {
  # require_tool <cmd> <hint>
  has "$1" || die "missing '$1' — $2 (run: $0 setup)"
}

require_android_tool() {
  # require_android_tool <name> → echoes the resolved path, or dies.
  local p; p="$(find_android_tool "$1")"
  [[ -n "${p}" ]] || die "missing Android SDK tool '$1' — install it (run: $0 setup)"
  printf '%s\n' "${p}"
}

# --- usage -------------------------------------------------------------------------------
usage() {
  cat <<EOF
${_bold}il2cpp-recover.sh${_rst} — staged Arknights IL2CPP camera-follow recovery (macOS / Apple Silicon)

Usage: $0 <stage>

Stages:
  check      Report install status of every tool the pipeline needs (never fails).
  setup      Idempotently install the toolchain (apkeep, adb/emulator SDK, dotnet, frida).
  download   Fetch the Arknights EN package from APKPure via apkeep into \$WORK/apk.
  extract    Run the in-repo client-extract on the package; confirm metadata is encrypted.
  avd        [best-effort] Create + boot a rooted Google-APIs arm64 AVD.
  dump       [best-effort] Live memory-dump decrypted metadata via Frida (may be BLOCKED).
  analyze    Il2CppDumper (if needed) + il2cpp-locate → ranked method + Ghidra offset.
  all        download → extract, then STOP with instructions for the interactive avd+dump.

Config (env overrides): PKG, WORK, AVD_NAME, SYS_IMAGE, IL2CPPDUMPER, DUMP_JS
  PKG=${PKG}
  WORK=${WORK}
  AVD_NAME=${AVD_NAME}
  SYS_IMAGE=${SYS_IMAGE}
  IL2CPPDUMPER=${IL2CPPDUMPER:-<unset — see 'analyze'>}
  DUMP_JS=${DUMP_JS}

Fully-automatic: check, setup, download, extract, analyze
Best-effort    : avd, dump  (Arknights anti-tamper may block the live dump)
Manual         : Ghidra decompile of the located offset (runbook Step E)
EOF
}

# =========================================================================================
# Stage: check
# =========================================================================================
stage_check() {
  info "Tool status (✓ present / ✗ missing). Missing tools → run: $0 setup"
  local name path
  # Simple PATH tools.
  for name in apkeep dotnet frida frida-server python3 cargo brew xz; do
    if has "${name}"; then ok "${name} ($(command -v "${name}"))"; else err "${name}"; fi
  done
  # pipx is optional (frida fallback path).
  if has pipx; then ok "pipx ($(command -v pipx))"; else warn "pipx (optional; frida install falls back to pip)"; fi
  # Android SDK tools (may live under the SDK root, not on PATH).
  for name in adb emulator sdkmanager avdmanager; do
    path="$(find_android_tool "${name}")"
    if [[ -n "${path}" ]]; then ok "${name} (${path})"; else err "${name}"; fi
  done
  # In-repo Rust tools.
  if [[ -f "${CLIENT_EXTRACT_MANIFEST}" ]]; then ok "client-extract crate (${CLIENT_EXTRACT_MANIFEST})"; else err "client-extract crate not found"; fi
  if [[ -f "${IL2CPP_LOCATE_MANIFEST}"  ]]; then ok "il2cpp-locate crate (${IL2CPP_LOCATE_MANIFEST})";   else err "il2cpp-locate crate not found"; fi
  # Desktop dumper (only needed if the live dump yields raw so + metadata, not dump.cs).
  if [[ -n "${IL2CPPDUMPER}" && -f "${IL2CPPDUMPER}" ]]; then
    ok "Il2CppDumper.dll (${IL2CPPDUMPER})"
  else
    warn "Il2CppDumper.dll not set (IL2CPPDUMPER) — only needed by 'analyze' if the dump isn't already dump.cs"
  fi
  return 0
}

# =========================================================================================
# Stage: setup
# =========================================================================================
stage_setup() {
  info "Installing the toolchain (idempotent; each step is skipped if already present)."
  has brew  || die "Homebrew is required — install it from https://brew.sh first"
  has cargo || die "Rust/cargo is required — install it from https://rustup.rs first"

  # Each install is guarded so one failure never aborts the rest (set -e safe): a failed
  # optional tool only blocks the stage that needs it, not the whole toolchain.

  # apkeep — APK downloader (cargo). Needed by 'download'.
  if has apkeep; then ok "apkeep already installed — skipped"
  elif { info "cargo install apkeep"; cargo install apkeep; }; then ok "installed apkeep"
  else warn "apkeep install failed — the 'download' stage needs it"; fi

  # adb (platform-tools). Needed by 'avd'/'dump'.
  if [[ -n "$(find_android_tool adb)" ]]; then ok "adb already installed — skipped"
  elif { info "brew install --cask android-platform-tools"; brew install --cask android-platform-tools; }; then ok "installed android-platform-tools (adb)"
  else warn "adb install failed"; fi

  # sdkmanager (command-line tools) — the gateway to emulator + system images. The Homebrew
  # cask is 'android-commandlinetools' (no hyphen before 'tools').
  if [[ -n "$(find_android_tool sdkmanager)" ]]; then ok "sdkmanager already installed — skipped"
  elif { info "brew install --cask android-commandlinetools"; brew install --cask android-commandlinetools; }; then ok "installed android-commandlinetools (sdkmanager)"
  else warn "android-commandlinetools install failed — 'avd'/'dump' need it"; fi

  # emulator binary + the arm64 Google-APIs system image, installed into a CONSISTENT SDK root
  # (--sdk_root) so find_android_tool locates them afterward.
  local sdkmanager sdkroot; sdkmanager="$(find_android_tool sdkmanager)"; sdkroot="$(android_sdk_root)"
  if [[ -n "${sdkmanager}" ]]; then
    export ANDROID_SDK_ROOT="${sdkroot}"; mkdir -p "${sdkroot}"
    if [[ -z "$(find_android_tool emulator)" ]]; then
      info "sdkmanager 'emulator' 'platform-tools' → ${sdkroot}"
      yes 2>/dev/null | "${sdkmanager}" --sdk_root="${sdkroot}" "emulator" "platform-tools" || warn "sdkmanager emulator install returned nonzero — verify manually"
      ok "requested emulator + platform-tools"
    else
      ok "emulator already installed — skipped"
    fi
    info "sdkmanager '${SYS_IMAGE}' (accepting licenses) → ${sdkroot}"
    yes 2>/dev/null | "${sdkmanager}" --sdk_root="${sdkroot}" "${SYS_IMAGE}" || warn "system-image install returned nonzero — check the image name: ${SYS_IMAGE}"
    ok "requested system image ${SYS_IMAGE}"
  else
    warn "sdkmanager not found — install the Android command-line tools, then re-run setup"
  fi

  # dotnet — runs the desktop Il2CppDumper (.NET) on Apple Silicon. Only needed by 'analyze'.
  if has dotnet; then ok "dotnet already installed — skipped"
  elif { info "brew install dotnet"; brew install dotnet; }; then ok "installed dotnet"
  else warn "dotnet install failed — only needed by 'analyze'"; fi

  # frida-tools — the memory-dump client. Only needed by 'dump'. Prefer pipx (isolated); the
  # system python is externally-managed, so plain pip needs --break-system-packages.
  if has frida; then ok "frida already installed — skipped"
  else
    if ! has pipx; then info "brew install pipx"; brew install pipx || warn "pipx install failed"; fi
    if has pipx && pipx install frida-tools; then ok "installed frida-tools (pipx) — ensure ~/.local/bin is on PATH"
    elif python3 -m pip install --user --break-system-packages frida-tools >/dev/null 2>&1; then ok "installed frida-tools (pip --user)"
    else warn "frida install failed — run 'pipx install frida-tools' manually (only needed by 'dump')"; fi
  fi

  info "setup complete — run: $0 check  to verify."
}

# =========================================================================================
# Stage: download
# =========================================================================================
stage_download() {
  require_tool apkeep "install with: cargo install apkeep"
  mkdir -p "${WORK}/apk"
  info "Downloading '${PKG}' from APKPure into ${WORK}/apk"
  # NOTE: APKPure availability for this exact package can change; if this fails, download the
  # arm64-v8a XAPK/APK manually (APKMirror/APKPure) into ${WORK}/apk and re-run 'extract'.
  apkeep -a "${PKG}" -d apk-pure "${WORK}/apk" \
    || die "apkeep failed — APKPure may not currently serve '${PKG}'. Download the arm64-v8a XAPK/APK manually into ${WORK}/apk, then run: $0 extract"

  # Report what landed (apkeep may yield .xapk or .apk depending on the listing).
  local pkg_file
  pkg_file="$(find "${WORK}/apk" -maxdepth 1 -type f \( -name '*.xapk' -o -name '*.apk' -o -name '*.apkm' \) -print 2>/dev/null | head -n1)"
  if [[ -n "${pkg_file}" ]]; then
    ok "downloaded package: ${pkg_file}"
  else
    warn "no .xapk/.apk/.apkm found in ${WORK}/apk — inspect the directory contents"
  fi
}

# Resolve the downloaded package path (first .xapk/.apk/.apkm under $WORK/apk).
resolve_package() {
  find "${WORK}/apk" -maxdepth 1 -type f \( -name '*.xapk' -o -name '*.apk' -o -name '*.apkm' \) -print 2>/dev/null | head -n1
}

# =========================================================================================
# Stage: extract
# =========================================================================================
stage_extract() {
  require_tool cargo "install Rust from https://rustup.rs"
  [[ -f "${CLIENT_EXTRACT_MANIFEST}" ]] || die "client-extract crate not found at ${CLIENT_EXTRACT_MANIFEST}"
  local pkg; pkg="$(resolve_package)"
  [[ -n "${pkg}" ]] || die "no downloaded package in ${WORK}/apk — run: $0 download (or drop an .xapk/.apk there)"

  local out="${WORK}/il2cpp-input"
  mkdir -p "${out}"
  info "Triaging ${pkg} with client-extract → ${out}"
  cargo run --manifest-path "${CLIENT_EXTRACT_MANIFEST}" --release -- \
    client-extract -i "${pkg}" -o "${out}"

  info "Expected verdict: global-metadata.dat 'LIKELY ENCRYPTED' — that CONFIRMS a live"
  info "memory dump is required (the encrypted on-disk file is unusable for a static dump)."
  info "The extracted libil2cpp.so is kept only for later offset cross-referencing."
}

# =========================================================================================
# Stage: avd  (best-effort)
# =========================================================================================
stage_avd() {
  local avdmanager emulator adb
  avdmanager="$(require_android_tool avdmanager)"
  emulator="$(require_android_tool emulator)"
  adb="$(require_android_tool adb)"

  # Create the AVD if it does not already exist.
  if "${avdmanager}" list avd 2>/dev/null | grep -qE "Name:[[:space:]]+${AVD_NAME}([[:space:]]|$)"; then
    ok "AVD '${AVD_NAME}' already exists — skipping creation"
  else
    info "Creating AVD '${AVD_NAME}' from ${SYS_IMAGE}"
    info "(the image must already be installed; if this fails run: $0 setup)"
    # `no` declines the custom-hardware-profile prompt.
    echo "no" | "${avdmanager}" create avd -n "${AVD_NAME}" -k "${SYS_IMAGE}" --force \
      || die "avdmanager create failed — confirm the system image is installed: ${SYS_IMAGE}"
    ok "created AVD '${AVD_NAME}'"
  fi

  info "Booting '${AVD_NAME}' with root-writable system (needed for adb root + frida-server push)."
  info "This opens a GUI emulator window; leave it running for the 'dump' stage."
  # -writable-system + a Google-APIs image is what makes 'adb root' succeed.
  # Launched detached so we can poll for boot; the window stays up after this script exits.
  nohup "${emulator}" -avd "${AVD_NAME}" -no-snapshot -writable-system \
    >"${WORK}/emulator.log" 2>&1 &
  local emu_pid=$!
  ok "emulator launched (pid ${emu_pid}); log: ${WORK}/emulator.log"

  info "Waiting for device (adb wait-for-device)…"
  "${adb}" wait-for-device

  info "Waiting for full boot (sys.boot_completed)…"
  local tries=0
  until [[ "$("${adb}" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" == "1" ]]; do
    tries=$((tries + 1))
    if (( tries > 120 )); then
      die "emulator did not finish booting after ~120 polls — check ${WORK}/emulator.log"
    fi
    "${adb}" wait-for-device
    # Deliberate short pause between boot-property polls (foreground sleep is fine here).
    sleep 2
  done
  ok "emulator booted."
  info "Next: hide root as needed (Magisk DenyList / Shamiko, enable Zygisk), then run: $0 dump"
}

# =========================================================================================
# Stage: dump  (BEST-EFFORT — may be detected / blocked)
# =========================================================================================
stage_dump() {
  # =======================================================================================
  # LOUD WARNING. Arknights' anti-tamper actively resists this. This stage MAY FAIL even
  # when every tool is correct — the game can detect the emulator, root, or Frida and refuse
  # to launch or to decrypt metadata. NO SCRIPTING REMOVES THAT WALL. If this is blocked:
  #   * Physical rooted Android device: Magisk DenyList + Shamiko to hide root from Arknights,
  #     enable Zygisk, then re-run this dump flow against that device.
  #   * Zygisk-Il2CppDumper module (Magisk) is often more robust than the Frida script.
  #   * Jailbroken iOS device over USB (palera1n / Dopamine): a memory dump beats FairPlay +
  #     anti-tamper at once. Same `frida -U -f … -l il2cpp_dump.js` invocation.
  # See assets/IL2CPP_RECOVERY.md (Path A §A2–A3, Path B) for the full fallback runbook.
  # =======================================================================================
  warn "BEST-EFFORT stage: Arknights anti-tamper may block this. Fallbacks in IL2CPP_RECOVERY.md."

  local adb frida
  adb="$(require_android_tool adb)"
  require_tool frida "install with: pipx install frida-tools"
  frida="$(command -v frida)"

  [[ -f "${DUMP_JS}" ]] || die "Frida dump script not found at DUMP_JS=${DUMP_JS} — download il2cpp_dump.js from https://github.com/Perfare/Il2CppDumper and set DUMP_JS"

  # Match frida-server to the installed frida client version (mismatches fail to attach).
  local frida_ver
  frida_ver="$("${frida}" --version 2>/dev/null | tr -d '\r')"
  [[ -n "${frida_ver}" ]] || die "could not read frida --version"
  info "frida client version: ${frida_ver} (frida-server MUST match this exactly)"

  # Gain root on the emulator (Google-APIs image + -writable-system required).
  info "adb root"
  "${adb}" root || die "adb root failed — the AVD must use a Google-APIs (NOT Play) image and be booted with -writable-system (run: $0 avd)"
  "${adb}" wait-for-device

  # Fetch the matching arm64 frida-server, decompress, push, chmod, run as root.
  local fs_xz="${WORK}/frida-server-${frida_ver}-android-arm64.xz"
  local fs_bin="${WORK}/frida-server-${frida_ver}-android-arm64"
  if [[ ! -f "${fs_bin}" ]]; then
    require_tool xz "install with: brew install xz"
    local url="https://github.com/frida/frida/releases/download/${frida_ver}/frida-server-${frida_ver}-android-arm64.xz"
    info "Downloading frida-server ${frida_ver} (android-arm64) from GitHub releases"
    if has curl; then
      curl -fL "${url}" -o "${fs_xz}" || die "download failed: ${url} (check the version tag exists)"
    elif has wget; then
      wget -O "${fs_xz}" "${url}" || die "download failed: ${url}"
    else
      die "need curl or wget to fetch frida-server"
    fi
    info "Decompressing ${fs_xz}"
    xz -d -f -k "${fs_xz}"
    # xz -k leaves <name> (without .xz) alongside the archive.
    [[ -f "${fs_bin}" ]] || mv "${WORK}/frida-server-${frida_ver}-android-arm64" "${fs_bin}" 2>/dev/null || true
    [[ -f "${fs_bin}" ]] || die "frida-server binary not produced by decompression"
  else
    ok "frida-server ${frida_ver} already present — skipping download"
  fi

  info "Pushing frida-server to /data/local/tmp and starting it as root (background)"
  "${adb}" push "${fs_bin}" /data/local/tmp/frida-server
  "${adb}" shell "chmod 755 /data/local/tmp/frida-server"
  # Start detached on-device; nohup so it survives this shell.
  "${adb}" shell "su -c 'nohup /data/local/tmp/frida-server >/dev/null 2>&1 &'" \
    || "${adb}" shell "nohup /data/local/tmp/frida-server >/dev/null 2>&1 &" \
    || warn "could not confirm frida-server started — verify with: ${adb} shell pgrep frida-server"
  ok "frida-server launch requested."

  # Install the game (the download stage's split APKs).
  info "Installing '${PKG}' from ${WORK}/apk"
  local apks=()
  local f
  while IFS= read -r f; do apks+=("${f}"); done \
    < <(find "${WORK}/apk" -maxdepth 2 -type f -name '*.apk' 2>/dev/null)
  if (( ${#apks[@]} == 0 )); then
    warn "no extracted .apk splits found under ${WORK}/apk — if you only have an .xapk, unzip it first,"
    warn "or install the game manually onto the emulator, then re-run this stage."
  else
    "${adb}" install-multiple "${apks[@]}" \
      || warn "install-multiple failed (already installed? signature mismatch?) — install manually and continue"
  fi

  # Spawn the game under the Il2CppDumper Frida script — this reads DECRYPTED metadata from
  # the live process. This is the step the anti-tamper may block.
  info "Spawning '${PKG}' under Frida + il2cpp_dump.js (this performs the live dump)"
  info "If the game force-closes or hangs at a black screen, that is the anti-tamper — see fallbacks above."
  "${frida}" -U -f "${PKG}" -l "${DUMP_JS}" --no-pause \
    || warn "frida spawn returned nonzero — the app may have detected Frida. Try the Zygisk-Il2CppDumper module or a physical rooted device."

  # Pull whatever the dumper wrote (dump.cs, dumped libil2cpp.so, decrypted global-metadata.dat).
  mkdir -p "${WORK}/dump"
  info "Pulling /data/data/${PKG}/files/ → ${WORK}/dump"
  # App-private files usually aren't adb-pullable directly; on failure, copy them to
  # /sdcard as root first (rooted device / emulator), then pull that.
  if ! "${adb}" pull "/data/data/${PKG}/files/." "${WORK}/dump"; then
    warn "direct pull failed — retrying via su copy to /sdcard"
    if "${adb}" shell "su -c 'mkdir -p /sdcard/il2cpp-dump && cp -r /data/data/${PKG}/files/. /sdcard/il2cpp-dump/'"; then
      "${adb}" pull /sdcard/il2cpp-dump "${WORK}/dump" \
        || warn "could not pull /sdcard/il2cpp-dump — inspect it manually on the device"
    else
      warn "could not copy dump artifacts as root — check that the process actually dumped and that you have su"
    fi
  fi

  if [[ -f "${WORK}/dump/dump.cs" ]]; then
    ok "GOT dump.cs → ${WORK}/dump/dump.cs — run: $0 analyze"
  else
    warn "no dump.cs pulled. If you have a dumped libil2cpp.so + decrypted global-metadata.dat"
    warn "in ${WORK}/dump, 'analyze' will run desktop Il2CppDumper on them. Otherwise the dump"
    warn "was blocked — use a physical rooted device or the Zygisk module (see IL2CPP_RECOVERY.md)."
  fi
}

# =========================================================================================
# Stage: analyze  (the payoff)
# =========================================================================================
stage_analyze() {
  require_tool cargo "install Rust from https://rustup.rs"
  [[ -f "${IL2CPP_LOCATE_MANIFEST}" ]] || die "il2cpp-locate crate not found at ${IL2CPP_LOCATE_MANIFEST}"

  local dump_cs="${WORK}/dump/dump.cs"

  # If the live dump only produced a raw .so + decrypted metadata, build dump.cs here with
  # the desktop Il2CppDumper.
  if [[ ! -f "${dump_cs}" ]]; then
    info "No dump.cs yet — looking for a dumped .so + decrypted global-metadata.dat to run Il2CppDumper."
    local so meta
    so="$(find "${WORK}/dump" -maxdepth 2 -type f \( -name 'libil2cpp.so' -o -name '*.so' \) -print 2>/dev/null | head -n1)"
    meta="$(find "${WORK}/dump" -maxdepth 2 -type f -name 'global-metadata.dat' -print 2>/dev/null | head -n1)"
    if [[ -n "${so}" && -n "${meta}" ]]; then
      [[ -n "${IL2CPPDUMPER}" && -f "${IL2CPPDUMPER}" ]] \
        || die "IL2CPPDUMPER is not set to a valid Il2CppDumper.dll. Download it from https://github.com/Perfare/Il2CppDumper/releases and set IL2CPPDUMPER=/path/to/Il2CppDumper.dll"
      require_tool dotnet "install with: brew install dotnet"
      info "Running Il2CppDumper on:"
      info "  so   = ${so}"
      info "  meta = ${meta}"
      info "IMPORTANT: 'meta' MUST be the DECRYPTED metadata from the live dump, not the encrypted APK copy."
      dotnet "${IL2CPPDUMPER}" "${so}" "${meta}" "${WORK}/dump" \
        || die "Il2CppDumper failed — verify the .so is the dumped/rebuilt one and metadata is decrypted"
    else
      die "no dump.cs, and no (libil2cpp.so + global-metadata.dat) pair in ${WORK}/dump. Run: $0 dump first (or copy a dump in)."
    fi
  fi

  [[ -f "${dump_cs}" ]] || die "still no dump.cs at ${dump_cs} after Il2CppDumper"

  info "Ranking camera-follow candidates in ${dump_cs} (il2cpp-locate)"
  cargo run --manifest-path "${IL2CPP_LOCATE_MANIFEST}" --release -- --input "${dump_cs}"

  info "The top-ranked row is almost certainly the entrance-camera updater."
  info "Each row prints a ready 'Ghidra: go to file offset 0x…' — open the dumped libil2cpp.so"
  info "in Ghidra at that offset, decompile the one method, and hand back the pseudocode"
  info "(runbook Step E → port into SceneIllust.tsx's entrance tick)."
}

# =========================================================================================
# Stage: all
# =========================================================================================
stage_all() {
  stage_download
  stage_extract
  cat <<EOF

${_bold}=== STOP — interactive steps remain ===${_rst}
download + extract are done. The next two stages need a LIVE GUI emulator session and are
NOT auto-run here (they require a visible emulator window and are subject to Arknights'
anti-tamper). Run them yourself, in order:

  1. $0 avd        # create + boot the rooted Google-APIs arm64 emulator (opens a window)
  2.               # (in the emulator) hide root: Magisk DenyList + Shamiko, enable Zygisk
  3. $0 dump       # BEST-EFFORT live memory dump — MAY BE BLOCKED by anti-tamper
  4. $0 analyze    # Il2CppDumper (if needed) + il2cpp-locate → ranked method + Ghidra offset

If 'dump' is blocked on the emulator, fall back to a physical rooted device or jailbroken
iOS — see assets/IL2CPP_RECOVERY.md. The wall is real; the script is honest about it.
EOF
}

# --- Dispatch ----------------------------------------------------------------------------
main() {
  mkdir -p "${WORK}"
  local stage="${1:-}"
  case "${stage}" in
    check)              stage_check ;;
    setup)              stage_setup ;;
    download)           stage_download ;;
    extract)            stage_extract ;;
    avd)                stage_avd ;;
    dump)               stage_dump ;;
    analyze)            stage_analyze ;;
    all)                stage_all ;;
    -h|--help|help|"")  usage ;;
    *)                  usage; die "unknown stage: ${stage}" ;;
  esac
}

main "$@"
