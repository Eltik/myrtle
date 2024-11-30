import { ExcelTables } from "../../../../../../types/impl/lib/impl/local/impl/handler";
import { join } from "node:path";

export const exists = async (table: ExcelTables) => {
    const file = Bun.file(join(import.meta.dir, `./data/${table}.json`));
    return await file.exists();
};
