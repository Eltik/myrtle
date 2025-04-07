import type { GitHubCommit } from "../../../../types/impl/lib/impl/cron";
import colors from "colors";
import { stat } from "node:fs/promises";
import { join } from "path";
import { env } from "../../../../env";

const MAX_TIME_DIFFERENCE = 5 * 60 * 1000; // 5 minutes in milliseconds

export const checkRepoUpdate = async (owner: string, repo: string): Promise<{ updated: boolean; lastUpdate: Date; commit: string }> => {
    try {
        const headers = env.GITHUB_TOKEN ? { Authorization: `Bearer ${env.GITHUB_TOKEN}` } : undefined;
        const response = await fetch(`https://api.github.com/repos/${owner}/${repo}/commits/main`, { headers });
        if (!response.ok) throw new Error(`Failed to fetch ${owner}/${repo}`);

        const data = (await response.json()) as GitHubCommit;
        const lastUpdate = new Date(data.commit.author.date);
        const now = new Date();

        // Check if the data directory exists and get its timestamp
        const dataDir = join(process.cwd(), "data");
        try {
            const stats = await stat(dataDir);
            const fileUpdateTime = stats.mtime;

            // Calculate time difference between file update and repository update
            const timeDifference = Math.abs(fileUpdateTime.getTime() - lastUpdate.getTime());

            // Consider it updated if:
            // 1. The repository was updated within the last hour AND
            // 2. The file update time is significantly different from the repository update time
            const updated = now.getTime() - lastUpdate.getTime() < 60 * 60 * 1000 && timeDifference > MAX_TIME_DIFFERENCE;

            console.log(colors.gray(`Repository ${owner}/${repo}:`));
            console.log(colors.gray(`  Last repository update: ${lastUpdate.toLocaleString()}`));
            console.log(colors.gray(`  Last file update: ${fileUpdateTime.toLocaleString()}`));
            console.log(colors.gray(`  Time difference: ${Math.round(timeDifference / 1000)} seconds`));
            console.log(colors.gray(`  Status: ${updated ? colors.yellow("Update needed") : colors.green("Up to date")}`));

            return {
                updated,
                lastUpdate,
                commit: data.sha.substring(0, 7),
            };
        } catch (error: unknown) {
            // If data directory doesn't exist or can't be accessed, consider it needs update
            const errorMessage = error instanceof Error ? error.message : "Unknown error";
            console.log(colors.yellow(`Data directory not found or inaccessible for ${owner}/${repo}: ${errorMessage}`));
            return {
                updated: true,
                lastUpdate,
                commit: data.sha.substring(0, 7),
            };
        }
    } catch (error) {
        console.error(colors.red(`Error checking repository ${owner}/${repo}:`), error);
        return {
            updated: false,
            lastUpdate: new Date(),
            commit: "unknown",
        };
    }
};
