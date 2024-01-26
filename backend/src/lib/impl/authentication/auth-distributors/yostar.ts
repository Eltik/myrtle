import * as readline from "readline";

import { request } from "../../../../helper/request";
import { PASSPORT_DOMAINS, type AKServer, type AKDomain, deviceIds, getU8Token, getSecret } from "../auth";
import type { AuthSession } from "../auth-session";

import colors from "colors";

const requestPassport = async (endpoint: string, server: AKServer, args?: RequestInit): Promise<Response> => {
    return await request(PASSPORT_DOMAINS[server] as AKDomain, endpoint, args);
};

export const getAccessToken = async (channelUID: string, yostarToken: string, server: AKServer): Promise<string> => {
    const body = {
        platform: "android",
        uid: channelUID,
        token: yostarToken,
        deviceId: deviceIds[0],
    };
    const data = (await (
        await requestPassport("user/login", server, {
            body: JSON.stringify(body),
        })
    ).json()) as { accessToken: string };
    return data.accessToken;
};

export const requestYostarAuth = async (email: string, server: AKServer) => {
    console.log(colors.gray(`Sending code to ${email}...`));
    const body = {
        platform: "android",
        account: email,
        authlang: "en",
    };
    const data = (await (
        await requestPassport("account/yostar_auth_request", server, {
            body: JSON.stringify(body),
        })
    ).json()) as { result: number };

    if (data.result !== 0) {
        console.log(colors.red(`Error sending code to ${email}:`), data);
    } else {
        console.log(colors.gray(`Code sent to ${email} successfully.`));
    }

    return data;
};

export const submitYostarAuth = async (email: string, code: string, server: AKServer): Promise<[string, string]> => {
    const body = {
        account: email,
        code: code,
    };

    const data = (await (
        await requestPassport("account/yostar_auth_submit", server, {
            body: JSON.stringify(body),
        })
    ).json()) as { yostar_uid: string; yostar_token: string };
    return [data.yostar_uid, data.yostar_token];
};

export const getYostarToken = async (email: string, yostarUID: string, yostarToken: string, server: AKServer): Promise<[string, string]> => {
    const body = {
        yostar_username: email,
        yostar_uid: yostarUID,
        yostar_token: yostarToken,
        deviceId: deviceIds[0],
        createNew: "0",
    };
    const data = (await (
        await requestPassport("user/yostar_createlogin", server, {
            body: JSON.stringify(body),
        })
    ).json()) as { uid: string; token: string };
    return [data.uid, data.token];
};

export const createGuestAccount = async (server: AKServer): Promise<[string, string]> => {
    const body = {
        deviceId: deviceIds[0],
    };
    const data = (await (
        await requestPassport("user/create", server, {
            body: JSON.stringify(body),
        })
    ).json()) as { uid: string; token: string };
    console.log(`Created guest account ${data.uid}`);
    return [data.uid, data.token];
};

export const bindNickname = async (server: AKServer, nickname: string) => {
    /*
    await this.authRequest("user/bindNickname", {
        body: JSON.stringify({
            nickName: nickname
        })
    });
    */
};

export const loginWithToken = async (channelUID: string, yostarToken: string, server: AKServer, session?: AuthSession) => {
    const accessToken = await getAccessToken(channelUID, yostarToken, server);
    const data = await getU8Token(channelUID, accessToken, server, session);

    Object.assign(session ?? {}, {
        uid: data[0],
    });

    const u8Token = data[1];
    await getSecret(data[0], u8Token, server, session);
};

export const getTokenFromEmailCode = async (server: AKServer, email?: string, code?: string, stdin: boolean = false): Promise<[string, string]> => {
    if (!email) {
        if (!stdin) {
            throw new Error("Email not provided but stdin is disabled.");
        }

        email = await getUserInput("Enter email:");
    }

    if (!code) {
        await requestYostarAuth(email!, server);
        if (!stdin) {
            return ["", ""];
        }

        code = await getUserInput("Enter code: ");
    }

    const [yostarUid, yostarToken] = await submitYostarAuth(email, code, server);

    console.log(colors.gray(`Successfully fetched token for ${email}.`));
    return getYostarToken(email, yostarUid, yostarToken, server);
};

export const getUserInput = async (prompt: string): Promise<string> => {
    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout,
    });

    return new Promise<string>((resolve) => {
        rl.question(prompt, (answer) => {
            rl.close();
            resolve(answer);
        });
    });
};

export const loginWithEmailCode = async (server: AKServer, session?: AuthSession, email?: string, code?: string, stdin: boolean = false) => {
    const data = await getTokenFromEmailCode(server, email, code, stdin);

    const channelUID = data[0];
    const token = data[1];
    await loginWithToken(channelUID, token, server, session);

    if (stdin) {
        console.log(`Channel UID: ${channelUID} Token: ${token}`);
        console.log(`Usage: loginWithToken("${channelUID}", "${token}")`);
    }

    return [channelUID, token];
};

export const loginAsGuest = async (server: AKServer, session?: AuthSession, nickname?: string): Promise<[string, string]> => {
    const data = await createGuestAccount(server);

    const channelUID = data[0];
    const token = data[1];

    await loginWithToken(channelUID, token, server, session);
    await bindNickname(server, nickname ?? "Guest");
    return [channelUID, token];
};
