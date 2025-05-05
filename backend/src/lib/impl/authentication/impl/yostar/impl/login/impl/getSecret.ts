import { DEVICE_IDS, VERSIONS } from "../../../../..";
import emitter from "../../../../../../../../events";
import { Events } from "../../../../../../../../events";
import type { AKServer } from "../../../../../../../../types/impl/lib/impl/authentication";
import { loadVersionConfig } from "../../../../load/impl/versionConfig";
import request from "../../../../request";

export const getSecret = async (uid: string, u8Token: string, server: AKServer): Promise<string> => {
    if (!VERSIONS[server]["resVersion"]) {
        await loadVersionConfig(server);
    }

    const networkVersion = {
        cn: "5",
        bili: "5",
        en: "1",
        jp: "1",
        kr: "1",
        tw: "ERROR",
    }[server];

    const body = {
        platform: 1,
        networkVersion: networkVersion,
        assetsVersion: VERSIONS[server]["resVersion"],
        clientVersion: VERSIONS[server]["clientVersion"],
        token: u8Token,
        uid: uid,
        deviceId: DEVICE_IDS[0],
        deviceId2: DEVICE_IDS[1],
        deviceId3: DEVICE_IDS[2],
    };

    const headers = {
        secret: "",
        seqnum: "1",
        uid: uid,
    };

    const data = (await (
        await request("gs", true, "account/login", {
            method: "POST",
            body: body,
            headers,
        })
    ).json()) as {
        result: number;
        uid: string;
        secret: string;
        serviceLicenseVersion: number;
        majorVersion: string;
    };

    if (data.result !== 0) {
        await emitter.emit(Events.AUTH_YOSTAR_GET_SECRET_ERROR, {
            uid,
            u8Token,
            server,
            data,
        });
    }

    const secret = data.secret;
    return secret;
};
