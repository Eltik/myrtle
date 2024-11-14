import { DEVICE_IDS, VERSIONS } from "../../../../..";
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
        await request("gs", "account/login", {
            body: JSON.stringify(body),
            headers,
        })
    ).json()) as { secret: string };

    const secret = data.secret;
    return secret;
};
