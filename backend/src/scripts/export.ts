import dotenv from "dotenv";
dotenv.config();

import { listener } from "../events/impl/listener";
import { db } from "../database";

import colors from "colors";

(async () => {
    await listener();

    await db.init();

    const BATCH_SIZE = 100;
    const fileName = "database.json";

    const file = Bun.file(fileName);
    if (await file.exists()) {
        console.log(colors.yellow("WARNING: ") + colors.gray(fileName) + colors.yellow(" already exists!"));

        // Overwrite
        const writer = file.writer();
        writer.write("");
    }

    const exportData = async (tableName: string) => {
        let offset = 0;
        let allRows: any[] = [];

        console.log(colors.yellow(`Exporting ${tableName}...`) + colors.gray(` (batch size: ${BATCH_SIZE})`));

        while (true) {
            const rows = await db.getAll(tableName, BATCH_SIZE.toString(), offset.toString());

            if (rows.length === 0) {
                break;
            }

            allRows = allRows.concat(rows);
            offset += BATCH_SIZE;
        }

        console.log(colors.green(`Exported ${tableName} successfully!`) + colors.gray(` (${allRows.length} rows)`));
        return allRows;
    };

    const toWrite = [];

    for (const table of db.tables) {
        toWrite.push({
            [table.tableName]: await exportData(table.tableName),
        });
    }

    const writer = file.writer();

    writer.write(JSON.stringify(toWrite, null, 4));

    writer.end();

    console.log(colors.green("Exported database successfully!"));
})();
