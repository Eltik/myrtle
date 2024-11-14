import emitter, { Events } from "../../../../../events";
import { ExcelTables } from "../../../../../types/impl/lib/impl/local/impl/handler";
import { download } from "./impl/download";
import { exists } from "./impl/exists";

export const GAME_DATA_REPOSITORY = "Kengxxiao/ArknightsGameData_YoStar";

export const init = async () => {
    const promises: Promise<void>[] = [];

    const keys = Object.values(ExcelTables);
    for (const key of keys) {
        promises.push(
            new Promise(async (resolve) => {
                const downloaded = await exists(key as ExcelTables);
                if (!downloaded) {
                    try {
                        await download(key as ExcelTables);
                        await emitter.emit(Events.LOCAL_TABLES_DOWNLOADED, {
                            name: key,
                        });
                    } catch {
                        console.error(`Failed to download ${key} from ${GAME_DATA_REPOSITORY}`);
                    }
                }
                resolve();
            }),
        );
    }

    await Promise.all(promises);

    await emitter.emit(Events.LOCAL_TABLES_INITIATED);
};
