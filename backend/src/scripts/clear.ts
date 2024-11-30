import dotenv from "dotenv";
dotenv.config();

import { listener } from "../events/impl/listener";
import { db } from "../database";

import colors from "colors";

(async () => {
    await listener();

    await db.init();

    for (const table of db.tables) {
        await db.deleteAll(table.tableName);

        console.log(colors.green(`Cleared table: ${table.tableName}`));
    }

    console.log(colors.green("Database cleared!"));
})();
