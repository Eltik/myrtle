import { createRandomDeviceIds } from "../../../helper";
import { request } from "../../../helper/request";
import type { AuthSession } from "./auth-session";

import colors from "colors";

export const PASSPORT_DOMAINS = {
    en: "https://passport.arknights.global",
    jp: "https://passport.arknights.jp",
    kr: "https://passport.arknights.kr",
    cn: "ERROR",
    bili: "ERROR",
    tw: "ERROR",
};

export const NETWORK_ROUTES = {
    en: "https://ak-conf.arknights.global/config/prod/official/network_config",
    jp: "https://ak-conf.arknights.jp/config/prod/official/network_config",
    kr: "https://ak-conf.arknights.kr/config/prod/official/network_config",
    cn: "https://ak-conf.hypergryph.com/config/prod/official/network_config",
    bili: "https://ak-conf.hypergryph.com/config/prod/b/network_config",
    tw: "https://ak-conf.txwy.tw/config/prod/official/network_config",
};

export const DEFAULT_HEADERS = {
    "Content-Type": "application/json",
    "X-Unity-Version": "2017.4.39f1",
    "User-Agent": "Dalvik/2.1.0 (Linux; U; Android 11; KB2000 Build/RP1A.201005.001)",
    Connection: "Keep-Alive",
};

export type AKDistributor = "yostar" | "hypergryph" | "bilibili" | "longcheng";
export type AKServer = "en" | "jp" | "kr" | "cn" | "bili" | "tw";
export type AKDomain = "gs" | "as" | "u8" | "hu" | "hv" | "rc" | "an" | "prean" | "sl" | "of" | "pkgAd" | "pkgIOS";

export const deviceIds: [string, string, string] = createRandomDeviceIds();
export const domains: { [key in AKServer]: { [key in AKDomain]: string } } = {} as any;
export const versions: Record<AKServer, Record<"resVersion" | "clientVersion", string>> = {} as any;

export const getSecret = async (uid: string, u8Token: string, server: AKServer, session?: AuthSession): Promise<string> => {
    if (!versions[server]["resVersion"]) {
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
        assetsVersion: versions[server]["resVersion"],
        clientVersion: versions[server]["clientVersion"],
        token: u8Token,
        uid: uid,
        deviceId: deviceIds[0],
        deviceId2: deviceIds[1],
        deviceId3: deviceIds[2],
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

    Object.assign(session ?? {}, {
        secret: secret,
    });

    console.log(colors.gray(`Logged in successfully with UID ${uid}.`));
    return secret;
};

export const getU8Token = async (channelUID: string, accessToken: string, server: AKServer, session?: AuthSession): Promise<[string, string]> => {
    const channelID = {
        cn: "1",
        bili: "2",
        en: "3",
        jp: "3",
        kr: "3",
        tw: "ERROR",
    }[server];

    let extension: { uid: string; token: string } | { uid: string; access_token: string } | null = null;
    if (channelID === "3") {
        extension = {
            uid: channelUID,
            token: accessToken,
        };
    } else {
        extension = {
            uid: channelUID,
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
        deviceId: deviceIds[0],
        deviceId2: deviceIds[1],
        deviceId3: deviceIds[2],
    };

    const data = (await (
        await request("u8", "user/v1/getToken", {
            body: JSON.stringify(body),
        })
    ).json()) as { uid: string; token: string };

    const uid = data.uid;
    const token = data.token;

    Object.assign(session ?? {}, {
        uid,
    });

    return [uid, token];
};

/*
export const fromToken = async(server: AKServer, channelUID: string, token: string): Promise<Auth | null> => {
    let auth: Auth | null = null;
    if (["en", "jp", "kr"].includes(server)) {
        auth = new YostarAuth(server);
        await auth.loginWithToken(channelUID, token);
    } else if (server === "cn") {
        // Hypergryph
    } else if (server === "bili") {
        // Bilbili
    } else if (server === "tw") {
        // Longcheng
    } else {
        throw new Error(`Invalid server ${server}.`);
    }
    return auth;
}
*/

export const loadNetworkConfig = async (server: AKServer | "all"): Promise<void> => {
    if (server === "all") {
        for (const server of ["en", "jp", "kr", "cn", "bili", "tw"]) {
            await loadNetworkConfig(server as AKServer);
        }
        return;
    }

    const data = JSON.parse(
        (
            (await (await request(NETWORK_ROUTES[server] as AKDomain)).json()) as {
                content: string;
                configs: Record<string, Record<string, Record<string, string>>>;
                funcVer: string;
            }
        ).content,
    );
    Object.assign(domains[server], data.configs[data["funcVer"]]["network"]);

    console.log(colors.gray(`Loaded network config for ${server}.`));
};

export const loadVersionConfig = async (server: AKServer | "all") => {
    if (server === "all") {
        for (const server of ["en", "jp", "kr", "cn", "bili", "tw"]) {
            await loadVersionConfig(server as AKServer);
        }
        return;
    }

    const data = await (await request("hv", null, undefined, server)).json();
    Object.assign(versions[server], data);

    console.log(colors.gray(`Loaded version config for ${server}.`));
};

export const reloadDeviceIds = () => {
    // Clear deviceIds array
    deviceIds.splice(0, deviceIds.length);

    // Create new deviceIds
    deviceIds.push(...createRandomDeviceIds());
};

export const clearDomains = () => {
    for (let server in NETWORK_ROUTES) {
        Object.assign(domains, {
            [server as AKServer]: {},
        });
    }
};

export const clearVersions = () => {
    for (let server in NETWORK_ROUTES) {
        Object.assign(versions, {
            [server as AKServer]: {
                resVersion: "",
                clientVersion: "",
            },
        });
    }
};
