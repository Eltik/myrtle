// https://github.com/fexli/ArknightsResource/tree/main/spine/char_002_amiya/char_002_amiya
// https://flashmercurymcfly.github.io/Arknights-SD-Viewer/
import type { RepoItem } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";
import { isCacheValid, loadFromCache, saveToCache } from "./impl/caching";
import { crawlDirectory } from "./impl/crawl";

export const CHIBIS: RepoItem[] = [];

// Delay to avoid rate limiting
export const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export const getAll = () => {
    return CHIBIS;
};

export const init = async () => {
    try {
        // Check if we have a valid cache
        if (await isCacheValid()) {
            // Load from cache
            const cachedData = await loadFromCache();
            if (cachedData.length > 0) {
                CHIBIS.length = 0; // Clear current data
                CHIBIS.push(...cachedData);
                return CHIBIS;
            }
        }

        // If cache is invalid or empty, crawl the repository
        console.log("Cache invalid or expired, recrawling GitHub repository...");
        const items = await crawlDirectory("spine", 1000);

        // Store the results
        CHIBIS.length = 0; // Clear current data
        CHIBIS.push(...items);
        console.log(`Successfully crawled ${CHIBIS.length} root items`);

        // Save to cache for future use
        await saveToCache(CHIBIS);

        return CHIBIS;
    } catch (error) {
        console.error("Failed to initialize chibi data:", error);
        throw error;
    }
};

export default (id: string) => {
    // Find an item by ID (assuming ID is the path or name)
    const findById = (items: RepoItem[], id: string): RepoItem | undefined => {
        for (const item of items) {
            if (item.path === id || item.name === id) {
                return item;
            }

            if (item.children) {
                const found = findById(item.children, id);
                if (found) return found;
            }
        }
        return undefined;
    };

    return findById(CHIBIS, id);
};
