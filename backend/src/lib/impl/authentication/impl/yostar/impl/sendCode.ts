import { PASSPORT_DOMAINS } from "../../..";
import emitter, { Events } from "../../../../../../events";
import type { AKDomain, AKServer } from "../../../../../../types/impl/lib/impl/authentication";
import request from "../../request";

export default async (email: string, server: AKServer): Promise<void> => {
    const body = {
        platform: "android",
        account: email,
        authlang: "en",
    };

    const data = (await (
        await request(PASSPORT_DOMAINS[server] as AKDomain, "account/yostar_auth_request", {
            body: JSON.stringify(body),
        })
    ).json()) as { result: number };

    if (data.result !== 0) {
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
