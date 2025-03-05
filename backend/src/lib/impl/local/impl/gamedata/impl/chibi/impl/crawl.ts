import { load } from "cheerio";
import { delay } from "..";
import { FEXLI_REPOSITORY } from "../../..";
import type { RepoItem } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";

// Crawl a specific GitHub directory and return its contents
export const crawlDirectory = async (path: string, delayMs: number = 1000): Promise<RepoItem[]> => {
    try {
        // Add delay to avoid rate limiting
        await delay(delayMs);

        const url = `https://github.com/${FEXLI_REPOSITORY}/tree/main/${path}`;
        console.log(`Crawling: ${url}`);

        const response = await fetch(url);
        if (!response.ok) {
            throw new Error(`Failed to fetch ${url}: ${response.status} ${response.statusText}`);
        }

        const html = await response.text();
        const $ = load(html);
        const json = $("script[data-target='react-app.embeddedData']").text();

        // Parse the JSON data
        const data = JSON.parse(json);
        const items = data.payload.tree.items;

        // Process each item
        const result: RepoItem[] = [];
        for (const item of items) {
            const repoItem: RepoItem = {
                name: item.name,
                path: `${path}/${item.name}`.replace(/^\//, ""),
                contentType: item.contentType,
            };

            // If it's a directory, recursively crawl it
            if (item.contentType === "directory") {
                repoItem.children = await crawlDirectory(repoItem.path, delayMs);
            }

            result.push(repoItem);
        }

        return result;
    } catch (error) {
        console.error(`Error crawling ${path}:`, error);
        return [];
    }
};
