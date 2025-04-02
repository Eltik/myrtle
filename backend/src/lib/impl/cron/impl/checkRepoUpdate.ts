import type { GitHubCommit } from "../../../../types/impl/lib/impl/cron";
import colors from "colors";

export const checkRepoUpdate = async (owner: string, repo: string): Promise<{ updated: boolean; lastUpdate: Date; commit: string }> => {
    try {
        const response = await fetch(`https://api.github.com/repos/${owner}/${repo}/commits/main`);
        if (!response.ok) throw new Error(`Failed to fetch ${owner}/${repo}`);

        const data = (await response.json()) as GitHubCommit;
        const lastUpdate = new Date(data.commit.author.date);
        const now = new Date();

        // Consider it updated if the last commit was within the last hour
        const updated = now.getTime() - lastUpdate.getTime() < 60 * 60 * 1000;

        return {
            updated,
            lastUpdate,
            commit: data.sha.substring(0, 7),
        };
    } catch (error) {
        console.error(colors.red(`Error checking repository ${owner}/${repo}:`), error);
        return {
            updated: false,
            lastUpdate: new Date(),
            commit: "unknown",
        };
    }
};
