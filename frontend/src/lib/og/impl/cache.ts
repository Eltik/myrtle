import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { OG_CONFIG } from "./config";

function cachePath(kind: string, id: string, hash: string): string {
    const safeKind = kind.replace(/[^a-z0-9_-]/gi, "_");
    const safeId = id.replace(/[^a-z0-9_-]/gi, "_");
    const safeHash = hash.replace(/[^a-z0-9_-]/gi, "_");
    return path.join(process.cwd(), OG_CONFIG.cacheDir, safeKind, `${safeId}-${safeHash}.png`);
}

export async function readCache(kind: string, id: string, hash: string): Promise<Buffer | null> {
    try {
        return await readFile(cachePath(kind, id, hash));
    } catch {
        return null;
    }
}

export async function writeCache(kind: string, id: string, hash: string, buf: Buffer): Promise<void> {
    const file = cachePath(kind, id, hash);
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, buf);
}
