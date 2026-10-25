// @vitest-environment node
/**
 * The site calls its users "players". Arknights calls them "Doctors", and the
 * site used both until the wording settled on "players" (nav, headings, SEO,
 * OG cards). Leftovers kept turning up one string at a time (a "Top Doctors"
 * panel, "opted-in doctors" in the Gacha menu), so this guards the whole class
 * instead of the instances: every English string a person can read must not
 * say Doctor/Doctors unless it is listed in ALLOWED below with a reason.
 *
 * What counts as readable English, and why each source is scanned:
 *
 *   `*.messages.ts`   the `text` of every message: the English catalog behind
 *                     every `t()` call, including `seo()` titles and OG card
 *                     copy. Descriptions are translator notes and are skipped,
 *                     since they legitimately explain the game's own term.
 *   other .ts/.tsx    every string literal, template chunk and JSX text, so a
 *                     hardcoded placeholder or a string that bypassed the
 *                     catalog is caught too. Skipped: import paths, message
 *                     keys (dotted identifiers like `search.breadcrumb.doctors`,
 *                     never rendered), object keys, type-level literals and
 *                     `class`/`className` values.
 *   openapi.json      the backend's API documentation snapshot. Its doc
 *                     comments are published as the API reference, and
 *                     `openapi_snapshot_test` keeps the file equal to the code.
 *
 * Test files are not scanned: fixtures quote game text ("Doctor, can you hear
 * me?") as data, not as UI.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import { describe, expect, it } from "vitest";

const SRC = fileURLToPath(new URL("../../", import.meta.url));
const OPENAPI = fileURLToPath(new URL("../../../../backend/tests/snapshots/openapi.json", import.meta.url));

/** Case-insensitive and word-bounded: "Doctors" and "doctor" match, "doctorate" does not. */
const WORD = /\bdoctors?\b/i;

/**
 * Strings that may say Doctor. Each is a game proper noun or a quotation of
 * game text, not the site's name for its users. `text` omitted allows every
 * match in the file. An entry that stops matching anything fails the test, so
 * this list cannot outlive the string it excuses.
 */
const ALLOWED: ReadonlyArray<{ file: string; text?: string; why: string }> = [
    // The story reader. "Doctor" is the protagonist's name in the game's own
    // script: `{@nickname}` lines address the player character as "Doctor" when
    // no name is set, so these label and default the in-story name, not the site's users.
    { file: "components/story/reader/reader.messages.ts", text: "Doctor's name", why: "Label of the in-story protagonist name field." },
    { file: "components/story/reader/reader.messages.ts", text: "Used wherever a story addresses you by name. Up to 24 characters; an empty field reads as Doctor.", why: "States the game's default protagonist name." },
    { file: "components/story/reader/reader.messages.ts", text: "Doctor. The convoy reaches the checkpoint at dawn.", why: "Font preview styled as a line of game dialogue." },
    { file: "components/story/export/export.messages.ts", text: "Doctor's name", why: "Same in-story protagonist name field, in the export sheet." },
    { file: "components/story/export/ExportSheet.tsx", text: "Doctor", why: "Placeholder showing the game's default protagonist name." },
    { file: "components/story/sprites/NameChip.tsx", text: "Doctor", why: "The protagonist's name on a story speaker chip." },
    { file: "lib/story/engine.ts", text: "Doctor", why: "The game's default protagonist name substituted into `{@nickname}`." },
    { file: "lib/story/settings.ts", text: "Doctor", why: "DEFAULT_NICKNAME, the game's default protagonist name." },
    // Renderer fixture shipped beside the book code (not a *.test file): quotes game dialogue.
    { file: "lib/story/book/sample.ts", why: "Test fixture quoting game dialogue." },
    // Command palette search synonyms are matched against what a person types
    // and never rendered, so someone who types the game's word still lands on player search.
    { file: "lib/registry/pages.ts", text: "doctors", why: "Search synonym for player search, never displayed." },
];

const SKIP_DIRS = new Set(["node_modules", "generated", "__snapshots__"]);

function* sourceFiles(dir: string): Generator<string> {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true }).sort((a, b) => (a.name < b.name ? -1 : 1))) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            if (!SKIP_DIRS.has(entry.name)) yield* sourceFiles(full);
            continue;
        }
        if (!/\.tsx?$/.test(entry.name) || /\.test\.tsx?$/.test(entry.name) || entry.name.endsWith(".d.ts") || entry.name === "routeTree.gen.ts" || entry.name === "test-setup.ts") continue;
        yield full;
    }
}

/** A message key or other dotted identifier: `user.search.empty.noResults.title`. Never shown as-is. */
const KEY_SHAPED = /^[\w$-]+(\.[\w$-]+)+$/;

function propertyNameText(name: ts.PropertyName): string | null {
    return ts.isIdentifier(name) || ts.isStringLiteralLike(name) ? name.text : null;
}

/** True when `node` sits where its text is a name, a path or a type, not something rendered. */
function isNonDisplayPosition(node: ts.Node): boolean {
    const parent = node.parent;
    if (!parent) return false;
    if (ts.isImportDeclaration(parent) || ts.isExportDeclaration(parent) || ts.isExternalModuleReference(parent) || ts.isLiteralTypeNode(parent)) return true;
    if (ts.isCallExpression(parent) && parent.expression.kind === ts.SyntaxKind.ImportKeyword) return true;
    // `{ "some key": ... }` and `{ class: "line doctor" }`
    if ((ts.isPropertyAssignment(parent) || ts.isPropertySignature(parent) || ts.isMethodDeclaration(parent)) && parent.name === node) return true;
    if (ts.isPropertyAssignment(parent) && parent.initializer === node) {
        const name = propertyNameText(parent.name);
        if (name === "class" || name === "className") return true;
    }
    if (ts.isJsxAttribute(parent) && ts.isIdentifier(parent.name) && parent.name.text === "className") return true;
    if (ts.isElementAccessExpression(parent) && parent.argumentExpression === node) return true;
    return false;
}

interface Hit {
    file: string;
    line: number;
    text: string;
}

function scanFile(file: string): Hit[] {
    const rel = path.relative(SRC, file).split(path.sep).join("/");
    const source = ts.createSourceFile(file, fs.readFileSync(file, "utf8"), ts.ScriptTarget.Latest, true, file.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS);
    const messagesModule = /\.messages\.tsx?$/.test(file);
    const hits: Hit[] = [];

    const record = (node: ts.Node, text: string) => {
        if (!WORD.test(text) || KEY_SHAPED.test(text)) return;
        hits.push({ file: rel, line: source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1, text });
    };

    const visit = (node: ts.Node) => {
        if (messagesModule) {
            // Only a message's `text` is English a person reads.
            if (ts.isPropertyAssignment(node) && propertyNameText(node.name) === "text" && ts.isStringLiteralLike(node.initializer)) {
                record(node.initializer, node.initializer.text);
            }
        } else if (ts.isStringLiteralLike(node) || ts.isTemplateHead(node) || ts.isTemplateMiddle(node) || ts.isTemplateTail(node)) {
            if (!isNonDisplayPosition(node)) record(node, node.text);
        } else if (ts.isJsxText(node)) {
            record(node, node.text.trim());
        }
        ts.forEachChild(node, visit);
    };
    visit(source);
    return hits;
}

function scanOpenApi(): Hit[] {
    const doc = JSON.parse(fs.readFileSync(OPENAPI, "utf8"));
    const hits: Hit[] = [];
    const walk = (value: unknown, at: string) => {
        if (Array.isArray(value)) {
            for (const [i, item] of value.entries()) walk(item, `${at}/${i}`);
            return;
        }
        if (value === null || typeof value !== "object") return;
        for (const [key, inner] of Object.entries(value)) {
            // Prose fields only: schema and property NAMES are API identifiers.
            if ((key === "description" || key === "summary" || key === "title") && typeof inner === "string" && WORD.test(inner)) {
                hits.push({ file: "openapi.json", line: 0, text: `${at}/${key}: ${inner}` });
            }
            walk(inner, `${at}/${key}`);
        }
    };
    walk(doc, "");
    return hits;
}

function isAllowed(hit: Hit): boolean {
    return ALLOWED.some((allow) => allow.file === hit.file && (allow.text === undefined || allow.text === hit.text));
}

describe("the site calls its users players", () => {
    const hits = [...sourceFiles(SRC)].flatMap(scanFile);

    it("no user-facing frontend string says Doctor outside the allowlist", () => {
        const offending = hits.filter((hit) => !isAllowed(hit)).map((hit) => `${hit.file}:${hit.line}  ${JSON.stringify(hit.text)}`);
        expect(offending, "Say 'player(s)'. If this is a game proper noun or quoted game text, add it to ALLOWED with a reason.").toEqual([]);
    });

    it("every allowlist entry still excuses a real string", () => {
        const stale = ALLOWED.filter((allow) => !hits.some((hit) => allow.file === hit.file && (allow.text === undefined || allow.text === hit.text))).map((allow) => `${allow.file} ${JSON.stringify(allow.text ?? "*")}`);
        expect(stale, "Remove allowlist entries whose string is gone.").toEqual([]);
    });

    it.skipIf(!fs.existsSync(OPENAPI))("no API documentation string says Doctor", () => {
        expect(scanOpenApi()).toEqual([]);
    });

    it("the scanner sees the strings it claims to (self-check)", () => {
        // If a refactor of the scanner silently matched nothing, both tests
        // above would pass vacuously. The allowlisted story strings are known
        // to exist, so finding them proves each source kind is read.
        expect(hits.some((hit) => hit.file.endsWith(".messages.ts"))).toBe(true);
        expect(hits.some((hit) => hit.file === "components/story/export/ExportSheet.tsx")).toBe(true);
    });
});
