import { NETWORK_ROUTES, VERSIONS } from "../../..";
import emitter, { Events } from "../../../../../../events";
import type { AKServer } from "../../../../../../types/impl/lib/impl/authentication";
import request from "../../request";
import colors from "colors";

// Timeout for version config loading (5 seconds)
const CONFIG_TIMEOUT_MS = 5000;

export const loadVersionConfig = async (server: AKServer | "all") => {
    if (server === "all") {
        for (const server of ["en", "jp", "kr", "cn", "bili", "tw"]) {
            await loadVersionConfig(server as AKServer);
        }
        return;
    }

    try {
        // Create a promise that will reject after the timeout
        const timeoutPromise = new Promise<never>((_, reject) => {
            setTimeout(() => {
                reject(new Error(`Version config loading timed out for server ${server} after ${CONFIG_TIMEOUT_MS}ms`));
            }, CONFIG_TIMEOUT_MS);
        });

        // Create the actual fetch promise
        const fetchPromise = (async () => {
            return await (await request("hv", null, undefined, server)).json();
        })();

        // Race the fetch against the timeout
        const data = await Promise.race([fetchPromise, timeoutPromise]);
        Object.assign(VERSIONS[server], data);

        await emitter.emit(Events.CONFIG_VERSION_LOADED, server);
    } catch (error) {
        console.error(colors.yellow(`Error loading version config for server ${server}:`), error);
    }
};

export const resetVersionConfig = () => {
    for (const server in NETWORK_ROUTES) {
        Object.assign(VERSIONS, {
            [server as AKServer]: {
                resVersion: "",
                clientVersion: "",
            },
        });
    }
};
