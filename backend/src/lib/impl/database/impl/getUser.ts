import { db, tableName } from "..";
import type { Db, PlayerData, PlayerDataDB } from "../../../../types/types";
import { schema } from "./schema";

export const getUser = async (uid: string, server: string): Promise<PlayerDataDB | null> => {
    const data = db
        .query<Db<PlayerData>, { $uid: string; $server: string }>(
            `SELECT * FROM ${tableName}
            WHERE uid = $uid
            AND server = $server
    `,
        )
        .get({ $uid: uid, $server: server });

    const tableSchema = await schema();

    tableSchema.map((column) => {
        if (column.type === "JSON") {
            const columnValue = data?.[column.name as keyof PlayerData];
            Object.assign(data ?? {}, {
                [column.name]: JSON.parse(columnValue ?? "{}"),
            });
        }
    });

    return data as unknown as PlayerDataDB;
};
