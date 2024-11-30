import { DEFAULT_HEADERS, DOMAINS } from "../../..";
import type { AKDomain, AKServer, AuthSession } from "../../../../../../types/impl/lib/impl/authentication";
import { loadNetworkConfig } from "../../load/impl/networkConfig";

export const request = async (domain: AKDomain, endpoint: string | null = null, args?: RequestInit, server?: AKServer, session?: AuthSession): Promise<Response> => {
    server = server ? server : "en";

    let url: string = "";
    if (domain.includes("http")) {
        url = domain;
    } else {
        if (server === null || server === undefined) {
            throw new Error("No default server set.");
        }
        if (!DOMAINS.hasOwnProperty(server)) {
            throw new Error(`Invalid server for server ${server}.`);
        }
        if (!DOMAINS[server] || domain === "hv") {
            await loadNetworkConfig(server);
        }
        if (!DOMAINS[server].hasOwnProperty(domain)) {
            throw new Error(`Invalid domain for domain ${domain}.`);
        }

        url = DOMAINS[server][domain];
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

    const data = await fetch(url, args);
    return data;
};
