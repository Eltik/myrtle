import { load } from "cheerio";
import { delay } from "..";
import { FEXLI_REPOSITORY } from "../../..";
import type { RepoItem } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";

// Rate limit manager to track GitHub API limits
class RateLimitManager {
    private remaining: number = 60; // Default GitHub unauthenticated rate limit
    private resetTime: number = Date.now() + 3600000; // Default reset after 1 hour
    private baseDelay: number = 50; // Minimum delay in ms between requests
    private maxDelay: number = 10000; // Maximum delay in ms
    private consecutiveErrors: number = 0;

    updateFromHeaders(headers: { get(name: string): string | null }): void {
        const remaining = headers.get("x-ratelimit-remaining");
        const reset = headers.get("x-ratelimit-reset");

        if (remaining) this.remaining = parseInt(remaining, 10);
        if (reset) this.resetTime = parseInt(reset, 10) * 1000; // Convert to milliseconds

        // If we got valid headers, reset consecutive errors
        this.consecutiveErrors = 0;
    }

    async handleRateLimit(): Promise<void> {
        // If plenty of requests remaining, use minimal delay
        if (this.remaining > 20) {
            await delay(this.baseDelay);
            return;
        }

        // If close to rate limit, calculate delay based on reset time
        if (this.remaining > 5) {
            const now = Date.now();
            const timeToReset = Math.max(0, this.resetTime - now);
            const delayTime = Math.min(this.maxDelay, timeToReset / (this.remaining + 1));
            await delay(delayTime);
            return;
        }

        // If very few requests left or if error occurred, use exponential backoff
        const backoffDelay = Math.min(this.maxDelay, this.baseDelay * Math.pow(2, this.consecutiveErrors));
        await delay(backoffDelay);
    }

    // Called when a rate limit error occurs
    handleError(): void {
        this.consecutiveErrors++;
        this.remaining = 0;
    }
}

// Create a singleton rate limit manager
const rateLimitManager = new RateLimitManager();

// Crawl a specific GitHub directory and return its contents
export const crawlDirectory = async (path: string): Promise<RepoItem[]> => {
    try {
        // Handle rate limiting based on current status
        await rateLimitManager.handleRateLimit();

        const url = `https://github.com/${FEXLI_REPOSITORY}/tree/main/${path}`;
        console.log(`Crawling: ${url}`);

        const response = await fetch(url);

        // Update rate limit info from response headers
        rateLimitManager.updateFromHeaders(response.headers);

        if (!response.ok) {
            if (response.status === 429) {
                // Rate limit exceeded
                rateLimitManager.handleError();
                console.warn(`Rate limit exceeded for ${url}, backing off and retrying...`);
                // Retry with backoff
                return crawlDirectory(path);
            }
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
                repoItem.children = await crawlDirectory(repoItem.path);
            }

            result.push(repoItem);
        }

        return result;
    } catch (error) {
        console.error(`Error crawling ${path}:`, error);
        return [];
    }
};
