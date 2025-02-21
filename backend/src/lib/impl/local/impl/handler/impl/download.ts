import { ExcelTables } from "../../../../../../types/impl/lib/impl/local/impl/handler";
import { GAME_DATA_REPOSITORY } from "..";
import { join } from "node:path";

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

async function fetchWithRetry(url: string, maxRetries = 3, initialDelay = 1000): Promise<any> {
    let lastError;
    for (let attempt = 0; attempt < maxRetries; attempt++) {
        try {
            const response = await fetch(url, {
                headers: {
                    Accept: "application/json",
                    "User-Agent": "Mozilla/5.0 (compatible; MyrtleMoe/1.0;)",
                },
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data = await response.json();
            return data;
        } catch (error) {
            lastError = error;
            if (attempt < maxRetries - 1) {
                // Exponential backoff with jitter
                const delay = initialDelay * Math.pow(2, attempt) * (0.5 + Math.random());
                await sleep(delay);
            }
        }
    }
    throw lastError;
}

export const download = async (table: ExcelTables) => {
    try {
        const url = `https://raw.githubusercontent.com/${GAME_DATA_REPOSITORY}/main/en_US/gamedata/excel/${table}.json`;
        const data = await fetchWithRetry(url);
        await Bun.write(join(import.meta.dir, `./data/${table}.json`), JSON.stringify(data, null, 4));
        return data;
    } catch (error) {
        console.error(`Failed to download ${table} from ${GAME_DATA_REPOSITORY}:`, error);
        throw error;
    }
};
