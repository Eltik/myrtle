import { db, tableName } from "..";
import { type Db, type LeaderboardType, type PlayerDataDB } from "../../../../types/types";

export const leaderboard = async ({ server, type, sort, limit, fields }: { server: string; type?: LeaderboardType; sort?: "asc" | "desc"; limit?: number; fields?: string[] }) => {
    if (!type) type = "level";
    if (!sort) sort = "desc";
    if (!limit) limit = 20;

    const params = {
        $limit: limit,
    };

    if (server) Object.assign(params, { $server: server });

    const results = db
        .query<Db<PlayerDataDB>, { $server?: string; $limit?: number }>(
            `SELECT *${type === "trust" ? ", (SELECT AVG(json_extract(value, '$.favorPoint')) FROM json_each(troop, json_extract(data, '$.chars'))) as avg_favorPoint" : ""}
             FROM ${tableName} 
             WHERE ${server ? `server = $server` : "1"}
             ORDER BY ${type === "trust" ? "avg_favorPoint" : "CAST(json_extract(data, '$.status.level') AS INTEGER)"} ${sort}
             LIMIT $limit`,
        )
        .all(params);

    for (const result of results) {
        Object.assign(result, { data: JSON.parse(result.data) });

        Object.entries(result.data).forEach(([key, value]) => {
            Object.assign(result, { [key]: value });
        });

        delete (result as any).data;
    }

    if (fields && fields.length > 0) {
        // Delete fields that don't exist in the fields array
        results.map((item) => {
            Object.keys(item).forEach((key) => {
                if (!fields.includes(key)) {
                    delete (item as { [key: string]: any })[key];
                }
            });
        });
    }

    return results as unknown as PlayerDataDB[];
};

export const leaderboardByOperator = async ({ server, operatorId, type, sort, limit }: { server: string; operatorId: string; type?: ""; sort?: "asc" | "desc"; limit?: number }) => {
    if (!type) type = ""; // TODO
    if (!sort) sort = "desc";
    if (!limit) limit = 20;

    console.log(server, operatorId);
};
