import { Pool, PoolClient } from "pg";
import { z } from "zod";
import { env } from "../env";
import type { ColumnMetadata, FlattenedKeys, JsonPath } from "../types/impl/database";
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

    public tables = [
        {
            tableName: usersTableName,
            schema: userSchema,
        },
    ];

    async init() {
        for (const table of this.tables) {
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
            const { rows: existingColumns } = await client.query(
                `SELECT column_name, data_type 
                 FROM information_schema.columns 
                 WHERE table_name = $1`,
                [tableName],
            );

            const existingColumnNames = existingColumns.map((col: { column_name: string }) => col.column_name);
            const existingColumnTypes = existingColumns.reduce((acc: Record<string, string>, col: { column_name: string; data_type: string }) => {
                acc[col.column_name] = col.data_type;
                return acc;
            }, {});

            for (const [key, value] of Object.entries(parsedSchema)) {
                const sqlType = DatabaseHandler.mapZodToSQL(value);
                const currentType = existingColumnTypes[key];

                if (!existingColumnNames.includes(key) || currentType?.toUpperCase() !== sqlType) {
                    const metadata = (value as any).dbMetadata as ColumnMetadata | undefined;
                    let alterQuery = `ALTER TABLE ${tableName} `;

                    if (!existingColumnNames.includes(key)) {
                        alterQuery += `ADD COLUMN ${key} ${sqlType}`;
                    } else if (currentType?.toUpperCase() !== sqlType) {
                        alterQuery += `ALTER COLUMN ${key} TYPE ${sqlType} USING ${key}::${sqlType}`;
                    }

                    if (metadata?.notNull && !existingColumnNames.includes(key)) alterQuery += " NOT NULL";
                    if (metadata?.defaultValue !== undefined && !existingColumnNames.includes(key)) alterQuery += ` DEFAULT '${metadata.defaultValue}'`;

                    await client.query(alterQuery);
                }
            }
        } finally {
            client.release();
        }
    }

    // Generic CRUD operations
    async create<T extends object>(tableName: string, data: T, conflictKey?: string) {
        const keys = Object.keys(data);
        const values = Object.values(data);
        const placeholders = keys.map((_, index) => `$${index + 1}`).join(", ");

        let query = `INSERT INTO ${tableName} (${keys.join(", ")}) VALUES (${placeholders})`;
        if (conflictKey) {
            query += ` ON CONFLICT (${conflictKey}) DO NOTHING`;
        }
        query += ` RETURNING *`;

        const client = await this.getClient();
        try {
            const result = await client.query(query, values);
            return result.rows[0] ?? null;
        } finally {
            client.release();
        }
    }

    async search<T>(
        tableName: string,
        filters: {
            conditions?: Partial<Pick<T, FlattenedKeys<T>>>;
            jsonConditions?: {
                column: Extract<keyof T, string>; // Column must exist in T
                path: JsonPath<T[keyof T]>;
                operator: "=" | "LIKE"; // Comparison operator
                value: any; // Value to compare against
            }[]; // Array of JSON-based conditions
        },
        fields?: string[]
    ): Promise<T[]> {
        const { conditions, jsonConditions = [] } = filters;

        const flatConditions = Object.entries(conditions || {}).map(([key], index) => `${key} = $${index + 1}`);

        const jsonClauses = jsonConditions.map((jsonCondition, index) => {
            const path = (jsonCondition.path as string[]).map((part) => `'${part}'`).join("->");
            const operator = jsonCondition.operator;
            const placeholder = `$${flatConditions.length + index + 1}`;
            if (operator === "LIKE") {
                return `(${jsonCondition.column}->${path})::text ${operator} ${placeholder}`;
            }
            return `${jsonCondition.column}->${path} ${operator} ${placeholder}`;
        });

        const whereClause = [...flatConditions, ...jsonClauses].join(" AND ");
        const values = [...Object.values(conditions || {}), ...jsonConditions.map((cond) => cond.value)];

        const query = `SELECT * FROM ${tableName} ${whereClause ? `WHERE ${whereClause}` : ""}`;
        const client = await this.getClient();

        try {
            const result = await client.query(query, values);
            const data = result.rows;

            for (const item of data) {
                if (fields && fields.length > 0) {
                    // Delete fields that don't exist in the fields array
                    Object.keys(item).forEach((key) => {
                        if (!fields.includes(key)) {
                            delete (item as { [key: string]: any })[key];
                        }
                    });
                }
            }

            return data;
        } finally {
            client.release();
        }
    }

    async read<T>(tableName: string, conditions: Partial<T> = {}, fields?: string[]) {
        const keys = Object.keys(conditions);
        const values = Object.values(conditions);

        const whereClause = keys.length ? `WHERE ${keys.map((key, index) => `${key} = $${index + 1}`).join(" AND ")}` : "";

        const query = `SELECT * FROM ${tableName} ${whereClause}`;
        const client = await this.getClient();
        try {
            const result = await client.query(query, values);
            const data = result.rows;
            
            for (const item of data) {
                if (fields && fields.length > 0) {
                    // Delete fields that don't exist in the fields array
                    Object.keys(item).forEach((key) => {
                        if (!fields.includes(key)) {
                            delete (item as { [key: string]: any })[key];
                        }
                    });
                }
            }

            return data;
        } finally {
            client.release();
        }
    }

    async update<T>(tableName: string, conditions: Partial<T>, updates: Partial<T>) {
        const conditionKeys = Object.keys(conditions);
        const conditionValues = Object.values(conditions);

        const updateKeys = Object.keys(updates);
        const updateValues = Object.values(updates);

        const whereClause = `WHERE ${conditionKeys.map((key, index) => `${key} = $${index + 1}`).join(" AND ")}`;
        const setClause = `SET ${updateKeys.map((key, index) => `${key} = $${conditionKeys.length + index + 1}`).join(", ")}`;

        const query = `UPDATE ${tableName} ${setClause} ${whereClause} RETURNING *`;
        const client = await this.getClient();
        try {
            const result = await client.query(query, [...conditionValues, ...updateValues]);
            return result.rows[0];
        } finally {
            client.release();
        }
    }

    async delete<T>(tableName: string, conditions: Partial<T>) {
        const keys = Object.keys(conditions);
        const values = Object.values(conditions);

        const whereClause = keys.map((key, index) => `${key} = $${index + 1}`).join(" AND ");

        const query = `DELETE FROM ${tableName} WHERE ${whereClause} RETURNING *`;
        const client = await this.getClient();
        try {
            const result = await client.query(query, values);
            return result.rows[0];
        } finally {
            client.release();
        }
    }

    async getAll(tableName: string, limit?: string, offset?: string) {
        const query = `SELECT * FROM ${tableName} ${limit ? `LIMIT ${limit}` : ""} ${offset ? `OFFSET ${offset}` : ""}`;
        const client = await this.getClient();
        try {
            const result = await client.query(query);
            return result.rows;
        } finally {
            client.release();
        }
    }

    async deleteAll(tableName: string) {
        const query = `DELETE FROM ${tableName}`;
        const client = await this.getClient();
        try {
            await client.query(query);
        } finally {
            client.release();
        }
    }
}

export const db = new DatabaseHandler(env.DATABASE_URL);
