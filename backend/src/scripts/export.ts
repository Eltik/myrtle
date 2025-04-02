import dotenv from "dotenv";
dotenv.config();

import { listener } from "../events/impl/listener";
import { db } from "../database";
import fs from "fs/promises";
import path from "path";

import colors from "colors";

const BATCH_SIZE = 1000;
const EXPORT_DIR = path.join(process.cwd(), "exports");

function formatDuration(ms: number): string {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    
    if (hours > 0) return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
    if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
    return `${seconds}s`;
}

function sanitizeData(data: any): any {
    if (data === null || data === undefined) return null;
    if (typeof data === 'object') {
        if (Array.isArray(data)) {
            return data.map(item => sanitizeData(item));
        }
        const sanitized: any = {};
        for (const [key, value] of Object.entries(data)) {
            // Ensure key is a string
            const stringKey = String(key);
            sanitized[stringKey] = sanitizeData(value);
        }
        return sanitized;
    }
    return data;
}

(async () => {
    console.log(colors.cyan("\n=== Starting Database Export ===\n"));
    const startTime = Date.now();
    
    await listener();
    await db.init();

    await fs.mkdir(EXPORT_DIR, { recursive: true });
    console.log(colors.blue(`Export directory: ${EXPORT_DIR}`));

    let totalRows = 0;
    for (const table of db.tables) {
        console.log(colors.yellow(`\nProcessing table: ${table.tableName}`));
        const tableStartTime = Date.now();
        const exportPath = path.join(EXPORT_DIR, `${table.tableName}.json`);
        const allData: any[] = [];
        let offset = 0;
        let hasMore = true;
        let batchCount = 0;

        while (hasMore) {
            const batch = await db.getAll(table.tableName, BATCH_SIZE.toString(), offset.toString());
            if (batch.length === 0) {
                hasMore = false;
                break;
            }
            allData.push(...batch);
            offset += batch.length;
            batchCount++;
            
            // Show progress every 5 batches
            if (batchCount % 5 === 0) {
                console.log(colors.gray(`  Fetched ${allData.length} rows so far...`));
            }
        }

        // Sanitize and validate the data before writing
        const sanitizedData = sanitizeData(allData);
        
        // Validate JSON serialization
        try {
            JSON.stringify(sanitizedData);
        } catch (error: any) {
            console.error(colors.red(`  ❌ Error: Data from ${table.tableName} contains invalid JSON structure`));
            console.error(colors.red(`  Details: ${error.message}`));
            continue;
        }

        await fs.writeFile(exportPath, JSON.stringify(sanitizedData, null, 2));
        const tableDuration = Date.now() - tableStartTime;
        console.log(colors.green(`  ✓ Exported ${allData.length} rows to ${exportPath}`));
        console.log(colors.gray(`  ⏱️  Table processing time: ${formatDuration(tableDuration)}`));
        totalRows += allData.length;
    }

    const totalDuration = Date.now() - startTime;
    console.log(colors.cyan("\n=== Export Summary ==="));
    console.log(colors.green(`✓ Total rows exported: ${totalRows}`));
    console.log(colors.green(`✓ Total tables processed: ${db.tables.length}`));
    console.log(colors.green(`✓ Total time: ${formatDuration(totalDuration)}`));
    console.log(colors.cyan("=== Export Complete ===\n"));
})();
