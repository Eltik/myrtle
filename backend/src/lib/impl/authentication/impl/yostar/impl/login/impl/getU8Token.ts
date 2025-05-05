import { DEVICE_IDS } from "../../../../..";
import emitter from "../../../../../../../../events";
import { Events } from "../../../../../../../../events";
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

    // Initialize extension as an object
    let extension: { type: number; uid: string; token: string } | { uid: string; access_token: string };
    if (channelID === "3") {
        // EN, JP, KR
        extension = {
            type: 1,
            uid: yostarUID,
            token: accessToken,
        };
    } else {
        // CN, Bili (Assuming this structure is correct for them)
        extension = {
            uid: yostarUID,
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

    Object.assign(body, {
        sign: generateU8Sign(body),
    });

    const data = (await (
        await request("u8", true, "user/v1/getToken", {
            method: "POST",
            body: body,
        })
    ).json()) as {
        result: number;
        captcha: any;
        error: string;
        uid: string;
        channelUid: string;
        token: string;
        isGuest: number;
        extension: string;
    };

    if (data.result !== 0) {
        await emitter.emit(Events.AUTH_YOSTAR_GET_U8_TOKEN_ERROR, {
            uid: yostarUID,
            accessToken,
            server,
            data,
        });
    }

    const uid = data.uid;
    const token = data.token;

    return {
        uid,
        token,
    };
};
