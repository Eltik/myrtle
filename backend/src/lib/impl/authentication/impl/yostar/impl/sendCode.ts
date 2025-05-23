import { YOSTAR_DOMAINS } from "../../..";
import emitter, { Events } from "../../../../../../events";
import type { AKDomain, AKServer } from "../../../../../../types/impl/lib/impl/authentication";
import request from "../../request";

export default async (email: string, server: AKServer): Promise<void> => {
    const body = {
        Account: email,
        Randstr: "",
        Ticket: "",
    };

    const data = (await (
        await request(YOSTAR_DOMAINS[server] as AKDomain, true, "yostar/send-code", {
            body: body,
        })
    ).json()) as {
        Code: number;
        Data: any;
        Msg: string;
    };

    if (data.Code !== 200) {
        await emitter.emit(Events.AUTH_YOSTAR_CODE_ERROR, {
            email,
            data,
        });
    } else {
        await emitter.emit(Events.AUTH_YOSTAR_CODE_SUCCESS, {
            email,
            data,
        });
    }

    return;
};
