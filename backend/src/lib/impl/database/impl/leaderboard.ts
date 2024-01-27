import { db, tableName } from "..";
import { Db, LeaderboardType, PlayerDataDB } from "../../../../types/types";
import { schema } from "./schema";

export const leaderboard = async(server: string, type: LeaderboardType = "level", sort: "asc" | "desc" = "desc") => {
    const params = {};
    if (server) Object.assign(params, { $server: server });

    /*
    const results = db
        .query<Db<PlayerDataDB>, { server?: string }>(
            `SELECT ${type === "trust" ? "AVG(favorPoint) as averageFavorPoint" : "*"} FROM ${tableName} WHERE (
                ${server ? `server = $server` : ""}
            ) ORDER BY ${
                type === "level" ?
                    `CAST(status->>'level' AS INTEGER)` :
                type === "trust" ?
                    `averageFavorPoint` : ""
            } ${sort}`,
        )
        .all(params);
        */
    const results = db
        .query<Db<PlayerDataDB>, { server?: string }>(
            `SELECT *,
                (SELECT AVG(json_extract(value, '$.favorPoint')) FROM json_each(troop, '$.chars')) as avg_favorPoint
             FROM ${tableName} 
             WHERE ${server ? `server = $server` : "1"}
             ORDER BY avg_favorPoint ${sort}`,
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

    return results as unknown as PlayerDataDB[];
};