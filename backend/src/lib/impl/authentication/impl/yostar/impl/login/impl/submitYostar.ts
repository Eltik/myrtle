import { YOSTAR_DOMAINS } from "../../../../..";
import type { AKDomain, AKServer } from "../../../../../../../../types/impl/lib/impl/authentication";
import request from "../../../../request";

export const submitYostarAuth = async (
    email: string,
    code: string,
    server: AKServer,
): Promise<{
    token: string;
}> => {
    const body = {
        Account: email,
        Code: code,
    };

    const data = (await (
        await request(YOSTAR_DOMAINS[server] as AKDomain, "yostar/get-auth", {
            body: JSON.stringify(body),
        })
    ).json()) as { Data: { Token: string } };

    return {
        token: data.Data.Token,
    };
};
