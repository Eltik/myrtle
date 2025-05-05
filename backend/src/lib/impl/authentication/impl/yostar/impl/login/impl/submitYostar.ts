import { YOSTAR_DOMAINS } from "../../../../..";
import emitter, { Events } from "../../../../../../../../events";
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
        await request(YOSTAR_DOMAINS[server] as AKDomain, true, "yostar/get-auth", {
            method: "POST",
            body: body,
        })
    ).json()) as {
        Code: number;
        Data: {
            UID: string; // Email
            Token: string;
            Account: string; // Email
        };
        Msg: string;
    };

    if (data.Code !== 200) {
        await emitter.emit(Events.AUTH_YOSTAR_SUBMIT_CODE_ERROR, {
            email,
            code,
            server,
            data,
        });
    }

    return {
        token: data.Data.Token,
    };
};
