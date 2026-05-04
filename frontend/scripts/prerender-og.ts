#!/usr/bin/env bun
/**
 * Build-time OG image warmup.
 *
 * Iterates every kind in the OG registry that exposes a `listIds()` and
 * pre-renders its images to the on-disk cache. Resources without `listIds()`
 * (e.g. user profiles) are rendered lazily on first request instead.
 *
 * Run manually:    bun run scripts/prerender-og.ts
 * Or postbuild:    add "postbuild": "bun run scripts/prerender-og.ts" to package.json scripts.
 *
 * Safe to run repeatedly — already-cached entries (matching hash) are skipped,
 * so this is fast on subsequent runs and only does work for new/changed data.
 */
import { stat } from "node:fs/promises";
import path from "node:path";
import { readCache, writeCache } from "../src/lib/og/impl/cache";
import { OG_CONFIG } from "../src/lib/og/impl/config";
import { type IAnyOgHandler, ogRegistry } from "../src/lib/og/impl/registry";
import { renderOgPng } from "../src/lib/og/impl/render";

const CONCURRENCY = 6;

async function prerenderKind(kind: string) {
    const handler = (ogRegistry as Record<string, IAnyOgHandler>)[kind];
    if (!handler.listIds) {
        console.log(`[og] skip ${kind} (no listIds)`);
        return;
    }
    const ids = await handler.listIds();
    console.log(`[og] ${kind}: ${ids.length} ids`);

    let done = 0;
    let rendered = 0;
    let cached = 0;
    let failed = 0;

    async function worker(slice: string[]) {
        for (const id of slice) {
            try {
                const data = await handler.fetch(id);
                if (!data) {
                    failed++;
                    continue;
                }
                const hash = handler.hash(data);
                const existing = await readCache(kind, id, hash);
                if (existing) {
                    cached++;
                } else {
                    const png = await renderOgPng(handler.template(data));
                    await writeCache(kind, id, hash, png);
                    rendered++;
                }
            } catch (err) {
                failed++;
                const msg = err instanceof Error ? err.message.split("\n")[0] : String(err);
                console.error(`[og] ${kind}/${id} failed: ${msg}`);
            } finally {
                done++;
                if (done % 25 === 0) console.log(`  ${done}/${ids.length}`);
            }
        }
    }

    const slices = Array.from({ length: CONCURRENCY }, (_, i) => ids.filter((_, idx) => idx % CONCURRENCY === i));
    await Promise.all(slices.map(worker));

    console.log(`[og] ${kind}: rendered=${rendered} cached=${cached} failed=${failed}`);
}

async function main() {
    const cacheRoot = path.join(process.cwd(), OG_CONFIG.cacheDir);
    try {
        await stat(cacheRoot);
    } catch {
        // Created lazily by writeCache.
    }

    for (const kind of Object.keys(ogRegistry)) {
        await prerenderKind(kind);
    }
}

main().catch((err) => {
    console.error(err);
    process.exit(1);
});
