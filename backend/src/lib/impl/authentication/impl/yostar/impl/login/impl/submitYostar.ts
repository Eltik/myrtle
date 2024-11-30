import { PASSPORT_DOMAINS } from "../../../../..";
import type { AKDomain, AKServer } from "../../../../../../../../types/impl/lib/impl/authentication";
import request from "../../../../request";

export const submitYostarAuth = async (
    email: string,
    code: string,
    server: AKServer,
): Promise<{
    yostar_uid: string;
    yostar_token: string;
}> => {
    const body = {
        account: email,
        code: code,
    };

    const data = (await (
        await request(PASSPORT_DOMAINS[server] as AKDomain, "account/yostar_auth_submit", {
            body: JSON.stringify(body),
        })
    ).json()) as { yostar_uid: string; yostar_token: string };

    return {
        yostar_uid: data.yostar_uid,
        yostar_token: data.yostar_token,
    };
};
