import { mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { OG_CONFIG } from "./config";

const IS_SERVERLESS = !!(process.env.VERCEL || process.env.AWS_LAMBDA_FUNCTION_NAME || process.env.NETLIFY || process.env.CF_PAGES);
const CACHE_ROOT = IS_SERVERLESS ? path.join(tmpdir(), "myrtle-og-cache") : path.join(process.cwd(), OG_CONFIG.cacheDir);

function cachePath(kind: string, id: string, hash: string): string {
    const safe = (s: string) => s.replace(/[^a-z0-9_-]/gi, "_");
    return path.join(CACHE_ROOT, safe(kind), `${safe(id)}-${safe(hash)}.png`);
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
    } catch (err) {
        console.warn(`[og] cache write failed for ${kind}/${id}:`, err);
    }
}
