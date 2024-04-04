import colors from "colors";
import { db, tableName } from "..";
import type { PlayerData } from "../../../../types/types";
import { getUser } from "./getUser";

export const insertUser = async (data: PlayerData, server: string): Promise<void> => {
    if (await getUser(data.status.uid, server)) {
        return;
    }

    const query = `
    INSERT INTO ${tableName} (
        uid,
        server,
        data
    )
    VALUES (
        $uid,
        $server,
        $data
    )
    `;

    const params = {
        $uid: data.status.uid,
        $server: server,
        $data: {},
    };

    Object.assign(params.$data, {
        ...data,
    });

    Object.assign(params, {
        $data: JSON.stringify(params.$data),
    });

    db.prepare(query).run(params as any);
    console.log(colors.green(`Inserted ${data.status.uid} into ${tableName}`));

    return;
};
