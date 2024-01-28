import { db, tableName } from "..";
import { Db, LeaderboardType, PlayerDataDB } from "../../../../types/types";
import { schema } from "./schema";

export const leaderboard = async({ server, type, sort, limit, fields }: { server: string, type?: LeaderboardType, sort?: "asc" | "desc", limit?: number, fields?: string[] }) => {
    if (!type) type = "level";
    if (!sort) sort = "desc";
    if (!limit) limit = 20;

    const params = {
        $limit: limit
    };
    
    if (server) Object.assign(params, { $server: server });

    const results = db
        .query<Db<PlayerDataDB>, { $server?: string, $limit?: number }>(
            `SELECT *${type === "trust" ? ", (SELECT AVG(json_extract(value, '$.favorPoint')) FROM json_each(troop, '$.chars')) as avg_favorPoint" : ""}
             FROM ${tableName} 
             WHERE ${server ? `server = $server` : "1"}
             ORDER BY ${type === "trust" ? "avg_favorPoint" : "CAST(status->>'level' AS INTEGER)"} ${sort}
             LIMIT $limit`,
        )
        .all(params);

    const tableSchema = await schema();
    tableSchema.map((column) => {
        if (column.type === "JSON") {
            results.map((item) => {
                const columnValue = item?.[column.name as keyof PlayerDataDB];
                Object.assign(item ?? {}, {
                    [column.name]: JSON.parse(columnValue ?? "{}"),
                });
            });
        }
    });

    if (fields && fields.length > 0) {
        // Delete fields that don't exist in the fields array
        results.map((item) => {
            Object.keys(item).forEach((key) => {
                if (!fields.includes(key)) {
                    delete (item as { [key: string]: any })[key];
                }
            });
        })
    }

    return results as unknown as PlayerDataDB[];
};

export const leaderboardByOperator = async({ server, operatorId, type, sort, limit }: { server: string, operatorId: string, type?: "", sort?: "asc" | "desc", limit?: number }) => {
    if (!type) type = ""; // TODO
    if (!sort) sort = "desc";
    if (!limit) limit = 20;
};