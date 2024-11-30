import dotenv from "dotenv";
dotenv.config();

import { listener } from "../events/impl/listener";
import { db } from "../database";

import colors from "colors";

(async () => {
    await listener();

    await db.init();

    const fileName = "database.json";

    const file = Bun.file(fileName);
    if (!(await file.exists())) {
        throw new Error("Database file does not exist!");
    }

    const json = (await file.json()) as {
        [tableName: string]: any[];
    }[];

    console.log(colors.green("Successfully parsed data! Importing..."));

    for (const data of json) {
        const tableName = Object.keys(data)[0];
        const rows = data[tableName];

        console.log(colors.yellow(`Importing ${tableName}...`) + colors.gray(` (${rows.length} rows)`));

        for (const row of rows) {
            await db.create(tableName, row);
        }

        console.log(colors.green(`Imported ${tableName} successfully!`));
    }

    console.log(colors.green("Imported database successfully!"));
})();
