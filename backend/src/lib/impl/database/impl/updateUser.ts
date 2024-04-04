import { db, tableName } from "..";
import { type PlayerData } from "../../../../types/types";
import colors from "colors";

export const updateUser = async (data: PlayerData, server: string) => {
    const query = `
    UPDATE ${tableName} SET data = $data
    WHERE uid = $uid AND server = $server
    `;

    const params = {
        $uid: data.status.uid,
        $server: server,
        $data: JSON.stringify(data),
    };

    db.prepare(query).run(params);

    console.log(colors.green(`Updated ${data.status.uid} in ${tableName}`));
};
