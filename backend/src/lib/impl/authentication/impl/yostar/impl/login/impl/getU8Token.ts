import { DEVICE_IDS } from "../../../../..";
import type { AKServer } from "../../../../../../../../types/impl/lib/impl/authentication";
import request from "../../../../request";

export const getU8Token = async (
    channelUID: string,
    accessToken: string,
    server: AKServer,
): Promise<{
    uid: string;
    token: string;
}> => {
    const channelID = {
        cn: "1",
        bili: "2",
        en: "3",
        jp: "3",
        kr: "3",
        tw: "ERROR",
    }[server];

    let extension: { uid: string; token: string } | { uid: string; access_token: string } | null = null;
    if (channelID === "3") {
        extension = {
            uid: channelUID,
            token: accessToken,
        };
    } else {
        extension = {
            uid: channelUID,
            access_token: accessToken,
        };
    }

    const body = {
        appId: "1",
        platform: 1,
        channelId: channelID,
        subChannel: channelID,
        extension: JSON.stringify(extension),

        // Optional fields
        worldId: channelID,
        deviceId: DEVICE_IDS[0],
        deviceId2: DEVICE_IDS[1],
        deviceId3: DEVICE_IDS[2],
    };

    const data = (await (
        await request("u8", "user/v1/getToken", {
            body: JSON.stringify(body),
        })
    ).json()) as { uid: string; token: string };

    const uid = data.uid;
    const token = data.token;

    return {
        uid,
        token,
    };
};
