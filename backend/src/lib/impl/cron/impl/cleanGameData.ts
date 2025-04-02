import { download } from "../../local/impl/handler/impl/download";
import { rm } from "node:fs/promises";
import { join } from "path";
import colors from "colors";
import { ExcelTables } from "../../../../types/impl/lib/impl/local/impl/handler";
import { init as initChibis } from "../../local/impl/gamedata/impl/chibi";
import { updateStaticData } from "../../local/impl/handler";

export const cleanAndRedownloadGameData = async () => {
    try {
        console.log(colors.yellow("Starting data cleanup and redownload..."));

        // Delete all existing data files
        const dataDir = join(process.cwd(), "data");
        await rm(dataDir, { recursive: true, force: true });
        console.log(colors.gray("Cleaned existing data directory"));

        // Redownload all tables
        console.log(colors.yellow("Downloading game data tables..."));
        for (const table of Object.values(ExcelTables)) {
            await download(table as ExcelTables);
            console.log(colors.gray(`Downloaded table: ${table}`));
        }

        // Update static data
        await updateStaticData();

        // Reinitialize chibis
        console.log(colors.yellow("Reinitializing chibi data..."));
        await initChibis();

        console.log(colors.green("Successfully cleaned and redownloaded game data"));
    } catch (error) {
        console.error(colors.red("Error cleaning and redownloading game data:"), error);
    }
};
