import { ExcelTables } from "../../../../../../types/impl/lib/impl/local/impl/handler";
import { GAME_DATA_REPOSITORY } from "..";
import { join } from "node:path";

export const download = async (table: ExcelTables) => {
    const data = await (await fetch(`https://raw.githubusercontent.com/${GAME_DATA_REPOSITORY}/main/en_US/gamedata/excel/${table}.json`)).json();
    Bun.write(join(import.meta.dir, `./data/${table}.json`), JSON.stringify(data, null, 4));
    return data;
};
