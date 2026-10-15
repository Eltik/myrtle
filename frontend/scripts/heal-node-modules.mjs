// Repairs the node_modules damage left by the commit that stopped tracking
// frontend/node_modules (7630c094). Runs from `postinstall`, once per install tree.
//
// Until that commit, git tracked 4,113 files inside these 16 packages. Pulling it
// deletes those files from any checkout that already had them installed, the VPS
// included. `bun install --frozen-lockfile` does not notice: it skips a package
// whose package.json is still in place, so @base-ui/react stays half-deleted and
// the build fails on `@base-ui/react/use-render`. Measured on bun 1.4.2: a plain
// install restored every package except @base-ui/react (1,608 files missing).
//
// Bun runs lifecycle scripts after it has decided what to install, so a package
// removed here is not reinstalled by the install that ran this script. Hence the
// nested install: remove the 16 packages, install again, which puts them back in
// full from bun's cache. On a fresh clone the same steps run once and cost well
// under a second; afterwards a marker file skips all of it.
import { spawnSync } from "node:child_process";
import { existsSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const NODE_MODULES = "node_modules";
const MARKER = join(NODE_MODULES, ".myrtle-untrack-healed");
const NESTED = "MYRTLE_HEAL_NESTED";

// Every package that had files tracked in git before 7630c094.
const ONCE_TRACKED = [
    "@base-ui/react",
    "@base-ui/utils",
    "ansi-styles",
    "boolbase",
    "cac",
    "class-variance-authority",
    "clsx",
    "date-fns-jalali",
    "isbot",
    "js-tokens",
    "loupe",
    "picocolors",
    "semver",
    "unplugin",
    "vite",
    "xmlchars",
];

if (process.env[NESTED] || existsSync(MARKER) || !existsSync(NODE_MODULES)) process.exit(0);

for (const name of ONCE_TRACKED) rmSync(join(NODE_MODULES, name), { recursive: true, force: true });

console.log(`heal-node-modules: reinstalling ${ONCE_TRACKED.length} packages once tracked in git`);
const result = spawnSync(process.execPath, ["install", "--frozen-lockfile"], {
    stdio: "inherit",
    env: { ...process.env, [NESTED]: "1" },
});
if (result.status !== 0) {
    console.error("heal-node-modules: reinstall failed; run `bun install --frozen-lockfile --force` by hand");
    process.exit(result.status ?? 1);
}

writeFileSync(MARKER, "Packages once tracked by git were reinstalled. Delete this file to repeat.\n");
