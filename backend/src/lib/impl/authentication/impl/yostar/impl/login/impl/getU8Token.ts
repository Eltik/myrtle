import { DEVICE_IDS } from "../../../../..";
import type { AKServer } from "../../../../../../../../types/impl/lib/impl/authentication";
import request from "../../../../request";
import { generateU8Sign } from "./generateU8Sign";

export const getU8Token = async (
    yostarUID: string,
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

    const extension: { type: number; uid: string; token: string } | { uid: string; access_token: string } | null = null;
    if (channelID === "3") {
        Object.assign(extension ?? {}, {
            type: 1,
            uid: yostarUID,
            token: accessToken,
        });
    } else {
        Object.assign(extension ?? {}, {
            uid: yostarUID,
            access_token: accessToken,
        });
    }

    const body = {
        appId: "1",
        platform: 1,
        channelId: channelID,
        subChannel: channelID,
        extension: JSON.stringify(extension, null, 0).replace(/\s/g, ""),

        // Optional fields
        worldId: channelID,
        deviceId: DEVICE_IDS[0],
        deviceId2: DEVICE_IDS[1],
        deviceId3: DEVICE_IDS[2],
    };

    Object.assign(body, {
        sign: generateU8Sign(body),
    });

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
