import dotenv from "dotenv";
dotenv.config();

import { listener } from "../events/impl/listener";
import { db } from "../database";
import fs from "fs/promises";
import path from "path";

import colors from "colors";

function formatDuration(ms: number): string {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);

    if (hours > 0) return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
    if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
    return `${seconds}s`;
}

(async () => {
    console.log(colors.cyan("\n=== Starting Database Import ===\n"));
    const startTime = Date.now();

    await listener();
    await db.init();

    const importDir = path.join(process.cwd(), "exports");

    try {
        await fs.access(importDir);
    } catch {
        console.error(colors.red("❌ No exports directory found. Please run export first."));
        process.exit(1);
    }
    console.log(colors.blue(`Import directory: ${importDir}`));

    let totalRows = 0;
    let successCount = 0;
    let errorCount = 0;

    for (const table of db.tables) {
        console.log(colors.yellow(`\nProcessing table: ${table.tableName}`));
        const tableStartTime = Date.now();
        const importPath = path.join(importDir, `${table.tableName}.json`);

        try {
            const data = JSON.parse(await fs.readFile(importPath, "utf-8"));

            // Clear existing data
            await db.deleteAll(table.tableName);
            console.log(colors.gray(`  Cleared existing data`));

            // Import new data
            const batchSize = 100;
            for (let i = 0; i < data.length; i += batchSize) {
                const batch = data.slice(i, i + batchSize);
                for (const item of batch) {
                    await db.create(table.tableName, item);
                }

                // Show progress every 5 batches
                if (i % (batchSize * 5) === 0) {
                    console.log(colors.gray(`  Imported ${Math.min(i + batchSize, data.length)}/${data.length} rows...`));
                }
            }

            const tableDuration = Date.now() - tableStartTime;
            console.log(colors.green(`  ✓ Imported ${data.length} rows from ${importPath}`));
            console.log(colors.gray(`  ⏱️  Table processing time: ${formatDuration(tableDuration)}`));
            totalRows += data.length;
            successCount++;
        } catch (error: any) {
            console.error(colors.red(`  ❌ Error importing ${table.tableName}: ${error.message}`));
            errorCount++;
        }
    }

    const totalDuration = Date.now() - startTime;
    console.log(colors.cyan("\n=== Import Summary ==="));
    console.log(colors.green(`✓ Total rows imported: ${totalRows}`));
    console.log(colors.green(`✓ Successfully processed tables: ${successCount}/${db.tables.length}`));
    console.log(colors.red(`✗ Failed tables: ${errorCount}`));
    console.log(colors.green(`✓ Total time: ${formatDuration(totalDuration)}`));
    console.log(colors.cyan("=== Import Complete ===\n"));
})();
