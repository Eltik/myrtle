// Copies the root-relative static assets the components hardcode into the built
// bundle. Run AFTER every `package-build.mjs` / `resync.mjs` run — the converter
// wipes the output directory, so this cannot be a one-off.
//
//   node .design-sync/copy-assets.mjs
//
// Why it exists: Header/AdminSidebar/MobileNav/Footer reference
// `/logo/bust_transparent.png` and SummonsSection/InfoContent reference
// `/stat-icons/*.png` as absolute paths. Without these files at the project root
// those cards show broken-image glyphs — and so would any design built with the
// components. `.design-sync/assets/logo/` holds 512px versions of the app's
// originals, which are ~2.8 MB each and render at roughly 22 px.

import { cpSync, existsSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const PKG_DIR = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const OUT = join(PKG_DIR, 'ds-bundle');

if (!existsSync(OUT)) {
    console.error('! copy-assets: ds-bundle/ does not exist — run the build first');
    process.exit(1);
}

for (const dir of ['logo', 'stat-icons']) {
    const from = join(PKG_DIR, '.design-sync/assets', dir);
    const to = join(OUT, dir);
    mkdirSync(to, { recursive: true });
    cpSync(from, to, { recursive: true });
}

console.error('  design-sync assets: logo/ + stat-icons/ → ds-bundle/');
