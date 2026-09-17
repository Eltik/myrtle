#!/usr/bin/env node
/**
 * Build the bundled English source catalog from the source tree.
 *
 * Two files come out of this, and they have different audiences:
 *
 *   src/lib/i18n/source-catalog.json      flat key -> English text. Imported
 *                                         by `source.ts`, so it ships in the
 *                                         bundle as the floor beneath every
 *                                         locale.
 *   src/lib/i18n/source-catalog.meta.json the payload for the backend's
 *                                         `/api/admin/i18n/sync`. Carries the
 *                                         namespace, the translator-facing
 *                                         description and the declared ICU
 *                                         placeholders per key. Imported only
 *                                         from here, never from app code, so
 *                                         none of it reaches the client.
 *
 * Two independent sources are read, which is the point:
 *
 *   TEXT  comes from `*.messages.ts` modules - `export const namespace` plus
 *         `export const messages = { ... } satisfies MessageMap`.
 *   USAGE comes from `t("key")` call sites, where `t` is the binding returned
 *         by `useT("namespace")` or `useRichT("namespace")` in that same file.
 *
 * Crossing them catches the two failure modes that otherwise only show up in
 * production: a key that is defined and never rendered (dead weight in the
 * translation queue) and a key that is rendered and never defined (renders as
 * its own raw identifier the moment the database does not have it).
 *
 * Usage:
 *   node scripts/i18n-extract.mjs            write both files
 *   node scripts/i18n-extract.mjs --check    exit 1 if they would change (CI)
 *   node scripts/i18n-extract.mjs --sync     write, then POST to the backend
 */

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const FRONTEND = path.resolve(HERE, "..");
const SRC = path.join(FRONTEND, "src");
const CATALOG_PATH = path.join(SRC, "lib", "i18n", "source-catalog.json");
const META_PATH = path.join(SRC, "lib", "i18n", "source-catalog.meta.json");

const GENERATED_BY = "bun run i18n:extract";
const SYNC_ENDPOINT = "/api/admin/i18n/sync";

/** Directories with nothing hand-written in them. */
const SKIP_DIRS = new Set([".git", ".nitro", ".output", ".vite", "dist", "node_modules", "coverage"]);
/** Generated or declaration-only files that cannot contain a call site. */
const SKIP_FILES = new Set(["routeTree.gen.ts"]);

// ----------------------------------------------------------------- placeholders

/**
 * The ICU argument names a source string references.
 *
 * This mirrors `format.ts`'s parser deliberately closely - same quoting rules,
 * same tolerance of malformed braces, same set of recognised types - because
 * the backend rejects a translation that drops a declared placeholder or
 * invents one. A list that disagreed with what the formatter actually
 * substitutes would either block a valid translation or let a broken one
 * through.
 */
function placeholdersOf(message) {
    const found = new Set();
    scanNodes(message, 0, false, found);
    return [...found].sort();
}

function scanNodes(src, start, nested, out) {
    let i = start;

    while (i < src.length) {
        const ch = src[i];

        if (ch === "'") {
            // ICU 4.8 "real literal" quoting, matching `format.ts` and the
            // backend validator:
            //
            //   ''            -> a literal apostrophe
            //   '{ '} '# '|   -> opens a quoted run, closed by the next '
            //   ' anywhere else -> a plain apostrophe, not a quote
            //
            // The last clause is the one this file was missing. Skipping to
            // the next quote on ANY apostrophe walked straight past the
            // argument in strings like "Couldn't load permissions for {slug}."
            // and "You've reached the cap ({max})", so those keys were
            // published declaring NO placeholders - and the backend then
            // rejected any translation that kept them, including the English
            // source pasted verbatim. 14 keys were untranslatable this way.
            if (src[i + 1] === "'") {
                i += 2;
                continue;
            }
            if (src[i + 1] === "{" || src[i + 1] === "}" || src[i + 1] === "#" || src[i + 1] === "|") {
                const close = src.indexOf("'", i + 2);
                i = close === -1 ? src.length : close + 1;
                continue;
            }
            i += 1;
            continue;
        }

        if (ch === "}" && nested) return i;

        if (ch === "#" && nested) {
            i += 1;
            continue;
        }

        if (ch === "{") {
            i = scanArg(src, i, out);
            continue;
        }

        i += 1;
    }

    return i;
}

function scanArg(src, start, out) {
    let i = start + 1;
    while (i < src.length && /\s/.test(src[i])) i += 1;

    let name = "";
    while (i < src.length && /[\w$]/.test(src[i])) {
        name += src[i];
        i += 1;
    }
    while (i < src.length && /\s/.test(src[i])) i += 1;

    if (name) out.add(name);

    // `{name}`
    if (src[i] === "}") return i + 1;

    // Malformed: `format.ts` swallows to the closing brace and renders noise.
    if (src[i] !== ",") {
        const close = src.indexOf("}", i);
        return close === -1 ? src.length : close + 1;
    }
    i += 1;
    while (i < src.length && /\s/.test(src[i])) i += 1;

    let typeName = "";
    while (i < src.length && /[a-zA-Z]/.test(src[i])) {
        typeName += src[i];
        i += 1;
    }
    while (i < src.length && /\s/.test(src[i])) i += 1;

    const type = ["plural", "selectordinal", "select", "number", "date", "time"].find((t) => t === typeName.toLowerCase());

    if (!type || type === "number" || type === "date" || type === "time") {
        const close = src.indexOf("}", i);
        return close === -1 ? src.length : close + 1;
    }

    // plural / selectordinal / select: `, key {branch} key {branch}`
    if (src[i] === ",") i += 1;

    while (i < src.length) {
        while (i < src.length && /\s/.test(src[i])) i += 1;
        if (src[i] === "}") {
            i += 1;
            break;
        }

        let key = "";
        while (i < src.length && !/[\s{]/.test(src[i])) {
            key += src[i];
            i += 1;
        }
        if (!key) break;

        if (key.startsWith("offset:")) continue;

        while (i < src.length && /\s/.test(src[i])) i += 1;
        if (src[i] !== "{") break;

        i = scanNodes(src, i + 1, true, out) + 1;
    }

    return i;
}

// ----------------------------------------------------------------- tree walk

function* walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true }).sort((a, b) => (a.name < b.name ? -1 : 1))) {
        if (entry.isDirectory()) {
            if (SKIP_DIRS.has(entry.name)) continue;
            yield* walk(path.join(dir, entry.name));
            continue;
        }
        if (!entry.isFile()) continue;
        // Tests declare throwaway catalogs under fake namespaces to exercise
        // the formatter. Counting those as usage makes the missing-key check
        // report fixtures as production keys.
        if (SKIP_FILES.has(entry.name) || entry.name.endsWith(".d.ts") || /\.test\.tsx?$/.test(entry.name)) continue;
        if (!/\.tsx?$/.test(entry.name)) continue;
        yield path.join(dir, entry.name);
    }
}

function parseFile(file) {
    return ts.createSourceFile(
        file,
        fs.readFileSync(file, "utf8"),
        ts.ScriptTarget.Latest,
        true,
        file.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS,
    );
}

function lineOf(sourceFile, node) {
    return sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile)).line + 1;
}

/** The prefixing rule shared by `useT` and `useRichT`, copied from `context.tsx`. */
function fullKey(namespace, key) {
    const prefix = namespace ? `${namespace}.` : "";
    return prefix && !key.startsWith(prefix) ? `${prefix}${key}` : key;
}

function stringOf(node) {
    if (!node) return null;
    if (ts.isStringLiteralLike(node)) return node.text;
    return null;
}

/** Unwrap `x satisfies T` / `x as T` / `(x)` down to the value. */
function unwrap(node) {
    let current = node;
    while (
        current &&
        (ts.isSatisfiesExpression(current) || ts.isAsExpression(current) || ts.isParenthesizedExpression(current) || ts.isTypeAssertionExpression?.(current))
    ) {
        current = current.expression;
    }
    return current;
}

function propertyName(prop) {
    const name = prop.name;
    if (!name) return null;
    if (ts.isIdentifier(name) || ts.isStringLiteralLike(name)) return name.text;
    return null;
}

// ----------------------------------------------------------------- definitions

/**
 * Read one `*.messages.ts` module.
 *
 * Parsed, not imported: a module that has to be executed to be read would drag
 * the whole `#/` alias graph and every React import into a plain node script.
 * The shape it accepts is exactly the shape `defineMessages` documents, so
 * anything it cannot read statically is a mistake worth reporting rather than
 * a pattern worth supporting.
 */
function readMessagesModule(file, problems) {
    const sourceFile = parseFile(file);
    const rel = path.relative(FRONTEND, file);

    let namespace = null;
    let messagesObject = null;
    // `dynamic: true` marks a module whose keys are rendered through a
    // variable (a registry storing `labelKey`), so the unused-key check has no
    // literal call site to find and must not report them.
    let dynamic = false;

    const visit = (node) => {
        if (ts.isVariableStatement(node)) {
            for (const decl of node.declarationList.declarations) {
                if (!ts.isIdentifier(decl.name) || !decl.initializer) continue;
                const value = unwrap(decl.initializer);
                if (decl.name.text === "namespace") {
                    namespace = stringOf(value) ?? namespace;
                } else if (decl.name.text === "messages" && ts.isObjectLiteralExpression(value)) {
                    messagesObject = value;
                }
            }
        }

        // `defineMessages({ namespace: "common", messages: { ... } })` inline.
        if (ts.isCallExpression(node)) {
            const callee = node.expression;
            const name = ts.isIdentifier(callee) ? callee.text : ts.isPropertyAccessExpression(callee) ? callee.name.text : null;
            if (name === "defineMessages") {
                const arg = unwrap(node.arguments[0]);
                if (arg && ts.isObjectLiteralExpression(arg)) {
                    for (const prop of arg.properties) {
                        if (!ts.isPropertyAssignment(prop)) continue;
                        const key = propertyName(prop);
                        const value = unwrap(prop.initializer);
                        if (key === "namespace" && namespace === null) namespace = stringOf(value);
                        if (key === "messages" && !messagesObject && ts.isObjectLiteralExpression(value)) messagesObject = value;
                        if (key === "dynamic" && value && value.kind === ts.SyntaxKind.TrueKeyword) dynamic = true;
                    }
                }
            }
        }

        ts.forEachChild(node, visit);
    };
    visit(sourceFile);

    if (!messagesObject) {
        problems.push(`${rel}: no \`messages\` object literal found - nothing extracted from this module`);
        return [];
    }
    if (namespace === null) {
        problems.push(`${rel}: no \`namespace\` string found - keys will be treated as root-namespace keys`);
        namespace = "";
    }

    const defined = [];

    for (const prop of messagesObject.properties) {
        if (!ts.isPropertyAssignment(prop)) {
            problems.push(`${rel}:${lineOf(sourceFile, prop)}: skipped a message entry that is not a plain \`key: { text }\` property`);
            continue;
        }
        const localKey = propertyName(prop);
        if (!localKey) {
            problems.push(`${rel}:${lineOf(sourceFile, prop)}: skipped a message entry whose key is computed`);
            continue;
        }

        const value = unwrap(prop.initializer);
        if (!value || !ts.isObjectLiteralExpression(value)) {
            problems.push(`${rel}:${lineOf(sourceFile, prop)}: \`${localKey}\` must be an object literal with a \`text\` field`);
            continue;
        }

        let text = null;
        let description;
        for (const field of value.properties) {
            if (!ts.isPropertyAssignment(field)) continue;
            const name = propertyName(field);
            if (name === "text") text = stringOf(unwrap(field.initializer));
            if (name === "description") description = stringOf(unwrap(field.initializer)) ?? undefined;
        }

        if (text === null) {
            problems.push(`${rel}:${lineOf(sourceFile, prop)}: \`${localKey}\` has no literal \`text\` - skipped`);
            continue;
        }

        defined.push({
            key: fullKey(namespace, localKey),
            namespace,
            source_text: text,
            description,
            placeholders: placeholdersOf(text),
            dynamic,
            file: rel,
            line: lineOf(sourceFile, prop),
        });
    }

    return defined;
}

// ----------------------------------------------------------------- call sites

/**
 * The key a `t(...)` first argument names, or `null` if it cannot be known
 * statically.
 *
 * Two forms are accepted: the plain literal `t("pagination.previous")`, and a
 * reference through the key map `defineMessages` returns -
 * `t(keys.previous)` or `t(m.keys["previous"])` - where the property name IS
 * the key by construction, so it needs no symbol resolution.
 */
function keyFromArgument(node) {
    if (!node) return null;

    const inner = unwrap(node);
    if (ts.isStringLiteralLike(inner)) return inner.text;

    const throughKeyMap = (access) =>
        ts.isIdentifier(access) ? access.text === "keys" : ts.isPropertyAccessExpression(access) ? access.name.text === "keys" : false;

    if (ts.isPropertyAccessExpression(inner) && throughKeyMap(inner.expression)) {
        return inner.name.text;
    }
    if (ts.isElementAccessExpression(inner) && throughKeyMap(inner.expression)) {
        return stringOf(inner.argumentExpression);
    }

    return null;
}

/**
 * Every `t()` call in one file, with the namespace its `t` was bound to.
 *
 * Bindings are collected per file rather than per lexical scope. A component
 * file has one `useT` per component and never rebinds the name to something
 * else, and the alternative - a full type-checked program over ~1000 files -
 * costs seconds per run for an answer that is the same.
 */
function readUsage(file, problems) {
    const sourceFile = parseFile(file);
    const rel = path.relative(FRONTEND, file);

    /** binding identifier -> namespace */
    const bindings = new Map();

    const collectBindings = (node) => {
        if (ts.isVariableDeclaration(node) && ts.isIdentifier(node.name) && node.initializer) {
            const init = unwrap(node.initializer);
            if (init && ts.isCallExpression(init)) {
                const callee = init.expression;
                const name = ts.isIdentifier(callee) ? callee.text : ts.isPropertyAccessExpression(callee) ? callee.name.text : null;
                // `useRichT` is the same contract as `useT` - same namespace
                // prefixing, same keys - it just returns a ReactNode so a
                // sentence containing markup can stay one message. Its call
                // sites have to count as usage, or every sentence collapsed
                // out of `.before`/`.after` fragments reports as a dead key
                // and the missing-key check stops covering them.
                if (name === "useT" || name === "useRichT") {
                    const arg = init.arguments[0];
                    if (!arg) {
                        bindings.set(node.name.text, "");
                    } else {
                        const ns = stringOf(unwrap(arg));
                        if (ns === null) {
                            problems.push(`${rel}:${lineOf(sourceFile, init)}: ${name}() called with a non-literal namespace - its keys cannot be extracted`);
                        } else {
                            bindings.set(node.name.text, ns);
                        }
                    }
                }
            }
        }
        ts.forEachChild(node, collectBindings);
    };
    collectBindings(sourceFile);

    if (bindings.size === 0) return [];

    const uses = [];

    const collectCalls = (node) => {
        if (ts.isCallExpression(node) && ts.isIdentifier(node.expression) && bindings.has(node.expression.text)) {
            const namespace = bindings.get(node.expression.text);
            const key = keyFromArgument(node.arguments[0]);
            const line = lineOf(sourceFile, node);
            if (key === null) {
                problems.push(`${rel}:${line}: ${node.expression.text}() called with a key that is not a literal - not extractable`);
            } else {
                uses.push({ key: fullKey(namespace, key), namespace, file: rel, line });
            }
        }
        ts.forEachChild(node, collectCalls);
    };
    collectCalls(sourceFile);

    return uses;
}

// ----------------------------------------------------------------- extraction

function extract() {
    const problems = [];
    const defined = [];
    const uses = [];

    for (const file of walk(SRC)) {
        if (/\.messages\.tsx?$/.test(file)) {
            defined.push(...readMessagesModule(file, problems));
            continue;
        }
        uses.push(...readUsage(file, problems));
    }

    const byKey = new Map();
    for (const entry of defined) {
        const existing = byKey.get(entry.key);
        if (existing) {
            if (existing.source_text !== entry.source_text) {
                problems.push(`${entry.key}: defined twice with different text (${existing.file}:${existing.line} and ${entry.file}:${entry.line})`);
            }
            continue;
        }
        byKey.set(entry.key, entry);
    }

    const usedKeys = new Set(uses.map((use) => use.key));
    // A dynamic module's keys are resolved through a variable, so their
    // absence from the literal call sites proves nothing.
    const unused = [...byKey.entries()]
        .filter(([key, entry]) => !usedKeys.has(key) && !entry.dynamic)
        .map(([key]) => key)
        .sort();
    const missing = [...new Set(uses.filter((use) => !byKey.has(use.key)).map((use) => `${use.key} (${use.file}:${use.line})`))].sort();

    const entries = [...byKey.values()].sort((a, b) => (a.key < b.key ? -1 : a.key > b.key ? 1 : 0));
    const namespaces = [...new Set(entries.map((entry) => entry.namespace || "(root)"))].sort();

    return { entries, namespaces, unused, missing, problems, useCount: uses.length };
}

// ----------------------------------------------------------------- output

function renderCatalog(entries) {
    const catalog = {};
    for (const entry of entries) catalog[entry.key] = entry.source_text;
    return `${JSON.stringify(catalog, null, 4)}\n`;
}

function metaEntries(entries) {
    return entries.map((entry) => ({
        key: entry.key,
        namespace: entry.namespace,
        source_text: entry.source_text,
        ...(entry.description === undefined ? {} : { description: entry.description }),
        placeholders: entry.placeholders,
    }));
}

/**
 * `generatedAt` is carried over whenever the entries themselves have not
 * moved. A timestamp that changed on every run would make `--check` fail
 * forever and turn every unrelated branch into a one-line diff on this file.
 */
function renderMeta(entries) {
    const payload = metaEntries(entries);

    let generatedAt = new Date().toISOString();
    try {
        const previous = JSON.parse(fs.readFileSync(META_PATH, "utf8"));
        if (previous?.generatedBy === GENERATED_BY && JSON.stringify(previous?.entries) === JSON.stringify(payload) && typeof previous.generatedAt === "string") {
            generatedAt = previous.generatedAt;
        }
    } catch {
        // No readable previous file: this is the first run.
    }

    return `${JSON.stringify({ generatedBy: GENERATED_BY, generatedAt, entries: payload }, null, 4)}\n`;
}

function readIfExists(file) {
    try {
        return fs.readFileSync(file, "utf8");
    } catch {
        return null;
    }
}

// ----------------------------------------------------------------- sync

async function sync(entries) {
    if (entries.length === 0) {
        console.error("refusing to --sync an empty catalog: that would deactivate every key in the database");
        return 1;
    }

    const base = (process.env.BACKEND_URL ?? process.env.VITE_BACKEND_URL ?? "").replace(/\/$/, "");

    // Authenticate with the backend's existing SERVICE_KEY rather than a user
    // token. `extractors/auth.rs` accepts `x-service-key` and grants that
    // request `GlobalRole::SuperAdmin`, which is exactly what the sync route
    // requires - so this needs no new secret, and unlike a JWT it does not
    // expire out from under a deploy script.
    //
    // A service-key caller deliberately cannot author translations: every
    // write route resolves `auth.user_uuid()` for the audit row, and the
    // service principal has no user id. It can only sync the key list, which
    // is the one thing a build step should be able to do.
    const serviceKey = process.env.SERVICE_KEY;
    // A real super-admin JWT still works, for a human running this by hand.
    const bearer = process.env.MYRTLE_ADMIN_TOKEN;

    if (!base) {
        console.error("--sync needs BACKEND_URL (or VITE_BACKEND_URL) in the environment");
        return 1;
    }
    if (!serviceKey && !bearer) {
        console.error("--sync needs SERVICE_KEY in the environment (the same value the backend boots with).");
        console.error("A super-admin JWT in MYRTLE_ADMIN_TOKEN also works.");
        return 1;
    }

    const url = `${base}${SYNC_ENDPOINT}`;
    let res;
    try {
        res = await fetch(url, {
            method: "POST",
            headers: {
                "content-type": "application/json",
                ...(serviceKey ? { "x-service-key": serviceKey } : { authorization: `Bearer ${bearer}` }),
            },
            body: JSON.stringify({ entries: metaEntries(entries) }),
            // A sync that hangs in CI is worse than one that fails: the job
            // would sit there holding a runner until the whole build times out.
            signal: AbortSignal.timeout(30_000),
        });
    } catch (error) {
        console.error(`POST ${url} failed: ${error instanceof Error ? error.message : String(error)}`);
        return 1;
    }

    const body = await res.text();
    if (!res.ok) {
        console.error(`POST ${url} -> ${res.status} ${res.statusText}\n${body}`);
        return 1;
    }

    let result;
    try {
        result = JSON.parse(body);
    } catch {
        console.error(`sync succeeded but the response was not JSON:\n${body}`);
        return 1;
    }

    console.log(`synced ${entries.length} entries: upserted ${result.upserted}, deactivated ${result.deactivated}`);
    return 0;
}

// ----------------------------------------------------------------- main

async function main() {
    const args = new Set(process.argv.slice(2));
    const check = args.has("--check");
    const doSync = args.has("--sync");

    for (const arg of args) {
        if (arg !== "--check" && arg !== "--sync") {
            console.error(`unknown flag ${arg}; usage: node scripts/i18n-extract.mjs [--check | --sync]`);
            process.exitCode = 2;
            return;
        }
    }

    const { entries, namespaces, unused, missing, problems, useCount } = extract();

    const catalog = renderCatalog(entries);
    const meta = renderMeta(entries);

    console.log(`i18n: ${entries.length} entries from ${useCount} t() call sites`);
    console.log(`      namespaces: ${namespaces.length ? namespaces.join(", ") : "(none)"}`);
    console.log(`      unused keys: ${unused.length}`);
    console.log(`      missing keys: ${missing.length}`);

    for (const key of unused) console.warn(`  warn: defined but never used: ${key}`);
    for (const key of missing) console.warn(`  warn: used but never defined: ${key}`);
    for (const problem of problems) console.warn(`  warn: ${problem}`);

    if (check) {
        const stale = [
            [CATALOG_PATH, catalog],
            [META_PATH, meta],
        ].filter(([file, content]) => readIfExists(file) !== content);

        if (stale.length > 0) {
            for (const [file] of stale) console.error(`stale: ${path.relative(FRONTEND, file)}`);
            console.error("run `bun run i18n:extract` and commit the result");
            process.exitCode = 1;
            return;
        }
        console.log("      up to date");
        return;
    }

    fs.writeFileSync(CATALOG_PATH, catalog);
    fs.writeFileSync(META_PATH, meta);
    console.log(`      wrote ${path.relative(FRONTEND, CATALOG_PATH)} and ${path.relative(FRONTEND, META_PATH)}`);

    if (doSync) process.exitCode = await sync(entries);
}

await main();
