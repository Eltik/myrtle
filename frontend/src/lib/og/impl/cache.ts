import { mkdir, readdir, readFile, stat, unlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { OG_CONFIG } from "./config";

const IS_SERVERLESS = !!(process.env.VERCEL || process.env.AWS_LAMBDA_FUNCTION_NAME || process.env.NETLIFY || process.env.CF_PAGES);
const CACHE_ROOT = IS_SERVERLESS ? path.join(tmpdir(), "myrtle-og-cache") : path.join(process.cwd(), OG_CONFIG.cacheDir);

// The cache is content-addressed (`<id>-<hash>.png`), so nothing in it is ever
// overwritten: every profile sync, tier-list edit and design bump adds a file
// and leaves the one it replaced behind. Unbounded, it reached 15 GB (31,683
// files) on the production box and was one of the things between that disk and
// full on 2026-11-11. A file here is only a second-level cache (the response is
// `immutable` for a year at the CDN, and a miss just renders again), so the
// oldest ones can go.
//
// OG_CACHE_MAX_MB is the size the directory is pruned back under; `0` turns
// pruning off. Unset, blank or unusable takes the default.
const DEFAULT_MAX_MB = 4096;
// How far under the limit a prune goes, so one over-the-limit write does not
// trigger a full walk on every write after it.
const PRUNE_TARGET = 0.8;
// The walk stats every cached file, so it runs at most this often.
const PRUNE_EVERY_MS = 10 * 60 * 1000;

export function ogCacheMaxBytes(raw: string | undefined = process.env.OG_CACHE_MAX_MB): number {
    const parsed = raw === undefined || raw.trim() === "" ? DEFAULT_MAX_MB : Number(raw);
    const mb = Number.isFinite(parsed) && parsed >= 0 ? parsed : DEFAULT_MAX_MB;
    return Math.round(mb * 1024 * 1024);
}

let lastPruneAt = 0;
let pruning = false;

function cachePath(kind: string, id: string, hash: string): string {
    const safe = (s: string) => s.replace(/[^a-z0-9_-]/gi, "_");
    return path.join(CACHE_ROOT, safe(kind), `${safe(id)}-${safe(hash)}.png`);
}

/**
 * Delete the oldest cached images until the cache is back under `maxBytes`
 * (down to `PRUNE_TARGET` of it). Oldest by mtime, which is when the image was
 * rendered: a card nobody has re-rendered in the longest goes first.
 *
 * Only `<root>/<kind>/*.png` is touched, which is all `writeCache` ever
 * creates. Every step is best effort: a file that vanished mid-walk or cannot
 * be removed is skipped, and the next prune sees the directory as it then is.
 */
export async function pruneCache(root: string, maxBytes: number): Promise<{ removed: number; freedBytes: number; totalBytes: number }> {
    const files: { file: string; size: number; mtimeMs: number }[] = [];
    let totalBytes = 0;

    const kinds = await readdir(root, { withFileTypes: true }).catch(() => []);
    for (const kind of kinds) {
        if (!kind.isDirectory()) continue;
        const dir = path.join(root, kind.name);
        const names = await readdir(dir).catch(() => [] as string[]);
        for (const name of names) {
            if (!name.endsWith(".png")) continue;
            const file = path.join(dir, name);
            const st = await stat(file).catch(() => null);
            if (!st?.isFile()) continue;
            files.push({ file, size: st.size, mtimeMs: st.mtimeMs });
            totalBytes += st.size;
        }
    }

    let removed = 0;
    let freedBytes = 0;
    if (maxBytes <= 0 || totalBytes <= maxBytes) return { removed, freedBytes, totalBytes };

    const target = maxBytes * PRUNE_TARGET;
    files.sort((a, b) => a.mtimeMs - b.mtimeMs);
    for (const { file, size } of files) {
        if (totalBytes - freedBytes <= target) break;
        try {
            await unlink(file);
            removed += 1;
            freedBytes += size;
        } catch {
            // Already gone, or not ours to remove: leave it for the next pass.
        }
    }
    return { removed, freedBytes, totalBytes };
}

/** Prune in the background after a write, at most once per `PRUNE_EVERY_MS`. */
function schedulePrune(): void {
    const maxBytes = ogCacheMaxBytes();
    if (maxBytes <= 0 || pruning || Date.now() - lastPruneAt < PRUNE_EVERY_MS) return;
    pruning = true;
    lastPruneAt = Date.now();
    void pruneCache(CACHE_ROOT, maxBytes)
        .then(({ removed, freedBytes }) => {
            if (removed > 0) console.info(`[og] cache pruned: ${removed} file(s), ${Math.round(freedBytes / (1024 * 1024))} MB freed`);
        })
        .catch((err) => console.warn("[og] cache prune failed:", err))
        .finally(() => {
            pruning = false;
        });
}

export async function readCache(kind: string, id: string, hash: string): Promise<Buffer | null> {
    try {
        return await readFile(cachePath(kind, id, hash));
    } catch {
        return null;
    }
}

export async function writeCache(kind: string, id: string, hash: string, buf: Buffer): Promise<void> {
    try {
        const file = cachePath(kind, id, hash);
        await mkdir(path.dirname(file), { recursive: true });
        await writeFile(file, buf);
        schedulePrune();
    } catch (err) {
        console.warn(`[og] cache write failed for ${kind}/${id}:`, err);
    }
}
