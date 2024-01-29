import { db, tableName } from "..";
import { type PlayerData } from "../../../../types/types";
import colors from "colors";

export const updateUser = async (data: PlayerData, server: string) => {
    const query = `
    UPDATE ${tableName} SET
        ${Object.keys(data).map((value) => {
            return `${value} = $${value}`;
        })}
    WHERE uid = $uid AND server = $server
    `;

    const params = {
        $uid: data.status.uid,
        $server: server,
    };

    Object.entries(data).forEach(([key, value]) => {
        Object.assign(params, {
            [`$${key}`]: typeof value === "string" ? `'${value}'` : typeof value === "object" ? JSON.stringify(value) : value,
        });
    });

    db.prepare(query).run(params);

    console.log(colors.green(`Updated ${data.status.uid} in ${tableName}`));
};
