import { db, tableName } from "..";
import type { Db, PlayerData, PlayerDataDB } from "../../../../types/types";
import { schema } from "./schema";

export const search = async ({ nickname, nicknumber, server }: { nickname: string, nicknumber?: string, server?: string }): Promise<PlayerDataDB[]> => {
    const params = {
        $nickname: nickname
    };

    if (nicknumber) Object.assign(params, { $nicknumber: nicknumber });
    if (server) Object.assign(params, { $server: server });

    const results = db
        .query<Db<PlayerDataDB>, { $nickname: string; $nicknumber?: string, server?: string }>(
            `SELECT * FROM ${tableName} WHERE (
                ${server ? `server = $server AND status->>'nickName' LIKE $nickname` : "status->>'nickName' LIKE $nickname"}
                ${nicknumber ? "AND status->>'nickNumber' = $nicknumber"
                : ""})`
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
