import { mkdir, mkdtemp, readdir, rm, utimes, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { ogCacheMaxBytes, pruneCache } from "./cache";

const MB = 1024 * 1024;

let root: string;
beforeEach(async () => {
    root = await mkdtemp(path.join(tmpdir(), "og-cache-test-"));
});
afterEach(async () => {
    await rm(root, { recursive: true, force: true });
});

/** Write `<root>/<kind>/<name>` of `bytes`, rendered `ageMin` minutes ago. */
async function put(kind: string, name: string, bytes: number, ageMin: number): Promise<void> {
    const file = path.join(root, kind, name);
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, Buffer.alloc(bytes));
    const at = new Date(Date.now() - ageMin * 60_000);
    await utimes(file, at, at);
}

async function left(kind: string): Promise<string[]> {
    return (await readdir(path.join(root, kind))).sort();
}

describe("og cache size limit", () => {
    it("reads OG_CACHE_MAX_MB, defaulting when unset, blank or unusable", () => {
        expect(ogCacheMaxBytes(undefined)).toBe(4096 * MB);
        expect(ogCacheMaxBytes("  ")).toBe(4096 * MB);
        expect(ogCacheMaxBytes("lots")).toBe(4096 * MB);
        expect(ogCacheMaxBytes("-5")).toBe(4096 * MB);
        expect(ogCacheMaxBytes("512")).toBe(512 * MB);
        expect(ogCacheMaxBytes("0")).toBe(0);
    });

    it("leaves a cache under the limit alone", async () => {
        await put("user", "a-1.png", 400, 30);
        await put("user", "a-2.png", 400, 10);
        expect(await pruneCache(root, 1000)).toEqual({ removed: 0, freedBytes: 0, totalBytes: 800 });
        expect(await left("user")).toEqual(["a-1.png", "a-2.png"]);
    });

    it("removes the oldest renders first, across kinds, down to 80% of the limit", async () => {
        await put("user", "u-old.png", 300, 90);
        await put("operator", "o-older.png", 300, 120);
        await put("user", "u-mid.png", 300, 60);
        await put("tier-list", "t-new.png", 300, 5);
        // 1200 bytes against a limit of 1000: prune to <= 800, so two files go.
        expect(await pruneCache(root, 1000)).toEqual({ removed: 2, freedBytes: 600, totalBytes: 1200 });
        expect(await left("operator")).toEqual([]);
        expect(await left("user")).toEqual(["u-mid.png"]);
        expect(await left("tier-list")).toEqual(["t-new.png"]);
    });

    it("never touches anything but <kind>/*.png", async () => {
        await put("user", "big.png", 900, 10);
        await put("user", "notes.txt", 900, 999);
        await put("user/nested", "deep.png", 900, 999);
        await writeFile(path.join(root, "stray.png"), Buffer.alloc(900));
        const result = await pruneCache(root, 100);
        expect(result).toEqual({ removed: 1, freedBytes: 900, totalBytes: 900 });
        expect(await left("user")).toEqual(["nested", "notes.txt"]);
        expect(await left("user/nested")).toEqual(["deep.png"]);
        expect(await readdir(root)).toContain("stray.png");
    });

    it("does nothing with pruning off or no cache directory yet", async () => {
        await put("user", "a.png", 500, 10);
        expect((await pruneCache(root, 0)).removed).toBe(0);
        expect(await pruneCache(path.join(root, "missing"), 100)).toEqual({ removed: 0, freedBytes: 0, totalBytes: 0 });
    });
});
