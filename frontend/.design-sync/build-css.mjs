// Compiles the design-sync Tailwind entry into a plain, static CSS file the
// converter can ship (`cfg.cssEntry`).
//
// Why this exists: src/styles.css is Tailwind *source* (`@import "tailwindcss"`,
// `@theme`, `@plugin`), not consumable CSS. The converter needs a compiled sheet
// that already contains the token layer plus every utility the components use.
//
// The input is .design-sync/tailwind-entry.css, not src/styles.css directly: it
// wraps the app stylesheet to also scan .design-sync/previews/ and to safelist
// the utility vocabulary designs are built with. See that file for why.
//
// It also repoints the @fontsource `url(./files/*.woff2)` references — which are
// package-relative and dead once the CSS moves — at the real files under
// node_modules, so the converter's @font-face scraper can resolve and copy them
// into the bundle's fonts/ dir.
//
// Run from the frontend package root:  node .design-sync/build-css.mjs

import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const PKG_DIR = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const IN = join(PKG_DIR, '.design-sync/tailwind-entry.css');
const OUT_DIR = join(PKG_DIR, '.design-sync/.cache');
const OUT = join(OUT_DIR, 'tailwind.css');
const TW_VERSION = '4.2.2';

// Font families the app imports as npm packages, keyed by the filename prefix
// Tailwind emits in the inlined @font-face rules.
const FONT_PKGS = [
    { prefix: 'inter-', pkg: '@fontsource-variable/inter' },
    { prefix: 'geist-mono-', pkg: '@fontsource-variable/geist-mono' },
];

// ── Safelist ──────────────────────────────────────────────────────────────
// Emitted as a plain candidate file rather than `@source inline(...)`, because
// inline brace-expansion does not cover variants (`md:`), negatives (`-mt-2`) or
// opacity modifiers (`bg-primary/20`) — exactly the classes that silently no-op.
// Tailwind scans this file like any other source. See tailwind-entry.css.
const SPACE = ['0', '0.5', '1', '1.5', '2', '2.5', '3', '4', '5', '6', '8', '10', '12', '16'];
const TEXT_TOKENS = ['foreground', 'muted-foreground', 'card-foreground', 'popover-foreground', 'primary', 'primary-foreground', 'secondary-foreground', 'accent-foreground', 'destructive', 'success', 'warning', 'info'];
const BG_TOKENS = ['background', 'card', 'popover', 'muted', 'accent', 'primary', 'primary-foreground', 'secondary', 'destructive', 'sidebar', 'transparent'];
const ALPHA = ['5', '10', '15', '20', '25', '30', '40', '50', '60', '70', '80', '90'];
const cross = (prefixes, values, { negatives = false } = {}) =>
    prefixes.flatMap((p) => values.flatMap((v) => (negatives ? [`${p}-${v}`, `-${p}-${v}`] : [`${p}-${v}`])));

const safelist = [
    'flex inline-flex grid inline-grid block inline-block hidden contents table'.split(' '),
    'flex-row flex-col flex-wrap flex-nowrap flex-1 flex-none shrink-0 grow min-w-0 w-fit h-fit w-full h-full mx-auto'.split(' '),
    cross(['items'], ['start', 'center', 'end', 'baseline', 'stretch']),
    cross(['justify'], ['start', 'center', 'end', 'between', 'around', 'evenly']),
    cross(['gap', 'gap-x', 'gap-y'], SPACE),
    cross(['p', 'px', 'py', 'pt', 'pr', 'pb', 'pl'], SPACE),
    cross(['m', 'mx', 'my', 'mt', 'mr', 'mb', 'ml'], SPACE, { negatives: true }),
    cross(['space-x', 'space-y'], ['0', '1', '1.5', '2', '3', '4', '6'], { negatives: true }),
    cross(['grid-cols'], ['1', '2', '3', '4', '5', '6', '12']),
    cross(['col-span'], ['1', '2', '3', '4', '6', 'full']),
    cross(['size'], ['3', '3.5', '4', '5', '6', '8', '10', '12', '14', '16']),
    cross(['max-w'], ['xs', 'sm', 'md', 'lg', 'xl', '2xl', '3xl', '4xl', '5xl', '6xl', 'full', 'none', 'prose']),
    'relative absolute fixed sticky inset-0 overflow-hidden overflow-x-auto overflow-y-auto'.split(' '),
    cross(['text'], ['xs', 'sm', 'base', 'lg', 'xl', '2xl', '3xl', '4xl', '5xl', 'left', 'center', 'right', ...TEXT_TOKENS]),
    cross(['font'], ['normal', 'medium', 'semibold', 'bold', 'sans', 'mono', 'heading', 'display']),
    'truncate text-balance text-pretty tabular-nums uppercase lowercase capitalize italic underline leading-none leading-tight leading-relaxed tracking-tight tracking-wide whitespace-nowrap'.split(' '),
    cross(['bg'], BG_TOKENS),
    ['primary', 'muted', 'accent', 'destructive', 'foreground'].flatMap((t) => ALPHA.map((a) => `bg-${t}/${a}`)),
    'border border-t border-b border-l border-r border-0 border-2'.split(' '),
    cross(['border'], ['border', 'input', 'primary', 'destructive', 'transparent']),
    cross(['rounded'], ['sm', 'md', 'lg', 'xl', '2xl', '3xl', 'full', 'none']).concat(['rounded']),
    'shadow-xs shadow-sm shadow shadow-md shadow-lg shadow-none'.split(' '),
    cross(['opacity'], ['40', '50', '60', '70', '80', '90']),
];
const flat = [...new Set(safelist.flat())];
// Responsive + dark variants of the classes that actually change per breakpoint.
const RESPONSIVE = flat.filter((c) => /^(flex|grid|hidden|block|flex-row|flex-col|grid-cols-|col-span-|gap-|p[xytrbl]?-|m[xytrbl]?-|text-(xs|sm|base|lg|xl|2xl|3xl|4xl|5xl)$|max-w-|items-|justify-)/.test(c));
const candidates = [
    ...flat,
    ...['sm', 'md', 'lg', 'xl'].flatMap((bp) => RESPONSIVE.map((c) => `${bp}:${c}`)),
    ...flat.filter((c) => /^(bg|text|border)-/.test(c)).map((c) => `dark:${c}`),
];

mkdirSync(OUT_DIR, { recursive: true });
writeFileSync(join(OUT_DIR, 'safelist.txt'), `${candidates.join('\n')}\n`);

execFileSync('bunx', [`@tailwindcss/cli@${TW_VERSION}`, '-i', IN, '-o', OUT], {
    cwd: PKG_DIR,
    stdio: ['ignore', 'inherit', 'inherit'],
});

let css = readFileSync(OUT, 'utf8');
let rewritten = 0;
const unresolved = [];

css = css.replace(/url\((['"]?)\.\/files\/([^'")]+)\1\)/g, (whole, quote, file) => {
    const hit = FONT_PKGS.find((f) => file.startsWith(f.prefix));
    const abs = hit && join(PKG_DIR, 'node_modules', hit.pkg, 'files', file);
    if (!abs || !existsSync(abs)) {
        unresolved.push(file);
        return whole;
    }
    rewritten++;
    return `url(${quote}${relative(dirname(OUT), abs)}${quote})`;
});

writeFileSync(OUT, css);

const kb = (css.length / 1024).toFixed(0);
console.error(`  design-sync css: ${kb}KB → ${relative(PKG_DIR, OUT)} (${rewritten} font url(s) repointed)`);
if (unresolved.length) console.error(`  ! unresolved font files: ${unresolved.join(', ')}`);
