import { processCharsForFrontend } from "../lib/impl/local/impl/gamedata/impl/chibi/impl/process";
import { isCacheValid, loadFromCache, saveToCache } from "../lib/impl/local/impl/gamedata/impl/chibi/impl/caching";
import type { RepoItem } from "../types/impl/lib/impl/local/impl/gamedata/impl/chibis";

// Mock directory structure for testing
const mockItems: RepoItem[] = [
    {
        name: "amiya1",
        path: "amiya1",
        contentType: "directory",
        children: [
            {
                name: "back",
                path: "amiya1/back",
                contentType: "directory",
                children: [
                    {
                        name: "char_002_amiya_winter_1.atlas",
                        path: "amiya1/back/char_002_amiya_winter_1.atlas",
                        contentType: "file",
                    },
                    {
                        name: "char_002_amiya_winter_1.png",
                        path: "amiya1/back/char_002_amiya_winter_1.png",
                        contentType: "file",
                    },
                    {
                        name: "char_002_amiya_winter_1.skel",
                        path: "amiya1/back/char_002_amiya_winter_1.skel",
                        contentType: "file",
                    },
                ],
            },
            {
                name: "front",
                path: "amiya1/front",
                contentType: "directory",
                children: [
                    {
                        name: "char_002_amiya_winter_1.atlas",
                        path: "amiya1/front/char_002_amiya_winter_1.atlas",
                        contentType: "file",
                    },
                    {
                        name: "char_002_amiya_winter_1.png",
                        path: "amiya1/front/char_002_amiya_winter_1.png",
                        contentType: "file",
                    },
                    {
                        name: "char_002_amiya_winter_1.skel",
                        path: "amiya1/front/char_002_amiya_winter_1.skel",
                        contentType: "file",
                    },
                ],
            },
        ],
    },
    {
        name: "amiya2",
        path: "amiya2",
        contentType: "directory",
        children: [
            {
                name: "build",
                path: "amiya2/build",
                contentType: "directory",
                children: [
                    {
                        name: "char_002_amiya_epoque_1.atlas",
                        path: "amiya2/build/char_002_amiya_epoque_1.atlas",
                        contentType: "file",
                    },
                    {
                        name: "char_002_amiya_epoque_1.png",
                        path: "amiya2/build/char_002_amiya_epoque_1.png",
                        contentType: "file",
                    },
                    {
                        name: "char_002_amiya_epoque_1.skel",
                        path: "amiya2/build/char_002_amiya_epoque_1.skel",
                        contentType: "file",
                    },
                ],
            },
        ],
    },
    {
        name: "amiya3",
        path: "amiya3",
        contentType: "directory",
        children: [
            {
                name: "char_002_amiya_test_1.atlas",
                path: "amiya3/char_002_amiya_test_1.atlas",
                contentType: "file",
            },
            {
                name: "char_002_amiya_test_1.png",
                path: "amiya3/char_002_amiya_test_1.png",
                contentType: "file",
            },
            {
                name: "char_002_amiya_test_1.skel",
                path: "amiya3/char_002_amiya_test_1.skel",
                contentType: "file",
            },
        ],
    },
    {
        name: "kal1",
        path: "kal1",
        contentType: "directory",
        children: [
            {
                name: "front",
                path: "kal1/front",
                contentType: "directory",
                children: [
                    {
                        name: "char_003_kalts_1.atlas",
                        path: "kal1/front/char_003_kalts_1.atlas",
                        contentType: "file",
                    },
                    {
                        name: "char_003_kalts_1.png",
                        path: "kal1/front/char_003_kalts_1.png",
                        contentType: "file",
                    },
                    {
                        name: "char_003_kalts_1.skel",
                        path: "kal1/front/char_003_kalts_1.skel",
                        contentType: "file",
                    },
                ],
            },
        ],
    },
    {
        name: "kal2",
        path: "kal2",
        contentType: "directory",
        children: [
            {
                name: "dorm",
                path: "kal2/dorm",
                contentType: "directory",
                children: [
                    {
                        name: "char_003_kalts_2.atlas",
                        path: "kal2/dorm/char_003_kalts_2.atlas",
                        contentType: "file",
                    },
                    {
                        name: "char_003_kalts_2.png",
                        path: "kal2/dorm/char_003_kalts_2.png",
                        contentType: "file",
                    },
                    {
                        name: "char_003_kalts_2.skel",
                        path: "kal2/dorm/char_003_kalts_2.skel",
                        contentType: "file",
                    },
                ],
            },
        ],
    },
];

async function main() {
    console.log("Starting chibi processing test...");

    try {
        // Test caching functionality
        console.log("\nTesting cache functionality...");

        // Check if cache is valid
        console.log("Checking cache validity...");
        const isValid = await isCacheValid();
        console.log(`Cache is ${isValid ? "valid" : "invalid"}`);

        // Try loading from cache
        console.log("\nTrying to load from cache...");
        const cachedItems = await loadFromCache();
        console.log(`Loaded ${cachedItems.length} items from cache`);

        // Save mock items to cache
        console.log("\nSaving mock items to cache...");
        await saveToCache(mockItems);

        // Load from cache again
        console.log("\nLoading from cache again...");
        const newCachedItems = await loadFromCache();
        console.log(`Loaded ${newCachedItems.length} items from cache`);

        // Process the mock items
        console.log("\nProcessing mock items...");
        const processedItems = processCharsForFrontend(mockItems);
        console.log(`Processed ${processedItems.length} characters`);

        // Log the processed data
        console.log("\nProcessed character data:");
        console.log(JSON.stringify(processedItems, null, 2));

        console.log("\nTest completed successfully!");
    } catch (error) {
        console.error("Test failed:", error);
    }
}

main();
