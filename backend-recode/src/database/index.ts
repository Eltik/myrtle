import { Pool, PoolClient } from "pg";
import { z } from "zod";
import { env } from "../env";
import type { ColumnMetadata } from "../types/impl/database";
import { tableName as usersTableName, userSchema } from "./impl/users";
import emitter, { Events } from "../events";

export function withDbMetadata<T extends z.ZodTypeAny>(schema: T, metadata: ColumnMetadata): T & { dbMetadata: ColumnMetadata } {
    return Object.assign(schema, { dbMetadata: metadata });
}

class DatabaseHandler {
    private pool: Pool;

    constructor(connectionString: string) {
        this.pool = new Pool({ connectionString });
    }

    async init() {
        const users = {
            tableName: usersTableName,
            schema: userSchema,
        };

        for (const table of [users]) {
            await this.createTable(table.tableName, table.schema);
            await this.syncTable(table.tableName, table.schema);
        }

        await emitter.emit(Events.DATABASE_INITIATED);
    }

    private async getClient(): Promise<PoolClient> {
        return this.pool.connect();
    }

    // Map Zod types to SQL types
    private static mapZodToSQL(zodType: z.ZodTypeAny): string {
        if (zodType instanceof z.ZodString) return "TEXT";
        if (zodType instanceof z.ZodNumber) return "NUMERIC";
        if (zodType instanceof z.ZodBoolean) return "BOOLEAN";
        if (zodType instanceof z.ZodDate) return "TIMESTAMP";
        if (zodType instanceof z.ZodArray) return "JSONB";
        if (zodType instanceof z.ZodObject) return "JSONB";
        if (zodType instanceof z.ZodAny) return "JSONB";

        throw new Error(`Unsupported Zod type: ${zodType.constructor.name}`);
    }

    // Dynamically create a table based on Zod schema
    async createTable<T extends z.ZodRawShape>(tableName: string, schema: z.ZodObject<T>) {
        const parsedSchema = schema.shape;
        const columnDefinitions = Object.entries(parsedSchema)
            .map(([key, value]) => {
                const sqlType = DatabaseHandler.mapZodToSQL(value);
                const metadata = (value as any).dbMetadata as ColumnMetadata | undefined;

                let definition = `${key} ${sqlType}`;
                if (metadata?.primaryKey) definition += " PRIMARY KEY";
                if (metadata?.notNull) definition += " NOT NULL";
                if (metadata?.unique) definition += " UNIQUE";
                if (metadata?.references) definition += ` REFERENCES ${metadata.references.table}(${metadata.references.column})`;
                if (metadata?.defaultValue !== undefined) definition += ` DEFAULT '${metadata.defaultValue}'`;
                return definition;
            })
            .join(", ");

        const createQuery = `CREATE TABLE IF NOT EXISTS ${tableName} (${columnDefinitions})`;
        const client = await this.getClient();
        try {
            await client.query(createQuery);
        } finally {
            client.release();
        }

        await emitter.emit(Events.DATABASE_TABLE_CREATE, tableName);
    }

    // Sync schema by altering table if the Zod schema changes
    async syncTable<T extends z.ZodRawShape>(tableName: string, schema: z.ZodObject<T>) {
        const parsedSchema = schema.shape;
        const client = await this.getClient();

        try {
            const { rows: existingColumns } = await client.query(`SELECT column_name FROM information_schema.columns WHERE table_name = $1`, [tableName]);

            const existingColumnNames = existingColumns.map((col: { column_name: string }) => col.column_name);

            for (const [key, value] of Object.entries(parsedSchema)) {
                if (!existingColumnNames.includes(key)) {
                    const sqlType = DatabaseHandler.mapZodToSQL(value);
                    const metadata = (value as any).dbMetadata as ColumnMetadata | undefined;
                    let alterQuery = `ALTER TABLE ${tableName} ADD COLUMN ${key} ${sqlType}`;

                    if (metadata?.notNull) alterQuery += " NOT NULL";
                    if (metadata?.defaultValue !== undefined) alterQuery += ` DEFAULT '${metadata.defaultValue}'`;

                    await client.query(alterQuery);
                }
            }
        } finally {
            client.release();
        }
    }

    // Create a user
    async createRecord(tableName: string, data: Record<string, any>) {
        const client = await this.getClient();
        try {
            const columns = Object.keys(data).join(", ");
            const values = Object.values(data);
            const placeholders = values.map((_, index) => `$${index + 1}`).join(", ");
            const insertQuery = `INSERT INTO ${tableName} (${columns}) VALUES (${placeholders}) RETURNING *`;

            const result = await client.query(insertQuery, values);
            return result.rows[0];
        } finally {
            client.release();
        }
    }

    // Update a user
    async updateRecord(tableName: string, id: number, data: Record<string, any>) {
        const client = await this.getClient();
        try {
            const updates = Object.keys(data)
                .map((key, index) => `${key} = $${index + 1}`)
                .join(", ");
            const values = [...Object.values(data), id];
            const updateQuery = `UPDATE ${tableName} SET ${updates} WHERE id = $${values.length} RETURNING *`;

            const result = await client.query(updateQuery, values);
            return result.rows[0];
        } finally {
            client.release();
        }
    }

    // Delete a user
    async deleteRecord(tableName: string, id: number) {
        const client = await this.getClient();
        try {
            const deleteQuery = `DELETE FROM ${tableName} WHERE id = $1 RETURNING *`;

            const result = await client.query(deleteQuery, [id]);
            return result.rows[0];
        } finally {
            client.release();
        }
    }
}

export const db = new DatabaseHandler(env.DATABASE_URL);
