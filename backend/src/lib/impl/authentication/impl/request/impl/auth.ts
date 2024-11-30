import type { AKServer, AuthSession } from "../../../../../../types/impl/lib/impl/authentication";
import { request } from "./request";

export const authRequest = async (endpoint: string, session: AuthSession, args?: RequestInit, server?: AKServer): Promise<Response> => {
    server = server ? server : "en";
    if (!session.uid) {
        throw new Error("Not logged in.");
    }

    session.seqnum++;

    return await request("gs", endpoint, args, server, session);
};
