import { db, tableName } from "..";
import type { Db, PlayerDataDB } from "../../../../types/types";

export const search = async ({ nickname, nicknumber, server }: { nickname: string; nicknumber?: string; server?: string }): Promise<PlayerDataDB[]> => {
    const params = {
        $nickname: `%${nickname}%`, // Add wildcards for LIKE operator
    };

    if (nicknumber) Object.assign(params, { $nicknumber: nicknumber });
    if (server) Object.assign(params, { $server: server });

    const results = db
        .query<Db<PlayerDataDB>, { $nickname: string; $nicknumber?: string; server?: string }>(
            `SELECT * FROM ${tableName} WHERE (
                ${server ? `server = $server AND data->'status'->>'nickName' LIKE $nickname` : "data->'status'->>'nickName' LIKE $nickname"}
                ${nicknumber ? "AND data->'status'->>'nickNumber' = $nicknumber" : ""})`,
        )
        .all(params);

    for (const result of results) {
        Object.assign(result, { data: JSON.parse(result.data) });

        Object.entries(result.data).forEach(([key, value]) => {
            Object.assign(result, { [key]: value });
        });

        delete (result as any).data;
    }

    return results as unknown as PlayerDataDB[];
};
