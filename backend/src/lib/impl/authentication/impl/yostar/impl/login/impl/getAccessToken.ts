import { DEVICE_IDS, PASSPORT_DOMAINS } from "../../../../..";
import type { AKDomain, AKServer } from "../../../../../../../../types/impl/lib/impl/authentication";
import request from "../../../../request";

export const getAccessToken = async (channelUID: string, yostarToken: string, server: AKServer): Promise<string> => {
    const body = {
        platform: "android",
        uid: channelUID,
        token: yostarToken,
        deviceId: DEVICE_IDS[0],
    };

    const data = (await (
        await request(PASSPORT_DOMAINS[server] as AKDomain, "user/login", {
            body: JSON.stringify(body),
        })
    ).json()) as { accessToken: string };

    return data.accessToken;
};
