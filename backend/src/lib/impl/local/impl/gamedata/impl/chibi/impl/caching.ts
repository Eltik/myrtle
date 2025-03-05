import { resolve, join } from "node:path";
import { mkdir, stat } from "node:fs/promises";
import type { CachedData, RepoItem } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";

const CACHE_DIR = resolve(process.cwd(), "data/cache");
const CACHE_FILE = join(CACHE_DIR, "chibi-data.json");
const CACHE_DURATION = 7 * 24 * 60 * 60 * 1000; // 1 week in milliseconds

// Ensure cache directory exists
const ensureCacheDir = async () => {
    try {
        await mkdir(CACHE_DIR, { recursive: true });
    } catch (error) {
        console.error("Failed to create cache directory:", error);
    }
};

// Check if cache is valid
export const isCacheValid = async (): Promise<boolean> => {
    try {
        const file = Bun.file(CACHE_FILE);
        const exists = await file.exists();

        if (!exists) return false;

        // Get file stats using fs/promises
        const fileInfo = await stat(CACHE_FILE);
        const fileDate = fileInfo.mtime.getTime();
        const now = Date.now();

        // Cache is valid if it's less than CACHE_DURATION old
        return now - fileDate < CACHE_DURATION;
    } catch (error) {
        // If file doesn't exist or can't be accessed, cache is invalid
        console.error("Error checking cache validity:", error);
        return false;
    }
};

// Load data from cache
export const loadFromCache = async (): Promise<RepoItem[]> => {
    try {
        const file = Bun.file(CACHE_FILE);
        const text = await file.text();
        const parsed = JSON.parse(text) as CachedData;
        console.log(`Loaded chibi data from cache (created ${new Date(parsed.timestamp).toLocaleString()})`);
        return parsed.data;
    } catch (error) {
        console.error("Failed to load from cache:", error);
        return [];
    }
};

// Save data to cache
export const saveToCache = async (data: RepoItem[]): Promise<void> => {
    try {
        await ensureCacheDir();
        const cacheData: CachedData = {
            timestamp: Date.now(),
            data,
        };
        const jsonString = JSON.stringify(cacheData, null, 2);
        await Bun.write(CACHE_FILE, jsonString);
        console.log("Chibi data saved to cache");
    } catch (error) {
        console.error("Failed to save to cache:", error);
    }
};
