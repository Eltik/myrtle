#!/bin/sh
# Read-only VPS inspection for finalizing deploy-enemy-chibis.sh.
# Prints no secrets: only whitelisted .env keys are shown.

section() { printf '\n===== %s =====\n' "$1"; }

section "system"
uname -m
[ -f /etc/os-release ] && grep -E '^(PRETTY_NAME)=' /etc/os-release
nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null
free -h 2>/dev/null | head -2
df -h /var/www / 2>/dev/null | awk 'NR==1 || /\/var\/www|\/$/'

section "toolchains"
for tool in node pm2 cargo rustc bun curl rsync; do
    if command -v "$tool" >/dev/null 2>&1; then
        printf '%-7s %s  (%s)\n' "$tool" "$("$tool" --version 2>/dev/null | head -1)" "$(command -v "$tool")"
    else
        printf '%-7s MISSING\n' "$tool"
    fi
done

section "pm2 processes"
pm2 ls 2>/dev/null
pm2 jlist 2>/dev/null | node -e '
const ps = JSON.parse(require("fs").readFileSync(0, "utf8"));
for (const p of ps) {
    console.log([
        p.name,
        "cwd=" + (p.pm2_env.pm_cwd || "?"),
        "exec=" + (p.pm2_env.pm_exec_path || "?"),
        "interp=" + (p.pm2_env.exec_interpreter || "?"),
        "args=" + JSON.stringify(p.pm2_env.args || []),
    ].join("  "));
}' 2>/dev/null

section "web root layout"
ls -la /var/www 2>/dev/null
for root in /var/www/myrtle.moe /var/www/myrtle; do
    if [ -d "$root" ]; then
        echo "--- $root ---"
        ls "$root"
        [ -f "$root/backend/Cargo.toml" ] && echo "backend/Cargo.toml: present"
        [ -f "$root/frontend/package.json" ] && echo "frontend/package.json: present"
        [ -d "$root/backend/target/release" ] && ls -la "$root/backend/target/release/backend" 2>/dev/null
    fi
done

section "assets layout"
for root in /var/www/myrtle.moe /var/www/myrtle; do
    [ -d "$root" ] || continue
    for spine in "$root/assets/output/en/spine" "$root/assets/en/spine" "$root/spine"; do
        if [ -d "$spine" ]; then
            echo "--- $spine ---"
            ls "$spine"
            du -sh "$spine"/* 2>/dev/null
        fi
    done
done

section "backend config (whitelisted keys only)"
for root in /var/www/myrtle.moe /var/www/myrtle; do
    if [ -f "$root/backend/.env" ]; then
        echo "--- $root/backend/.env ---"
        grep -E '^\s*(ASSETS_DIR|PORT|HOST|DPS_AUTO_RESTART|ASSET_WS_URL)' "$root/backend/.env" 2>/dev/null
    fi
done

section "listening ports (web/backend)"
(ss -tln 2>/dev/null || netstat -tln 2>/dev/null) | grep -E ':(80|443|3000|3060)\b'

section "backend endpoint check"
curl -s -o /dev/null -w "GET /api/static/chibis -> %{http_code}\n" --max-time 10 http://localhost:3060/api/static/chibis 2>/dev/null
curl -s -o /dev/null -w "GET /api/static/enemy-chibis -> %{http_code}\n" --max-time 10 http://localhost:3060/api/static/enemy-chibis 2>/dev/null

echo
echo "===== done ====="
