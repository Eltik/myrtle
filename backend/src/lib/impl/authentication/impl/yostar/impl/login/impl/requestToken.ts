import { DEVICE_IDS, PASSPORT_DOMAINS } from "../../../../..";
import type { AKDomain, AKServer } from "../../../../../../../../types/impl/lib/impl/authentication";
import request from "../../../../request";

export const requestToken = async (
    email: string,
    yostarUID: string,
    yostarToken: string,
    server: AKServer,
): Promise<{
    uid: string;
    token: string;
}> => {
    const body = {
        yostar_username: email,
        yostar_uid: yostarUID,
        yostar_token: yostarToken,
        deviceId: DEVICE_IDS[0],
        createNew: "0",
    };

    const data = (await (
        await request(PASSPORT_DOMAINS[server] as AKDomain, "user/yostar_createlogin", {
            body: JSON.stringify(body),
        })
    ).json()) as { uid: string; token: string };

    return {
        uid: data.uid,
        token: data.token,
    };
};
