import { domains, type AKDomain, type AKServer, DEFAULT_HEADERS, loadNetworkConfig } from "../lib/impl/authentication/auth";
import type { AuthSession } from "../lib/impl/authentication/auth-session";

export const authRequest = async (endpoint: string, session: AuthSession, args?: RequestInit, server?: AKServer): Promise<Response> => {
    server = server ? server : "en";
    if (!session.uid) {
        throw new Error("Not logged in.");
    }

    session.seqnum++;

    return await request("gs", endpoint, args, server, session);
};

export const request = async (domain: AKDomain, endpoint: string | null = null, args?: RequestInit, server?: AKServer, session?: AuthSession): Promise<Response> => {
    server = server ? server : "en";

    let url: string = "";
    if (domain.includes("http")) {
        url = domain;
    } else {
        if (server === null || server === undefined) {
            throw new Error("No default server set.");
        }
        if (!domains.hasOwnProperty(server)) {
            throw new Error(`Invalid server for server ${server}.`);
        }
        if (!domains[server] || domain === "hv") {
            await loadNetworkConfig(server);
        }
        if (!domains[server].hasOwnProperty(domain)) {
            throw new Error(`Invalid domain for domain ${domain}.`);
        }

        url = domains[server][domain];
    }

    if (url.includes("{0}")) {
        url = url.replace("{0}", "Android");
    }
    if (endpoint) {
        url = url + "/" + endpoint;
    }

    if (!args) {
        args = {};
    }
    if (!args.method) {
        args.method = args.body ? "POST" : "GET";
    }
    if (!args.headers) {
        args.headers = DEFAULT_HEADERS;
        if (session !== null && session !== undefined) {
            args.headers["uid"] = session.uid;
            args.headers["seqnum"] = String(session.seqnum);
            args.headers["secret"] = session.secret;
        }
    } else {
        Object.assign(args.headers, DEFAULT_HEADERS);
    }

    const data = await await fetch(url, args);
    return data;
};
