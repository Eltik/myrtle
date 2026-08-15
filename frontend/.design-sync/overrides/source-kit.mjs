// forked from design-sync lib/source-kit.mjs — app repo, not a library: no dist entry, and `export *` over 330 component files silently drops 22 colliding names.
//
// What changes vs. the bundled adapter:
//  1. The synthesized entry is a barrel of EXPLICIT named re-exports instead of
//     `export * from ...`. ESM makes an ambiguous star-export name unresolvable,
//     so names like Kicker/Hero/Pagination (declared in 2-4 files each) would
//     vanish from window.<globalName>. Colliding names are disambiguated with
//     their feature-dir prefix (HomeHero, LeaderboardHero, ...), and `ui/`
//     always wins the bare name because it is the design-system layer.
//  2. Grouping is the top-level dir under src/components (ui -> "primitives"),
//     because the bundled heuristic treats `ui` as a generic container and would
//     drop every primitive into "general".
//  3. srcPath is known exactly (the barrel builder read the file), so the
//     fuzzy-find enrichment pass is skipped rather than guessing.
//
// Everything else keeps the bundled contract: same resolvePackage(ctx) signature
// and the same { shape, entry, components, synthEntry, exported } return value.

import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { Node, Project, ts } from 'ts-morph';
import { leadingJsdoc, readText, slash, walk } from '../../.ds-sync/lib/common.mjs';

const NON_IMPL_RX = /\.(stories|test|spec)\./;
const SRC_IMPL_RX = /\.(tsx|jsx)$/;
// Path segments that describe file organisation rather than a feature area, so
// they never earn a place in a disambiguating prefix.
const FILLER_DIR = new Set(['impl', 'components', 'component', 'view', 'cards', 'tabs', 'screens', 'shared', 'src', 'lib']);
const GROUP_ALIAS = { ui: 'primitives' };

const pascal = (s) =>
    s
        .split(/[^a-zA-Z0-9]+/)
        .filter(Boolean)
        .map((w) => w[0].toUpperCase() + w.slice(1))
        .join('');
const slug = (s) => s.trim().toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '') || 'general';

// Value exports only — a type re-exported through `export { T } from` is a build
// error, and the design agent can only build with runtime values anyway.
function valueExports(sf, dsLayer) {
    const out = [];
    for (const [name, decls] of sf.getExportedDeclarations()) {
        const isValue = decls.some(
            (d) =>
                Node.isVariableDeclaration(d) ||
                Node.isFunctionDeclaration(d) ||
                Node.isClassDeclaration(d) ||
                Node.isEnumDeclaration(d),
        );
        if (!isValue) continue;
        // `export default function Hero()` is keyed 'default'; recover the
        // declared name so the barrel can re-export it as `default as Hero`.
        if (name === 'default') {
            const real = decls.map((d) => d.getName?.()).find((n) => n && n !== 'default');
            if (real && /^[A-Z][A-Za-z0-9]*$/.test(real)) out.push({ local: 'default', name: real });
            continue;
        }
        if (/^[A-Z][A-Za-z0-9]*$/.test(name)) {
            out.push({ local: name, name });
            continue;
        }
        // Lowercase-initial value exports from the design-system layer only:
        // `cn`, `buttonVariants`, `toastManager`, `useComboboxFilter`, … These
        // are real API a design is built with (there is no other way to raise a
        // toast, or to style a non-button as a button), but they are not
        // components — they ride the bundle without earning a card.
        if (dsLayer && /^[a-z][A-Za-z0-9]*$/.test(name)) out.push({ local: name, name, helper: true });
    }
    return out;
}

export async function resolvePackage(ctx) {
    const { PKG_DIR, PKG, OUT, cfg } = ctx;
    const srcMap = cfg.componentSrcMap ?? {};

    const srcRoot = [cfg.srcDir, 'src', 'lib', 'components'].map((d) => d && resolve(PKG_DIR, d)).find((d) => d && existsSync(d));
    if (!srcRoot) {
        console.error(`[NO_DIST] ${PKG} has no built entry and no src/ to synthesize from.`);
        process.exit(1);
    }

    const files = walk(srcRoot, (n) => SRC_IMPL_RX.test(n))
        .filter((p) => !NON_IMPL_RX.test(p))
        // `ui/` first so the design-system layer claims contested names; the rest
        // alphabetically, which keeps the barrel byte-stable across runs.
        .sort((a, b) => {
            const ui = (p) => (slash(relative(srcRoot, p)).startsWith('ui/') ? 0 : 1);
            return ui(a) - ui(b) || (a < b ? -1 : a > b ? 1 : 0);
        });

    const project = new Project({
        skipAddingFilesFromTsConfig: true,
        compilerOptions: { jsx: ts.JsxEmit.Preserve, allowJs: true, skipLibCheck: true },
    });

    // Pass 1: collect every value export with the file it came from.
    const raw = [];
    for (const p of files) {
        const sf = project.addSourceFileAtPathIfExists(p);
        if (!sf) continue;
        const segs = slash(relative(srcRoot, dirname(p))).split('/').filter(Boolean);
        for (const e of valueExports(sf, segs[0] === 'ui')) raw.push({ ...e, file: p, segs });
        project.removeSourceFile(sf);
    }

    // Pass 2: disambiguate. `ui/` keeps the bare name; every other claimant gets
    // its feature dir prepended, widening through the path until unique.
    const counts = new Map();
    for (const r of raw) counts.set(r.name, (counts.get(r.name) ?? 0) + 1);

    const taken = new Set();
    const picked = [];
    for (const r of raw) {
        let final = r.name;
        if (counts.get(r.name) > 1 && r.segs[0] !== 'ui') {
            const parts = r.segs.filter((s) => !FILLER_DIR.has(s.toLowerCase()));
            final = null;
            for (let depth = 1; depth <= parts.length; depth++) {
                // Widen from the most specific segment outward: profile -> user/profile.
                const cand = pascal(parts.slice(-depth).join('-')) + r.name;
                if (!taken.has(cand)) {
                    final = cand;
                    break;
                }
            }
            final ??= `${pascal(parts.join('-'))}${r.name}`;
        }
        // Same name, same layer (e.g. FieldPrimitive re-exported by two ui files):
        // identical binding, so the first one is the canonical export.
        if (taken.has(final)) continue;
        taken.add(final);
        picked.push({ ...r, final });
    }

    // Pass 3: the barrel.
    const byFile = new Map();
    for (const p of picked) {
        if (!byFile.has(p.file)) byFile.set(p.file, []);
        byFile.get(p.file).push(p);
    }
    const entry = join(OUT, '.pkg-entry.mjs');
    const lines = [];
    for (const [file, exps] of byFile) {
        const spec = exps.map((e) => (e.local === e.final ? e.local : `${e.local} as ${e.final}`)).join(', ');
        lines.push(`export { ${spec} } from ${JSON.stringify(file)};`);
    }
    writeFileSync(entry, `${lines.join('\n')}\n`);
    console.error(`[OVERRIDE] source-kit: barrel of ${picked.length} exports from ${byFile.size} files`);

    // Hand the .d.ts extractor the emitted-name → source-declaration mapping.
    // It can't recover this itself: 39 exports are renamed to break collisions,
    // so the name on the card is not the name declared in the file.
    // Consumed by .design-sync/overrides/dts.mjs.
    const srcMapOut = join(PKG_DIR, '.design-sync/.cache/component-src.json');
    mkdirSync(dirname(srcMapOut), { recursive: true });
    writeFileSync(
        srcMapOut,
        `${JSON.stringify(Object.fromEntries(picked.map((p) => [p.final, { file: p.file, local: p.local }])), null, 2)}\n`,
    );

    const renamed = picked.filter((p) => p.final !== p.name);
    if (renamed.length) {
        console.error(`  disambiguated ${renamed.length}: ${renamed.map((p) => `${p.name}->${p.final}`).join(', ')}`);
    }

    // Pass 4: components (cards). Same barrel set, minus config exclusions.
    const components = picked
        .filter((p) => !p.helper && srcMap[p.final] !== null)
        .map((p) => ({
            name: p.final,
            group: slug(GROUP_ALIAS[p.segs[0]] ?? p.segs[0] ?? 'general'),
            srcPath: p.file,
            doc: leadingJsdoc(readText(p.file), p.name) || undefined,
        }))
        .sort((a, b) => (a.name < b.name ? -1 : 1));

    if (!components.length) {
        console.error('[ZERO_MATCH] barrel produced no components');
        process.exit(1);
    }

    console.error(`  package: ${components.length} components (${components.length} src-matched)`);
    return { shape: 'package', entry, components, synthEntry: true, exported: new Set(picked.map((p) => p.final)) };
}
