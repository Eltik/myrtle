import { ExcelTables } from "../../../../../../types/impl/lib/impl/local/impl/handler";
import { join } from "node:path";
import { exists } from "./exists";

export const get = async (table: ExcelTables) => {
    if (!(await exists(table))) return null;

    const file = Bun.file(join(process.cwd(), "data", `${table}.json`));

    return await file.json();
};
