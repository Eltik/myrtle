import { db, tableName } from "..";
import type { Db, PlayerData, PlayerDataDB } from "../../../../types/types";

export const getUser = async (uid: string, server: string): Promise<PlayerData | null> => {
    const data = db
        .query<Db<PlayerDataDB>, { $uid: string; $server: string }>(
            `SELECT * FROM ${tableName}
            WHERE uid = $uid
            AND server = $server
    `,
        )
        .get({ $uid: uid, $server: server });

    if (!data) return null;

    const result: PlayerData = {
        uid: data.uid,
        server: data.server,
        ...JSON.parse(data.data),
    };

    return result;
};
