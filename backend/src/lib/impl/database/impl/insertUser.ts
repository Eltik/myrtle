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
        ${Object.keys(data).join(", ")}
    )
    VALUES (
        $uid,
        $server,
        ${Object.keys(data)
            .map((key) => {
                return `$${key}`;
            })
            .join(", ")}
    )
    `;

    const params = {
        $uid: data.status.uid,
        $server: server,
    };

    // Convert all values to strings and assign to params
    Object.entries(data).forEach(([key, value]) => {
        Object.assign(params, {
            [`$${key}`]: typeof value === "string" ? `'${value}'` : typeof value === "object" ? JSON.stringify(value) : value,
        });
    });

    db.prepare(query).run(params);
    console.log(colors.green(`Inserted ${data.status.uid} into ${tableName}`));

    return;
};
