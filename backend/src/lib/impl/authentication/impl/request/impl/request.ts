import { DEFAULT_HEADERS, DOMAINS } from "../../..";
import type { AKDomain, AKServer, AuthSession } from "../../../../../../types/impl/lib/impl/authentication";
import { loadNetworkConfig } from "../../load/impl/networkConfig";
import { generateHeaders } from "../../yostar/impl/login/impl/generateHeaders";

// Global request timeout (5 seconds)
const REQUEST_TIMEOUT_MS = 5000;

// Modify args type to allow object for body, as we stringify it internally
// Using 'any' for simplicity as fetch handles various types and we stringify before signing
export const request = async (domain: AKDomain, assignHeaders: boolean = true, endpoint: string | null = null, args?: Omit<RequestInit, "body"> & { body?: any }, server?: AKServer, session?: AuthSession): Promise<Response> => {
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

    // Pass uid and secret (as token) to generateHeaders if session exists
    const bodyStringForSign = JSON.stringify(args.body); // Stringify once
    if (assignHeaders) {
        const yostarHeaders = generateHeaders(
            bodyStringForSign, // Use the pre-stringified version
            server,
            session?.uid,
            session?.secret, // Assuming session.secret corresponds to the Python 'token'
            // deviceId is omitted, assuming generateHeaders handles default like Python
        );
        Object.assign(args.headers, yostarHeaders);
    }

    // Add signal with timeout to the request
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);

    try {
        if (!args.signal) {
            args.signal = controller.signal;
        }

        // Use the stringified body for the fetch call, matching the signature calculation
        const fetchOptions = { ...args, body: bodyStringForSign }; // Use bodyStringForSign

        const data = await fetch(url, fetchOptions as RequestInit); // Pass the modified options
        clearTimeout(timeoutId);
        return data;
    } catch (error) {
        clearTimeout(timeoutId);
        throw error;
    }
};
