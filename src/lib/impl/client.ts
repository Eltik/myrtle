import { authRequest } from "../../helper/request";
import type { AuthSession } from "./auth-session";

export const getRawData = async (session: AuthSession) => {
    const data = await (
        await authRequest("account/syncData", session, {
            body: JSON.stringify({
                platform: 1,
            }),
        })
    ).json();
    return data;
};

export const getSocialSortList = async (session: AuthSession, type: number, sortKey = ["level"], param = {}, { server = null } = {}) => {
    const data = await (
        await authRequest("social/getSocialSortList", session, {
            body: JSON.stringify({
                type,
                sortKey,
                param,
                server,
            }),
        })
    ).json();
    return data;
};
