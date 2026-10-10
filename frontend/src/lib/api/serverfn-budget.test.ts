/**
 * Payload budget for server functions that read whole backend tables.
 *
 * A server function's result reaches the browser through `/_serverFn`, where
 * `@tanstack/start-server-core` refuses any result whose seroval output exceeds a fixed
 * 33,554,432 bytes (32 MiB) and answers 500. Nothing in the app can raise that limit. This test
 * keeps every server function that reads a large table well under it, without game data or a
 * network: it reads the source, finds which server functions read which large endpoints, and
 * checks each against the body size last measured by the runtime harness.
 *
 * What it catches: a new or changed server function that reads one of the large endpoints
 * below without an entry here; an entry whose measured size times the worst seroval expansion
 * seen (2.01x) is above the budget; a browser-side importer of a server-only function; a change
 * to a projection's field list that was not re-measured.
 *
 * What it does not catch: growth of the game data itself (the sizes are a snapshot, refreshed
 * by hand from the harness), a large endpoint not listed in LARGE_ENDPOINTS, a large result
 * assembled from many small calls, and per-account growth (gacha records, tier-list favorites).
 */
import { readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

const ROOT = process.cwd();
const SRC = join(ROOT, "src");

/** The limit `start-server-core` applies to one serialized result; fixed in its source. */
const SERVER_FN_LIMIT = 33_554_432;
/** A quarter of the limit, so data growth and CN rows have room before anything breaks. */
const BUDGET = 8 * 1024 * 1024;
/** The largest seroval expansion measured over endpoint JSON (openapi.json; operators 1.58 to 1.69). */
const EXPANSION = 2.01;

/**
 * Backend endpoints whose body is above 1 MB. A server function that reads one of these must
 * have an entry in BUDGETS. `/operators/roster` is read by the browser directly and must never
 * go through a server function, so it has no entry.
 */
const LARGE_ENDPOINTS = ["/static/operators", "/static/voices", "/static/stages", "/static/enemies", "/static/stage-index", "/skins/index", "/static/materials", "/operators/roster"] as const;

/** Helpers that read a large endpoint on a server function's behalf, by the file they live in. */
const INDIRECT_READS: Record<string, { file: string; endpoint: (typeof LARGE_ENDPOINTS)[number] }> = {
    cachedOperatorsList: { file: "src/lib/api/operators.ts", endpoint: "/static/operators" },
};

type Budget = {
    file: string;
    endpoint: (typeof LARGE_ENDPOINTS)[number];
    /** Largest 200-status body seen by the harness (bytes), or the projection's own size. */
    bytes: number;
    /** Never called over `/_serverFn`; may exceed the budget. `importers` are the only files allowed to name it. */
    serverOnly?: { names: string[]; importers: string[] };
    /** The handler returns `pickFields(rows, FIELDS)`; FIELDS must equal this list. */
    projection?: { constName: string; fields: string[] };
};

/** Sizes from report 15 (harness run on 2026-10-10, max of en and cn where both were recorded). */
const BUDGETS: Record<string, Budget> = {
    getOperatorsListFn: { file: "src/lib/api/operators.ts", endpoint: "/static/operators", bytes: 24_412_256, serverOnly: { names: ["getOperatorsListFn"], importers: ["src/lib/og/impl/registry.ts"] } },
    getOperatorsExportFieldsFn: {
        file: "src/lib/api/operators.ts",
        endpoint: "/static/operators",
        bytes: 266_652,
        projection: { constName: "OPERATOR_EXPORT_FIELDS", fields: ["id", "displayNumber", "description", "itemUsage", "itemDesc", "itemObtainApproach", "isSpChar", "maxPotentialLevel", "skin"] },
    },
    getVoicesFn: { file: "src/lib/api/voices.ts", endpoint: "/static/voices", bytes: 17_846_519, serverOnly: { names: ["getVoicesFn", "voicesQueryOptions"], importers: [] } },
    getStagesFn: { file: "src/lib/api/stages.ts", endpoint: "/static/stages", bytes: 3_953_651 },
    getStageIndexFn: { file: "src/lib/api/stages.ts", endpoint: "/static/stage-index", bytes: 1_793_723 },
    getEnemiesFn: { file: "src/lib/api/enemies.ts", endpoint: "/static/enemies", bytes: 2_413_045 },
    getEnemiesListFn: { file: "src/lib/api/enemies.ts", endpoint: "/static/enemies", bytes: 2_413_045 },
    getSkinsIndexFn: { file: "src/lib/api/skins.ts", endpoint: "/skins/index", bytes: 1_275_634 },
    getMaterialsFn: { file: "src/lib/api/materials.ts", endpoint: "/static/materials", bytes: 1_011_412 },
    getItemsListFn: { file: "src/lib/api/materials.ts", endpoint: "/static/materials", bytes: 1_011_412 },
};

function sourceFiles(dir: string): string[] {
    return readdirSync(dir, { recursive: true, encoding: "utf8" })
        .filter((f) => /\.tsx?$/.test(f) && !/\.test\.tsx?$/.test(f))
        .map((f) => join(dir, f));
}

const files = sourceFiles(SRC).map((path) => ({ path: relative(ROOT, path).split("\\").join("/"), text: readFileSync(path, "utf8") }));

/** Each `export const NAME = createServerFn(` up to the next top-level statement or comment. */
function serverFnBlocks(): { name: string; file: string; body: string }[] {
    const out: { name: string; file: string; body: string }[] = [];
    for (const { path, text } of files) {
        for (const m of text.matchAll(/^export const (\w+) = createServerFn\(/gm)) {
            const start = m.index ?? 0;
            const rest = text.slice(start + 1);
            const end = rest.search(/\n(?=[^\s}])/);
            out.push({ name: m[1] ?? "", file: path, body: end === -1 ? rest : rest.slice(0, end) });
        }
    }
    return out;
}

function endpointsRead(body: string): string[] {
    const found = new Set<string>();
    for (const endpoint of LARGE_ENDPOINTS) if (body.includes(`"${endpoint}"`)) found.add(endpoint);
    for (const [helper, { endpoint }] of Object.entries(INDIRECT_READS)) if (body.includes(`${helper}(`)) found.add(endpoint);
    return [...found];
}

const blocks = serverFnBlocks();

describe("server-function payload budget", () => {
    it("stays below the fixed start-server-core limit", () => {
        const handler = readFileSync(join(ROOT, "node_modules/@tanstack/start-server-core/dist/esm/server-functions-handler.js"), "utf8");
        const limit = Number(/MAX_PENDING_SERIALIZATION_BYTES = (\d+)/.exec(handler)?.[1]);
        expect(limit, "start-server-core changed its limit: re-check report 15 and BUDGET").toBe(SERVER_FN_LIMIT);
        expect(BUDGET).toBeLessThan(SERVER_FN_LIMIT);
    });

    it("finds the server functions it budgets", () => {
        expect(blocks.length).toBeGreaterThan(100);
        for (const [name, budget] of Object.entries(BUDGETS)) {
            const block = blocks.find((b) => b.name === name);
            expect(block, `${name} is not a server function any more: drop or move its entry`).toBeDefined();
            expect(block?.file).toBe(budget.file);
            expect(endpointsRead(block?.body ?? ""), `${name} no longer reads ${budget.endpoint}: re-measure it`).toContain(budget.endpoint);
        }
        for (const [helper, { file, endpoint }] of Object.entries(INDIRECT_READS)) {
            const text = files.find((f) => f.path === file)?.text ?? "";
            expect(text).toMatch(new RegExp(`function ${helper}\\(`));
            expect(text).toContain(`"${endpoint}"`);
        }
    });

    it("every server function that reads a large endpoint has a budget entry", () => {
        const unbudgeted = blocks.filter((b) => endpointsRead(b.body).length > 0 && !(b.name in BUDGETS)).map((b) => `${b.name} (${b.file}) reads ${endpointsRead(b.body).join(", ")}`);
        expect(unbudgeted, "add an entry to BUDGETS with a measured size, or return a projection").toEqual([]);
    });

    it("no browser-reachable server function is configured above the budget", () => {
        const over = Object.entries(BUDGETS)
            .filter(([, b]) => !b.serverOnly && b.bytes * EXPANSION > BUDGET)
            .map(([name, b]) => `${name}: ${b.bytes} B x ${EXPANSION} = ${Math.round(b.bytes * EXPANSION)} B > ${BUDGET} B`);
        expect(over).toEqual([]);
    });

    it("server-only functions are named only by their own file and the allowed importers", () => {
        const leaks: string[] = [];
        for (const budget of Object.values(BUDGETS)) {
            if (!budget.serverOnly) continue;
            for (const { path, text } of files) {
                if (path === budget.file || budget.serverOnly.importers.includes(path)) continue;
                for (const name of budget.serverOnly.names) if (new RegExp(`\\b${name}\\b`).test(text)) leaks.push(`${path} names ${name}`);
            }
        }
        expect(leaks, "a browser caller of a whole-table server function gets a 500 above 32 MiB: use a projection").toEqual([]);
    });

    it("projections return the reviewed field list and nothing more", () => {
        for (const [name, budget] of Object.entries(BUDGETS)) {
            if (!budget.projection) continue;
            const { constName, fields } = budget.projection;
            const block = blocks.find((b) => b.name === name);
            expect(block?.body, `${name} must return pickFields(..., ${constName})`).toMatch(new RegExp(`pickFields\\([^;]*\\b${constName}\\)`));
            const text = files.find((f) => f.path === budget.file)?.text ?? "";
            const decl = new RegExp(`const ${constName} = \\[([^\\]]*)\\] as const`).exec(text)?.[1] ?? "";
            const declared = [...decl.matchAll(/"(\w+)"/g)].map((m) => m[1] ?? "");
            expect(declared, `${constName} changed: re-measure ${name} and update its entry`).toEqual(fields);
        }
    });
});
