import { delay } from "..";
import type { RepoItem } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/chibis";
import colors from "colors";
import fs from "fs";
import path from "path";
import { env } from "../../../../../../../../env";

// Repository URL
export const CHIBI_REPOSITORY = "HermitzPlanner/chibi-assets";

// Get GitHub token from environment variable
const GITHUB_TOKEN = env.GITHUB_TOKEN;

// Path to store the chibi data
const CHIBI_DATA_PATH = path.join(process.cwd(), "data", "chibi-data.json");

// Ensure the data directory exists
const ensureDataDir = () => {
    const dataDir = path.dirname(CHIBI_DATA_PATH);
    if (!fs.existsSync(dataDir)) {
        fs.mkdirSync(dataDir, { recursive: true });
    }
};

// Load existing chibi data if available
const loadExistingData = (): RepoItem[] => {
    ensureDataDir();
    if (fs.existsSync(CHIBI_DATA_PATH)) {
        try {
            const data = fs.readFileSync(CHIBI_DATA_PATH, "utf8");
            return JSON.parse(data) as RepoItem[];
        } catch (error) {
            console.error(colors.red(`❌ Error loading existing chibi data: ${error}`));
            return [];
        }
    }
    return [];
};

// Save chibi data to file
const saveChibiData = (data: RepoItem[]): void => {
    ensureDataDir();
    try {
        fs.writeFileSync(CHIBI_DATA_PATH, JSON.stringify(data, null, 2));
        console.log(colors.green(`✅ Saved chibi data to ${CHIBI_DATA_PATH}`));
    } catch (error) {
        console.error(colors.red(`❌ Error saving chibi data: ${error}`));
    }
};

// Merge two arrays of RepoItem, avoiding duplicates based on path
const mergeChibiData = (existing: RepoItem[], newData: RepoItem[]): RepoItem[] => {
    const merged = [...existing];
    const pathMap = new Map<string, RepoItem>();

    // Create a map of existing items by path
    existing.forEach((item) => {
        pathMap.set(item.path, item);
    });

    // Merge new items
    newData.forEach((item) => {
        if (pathMap.has(item.path)) {
            // Item exists, merge children if it's a directory
            const existingItem = pathMap.get(item.path)!;
            if (item.contentType === "dir" && item.children) {
                if (!existingItem.children) {
                    existingItem.children = [];
                }
                // Merge children recursively
                existingItem.children = mergeChibiData(existingItem.children, item.children);
            }
        } else {
            // New item, add to merged array
            merged.push(item);
            pathMap.set(item.path, item);
        }
    });

    return merged;
};

// Simple rate limit manager
class RateLimitManager {
    private remaining: number = GITHUB_TOKEN ? 5000 : 60;
    private resetTime: number = Date.now() + 3600000;
    private responseCache: Map<string, { data: any; timestamp: number }> = new Map();
    private cacheExpiry: number = 5 * 60 * 1000; // 5 minutes
    private totalRequests: number = 0;
    private successfulRequests: number = 0;
    private failedRequests: number = 0;
    private isWaiting: boolean = false;
    private isPaused: boolean = false;
    private currentData: RepoItem[] = [];
    private currentPath: string = "";

    updateFromHeaders(headers: { get(name: string): string | null }): void {
        const remaining = headers.get("x-ratelimit-remaining");
        const reset = headers.get("x-ratelimit-reset");

        if (remaining) this.remaining = parseInt(remaining, 10);
        if (reset) this.resetTime = parseInt(reset, 10) * 1000;

        // Log rate limit status
        this.logRateLimit();
    }

    logRateLimit(): void {
        const resetTime = new Date(this.resetTime).toLocaleTimeString();
        const waitTime = Math.max(0, this.resetTime - Date.now());
        const waitMinutes = Math.ceil(waitTime / 60000);

        if (this.remaining < 10) {
            console.log(colors.yellow(`⚠️ Rate limit: ${this.remaining} requests remaining, resets at ${resetTime} (${waitMinutes} minutes)`));
        } else {
            console.log(colors.magenta(`⏱️ Rate limit: ${this.remaining} requests remaining, resets at ${resetTime}`));
        }
    }

    async waitIfNeeded(): Promise<void> {
        // If we're already waiting, don't create a new wait
        if (this.isWaiting) {
            return;
        }

        // If we have plenty of requests remaining, no need to wait
        if (this.remaining > 10) {
            return;
        }

        // If we're close to the rate limit, wait until reset
        if (this.remaining <= 5) {
            const waitTime = Math.max(0, this.resetTime - Date.now()) + 1000; // Add 1 second buffer
            const waitMinutes = Math.ceil(waitTime / 60000);

            this.isWaiting = true;
            console.log(colors.yellow(`⚠️ Rate limit low. Waiting ${waitMinutes} minutes for reset...`));

            await delay(waitTime);

            this.isWaiting = false;
            this.remaining = GITHUB_TOKEN ? 5000 : 60; // Reset to default
            console.log(colors.green("✅ Rate limit reset complete. Resuming requests."));
        }
    }

    getCachedResponse(url: string): any | null {
        const cached = this.responseCache.get(url);
        if (cached && Date.now() - cached.timestamp < this.cacheExpiry) {
            return cached.data;
        }
        return null;
    }

    cacheResponse(url: string, data: any): void {
        this.responseCache.set(url, { data, timestamp: Date.now() });
    }

    incrementRequests(success: boolean): void {
        this.totalRequests++;
        if (success) {
            this.successfulRequests++;
        } else {
            this.failedRequests++;
        }

        // Update progress
        this.updateProgress();
    }

    updateProgress(): void {
        const percent = Math.round((this.successfulRequests / this.totalRequests) * 100);
        const bar = "█".repeat(Math.floor(percent / 5)) + "░".repeat(20 - Math.floor(percent / 5));

        // Clear the current line and write the new progress
        process.stdout.write("\r" + colors.blue(`📊 Progress: [${bar}] ${percent}% (${this.successfulRequests}/${this.totalRequests})`));
    }

    getStats(): { total: number; successful: number; failed: number; successRate: number } {
        const successRate = this.totalRequests > 0 ? Math.round((this.successfulRequests / this.totalRequests) * 100) : 0;
        return {
            total: this.totalRequests,
            successful: this.successfulRequests,
            failed: this.failedRequests,
            successRate,
        };
    }

    // Store current crawling state
    setCurrentState(path: string, data: RepoItem[]): void {
        this.currentPath = path;
        this.currentData = data;
    }

    // Get current crawling state
    getCurrentState(): { path: string; data: RepoItem[] } {
        return {
            path: this.currentPath,
            data: this.currentData,
        };
    }

    // Pause crawling
    pauseCrawling(): void {
        this.isPaused = true;
    }

    // Resume crawling
    resumeCrawling(): void {
        this.isPaused = false;
    }

    // Check if crawling is paused
    isCrawlingPaused(): boolean {
        return this.isPaused;
    }
}

// Create a singleton rate limit manager
const rateLimitManager = new RateLimitManager();

// GitHub API interface for directory contents
interface GitHubContent {
    name: string;
    path: string;
    sha: string;
    size: number;
    url: string;
    html_url: string;
    git_url: string;
    download_url: string | null;
    type: "file" | "dir";
    _links: {
        self: string;
        git: string;
        html: string;
    };
}

// Fetch a URL with rate limiting and caching
async function fetchWithRateLimit(url: string): Promise<Response> {
    // Check cache first
    const cachedData = rateLimitManager.getCachedResponse(url);
    if (cachedData) {
        console.log(colors.cyan(`ℹ️ Using cached data for: ${url}`));
        return new Response(JSON.stringify(cachedData), {
            status: 200,
            headers: new Headers({
                "content-type": "application/json",
                "x-ratelimit-remaining": "60",
                "x-ratelimit-reset": Math.floor((Date.now() + 3600000) / 1000).toString(),
            }),
        });
    }

    // Wait if we're close to rate limit
    await rateLimitManager.waitIfNeeded();

    console.log(colors.cyan(`ℹ️ Fetching: ${url}`));

    // Add authentication headers if token is available
    const headers: Record<string, string> = {
        Accept: "application/vnd.github.v3+json",
    };

    if (GITHUB_TOKEN) {
        headers["Authorization"] = `token ${GITHUB_TOKEN}`;
    }

    try {
        const response = await fetch(url, { headers });

        // Update rate limit info from response headers
        rateLimitManager.updateFromHeaders(response.headers);

        if (!response.ok) {
            if (response.status === 403 && response.headers.get("x-ratelimit-remaining") === "0") {
                // Rate limit exceeded
                const resetTime = response.headers.get("x-ratelimit-reset");

                if (resetTime) {
                    const resetTimestamp = parseInt(resetTime, 10) * 1000;
                    const waitTime = Math.max(0, resetTimestamp - Date.now()) + 1000;
                    const waitMinutes = Math.ceil(waitTime / 60000);

                    console.log(colors.yellow(`⚠️ Rate limit exceeded. Waiting ${waitMinutes} minutes for reset...`));

                    // Store current data and pause crawling
                    const { path, data } = rateLimitManager.getCurrentState();
                    const existingData = loadExistingData();
                    const mergedData = mergeChibiData(existingData, data);
                    saveChibiData(mergedData);

                    console.log(colors.yellow(`⏸️ Crawling paused. Current path: ${path}`));
                    console.log(colors.yellow(`ℹ️ To resume crawling later, use: crawlDirectory("${path}")`));

                    // Pause crawling
                    rateLimitManager.pauseCrawling();

                    // Wait for reset
                    await delay(waitTime);

                    // Resume crawling
                    rateLimitManager.resumeCrawling();

                    // Retry after waiting
                    return fetchWithRateLimit(url);
                }
            }

            rateLimitManager.incrementRequests(false);
            throw new Error(`Failed to fetch ${url}: ${response.status} ${response.statusText}`);
        }

        // Cache the response
        const responseClone = response.clone();
        const responseData = await responseClone.json();
        rateLimitManager.cacheResponse(url, responseData);

        // Increment successful requests
        rateLimitManager.incrementRequests(true);

        return response;
    } catch (error) {
        rateLimitManager.incrementRequests(false);
        console.error(colors.red(`❌ Error fetching ${url}: ${error}`));
        throw error;
    }
}

// Crawl a specific GitHub directory and return its contents
export const crawlDirectory = async (path: string): Promise<RepoItem[]> => {
    try {
        // Check if we're resuming from a previous crawl
        const existingData = loadExistingData();
        if (existingData.length > 0) {
            console.log(colors.cyan(`ℹ️ Found existing chibi data with ${existingData.length} items`));
        }

        const url = `https://api.github.com/repos/${CHIBI_REPOSITORY}/contents/${path}`;

        console.log(colors.cyan(`ℹ️ Crawling directory: ${path}`));

        // Fetch the directory contents
        const response = await fetchWithRateLimit(url);
        const contents = (await response.json()) as GitHubContent[];

        console.log(colors.green(`✅ Found ${contents.length} items in ${path}`));

        // Process each item
        const result: RepoItem[] = [];

        for (const item of contents) {
            const repoItem: RepoItem = {
                name: item.name,
                path: item.path,
                contentType: item.type,
            };

            // If it's a directory, recursively crawl it
            if (item.type === "dir") {
                console.log(colors.gray(`🔍 Processing directory: ${item.path}`));
                try {
                    // Update current state for potential pause
                    rateLimitManager.setCurrentState(item.path, result);

                    const children = await crawlDirectory(item.path);
                    repoItem.children = children;
                    console.log(colors.green(`✅ Completed directory: ${item.path} (${children.length} items)`));
                } catch (error) {
                    console.error(colors.red(`❌ Error crawling directory ${item.path}: ${error}`));
                    // Continue with the item without children
                }
            } else {
                console.log(colors.gray(`🔍 Found file: ${item.path}`));
            }

            result.push(repoItem);
        }

        // Log statistics
        const stats = rateLimitManager.getStats();
        console.log(colors.cyan(`ℹ️ Crawl statistics: ${stats.successful}/${stats.total} successful (${stats.successRate}%)`));

        // Save the data after each directory is processed
        const mergedData = mergeChibiData(existingData, result);
        saveChibiData(mergedData);

        return result;
    } catch (error) {
        console.error(colors.red(`❌ Error crawling ${path}: ${error}`));
        return [];
    }
};
