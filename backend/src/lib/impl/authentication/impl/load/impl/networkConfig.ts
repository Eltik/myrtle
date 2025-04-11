import { DOMAINS, NETWORK_ROUTES } from "../../..";
import emitter, { Events } from "../../../../../../events";
import type { AKDomain, AKServer } from "../../../../../../types/impl/lib/impl/authentication";
import request from "../../request";
import colors from "colors";

// Timeout for network config loading (5 seconds)
const CONFIG_TIMEOUT_MS = 5000;

export const loadNetworkConfig = async (server: AKServer | "all"): Promise<void> => {
    if (server === "all") {
        for (const server of ["en", "jp", "kr", "cn", "bili", "tw"]) {
            await loadNetworkConfig(server as AKServer);
        }
        return;
    }

    try {
        // Create a promise that will reject after the timeout
        const timeoutPromise = new Promise<never>((_, reject) => {
            setTimeout(() => {
                reject(new Error(`Network config loading timed out for server ${server} after ${CONFIG_TIMEOUT_MS}ms`));
            }, CONFIG_TIMEOUT_MS);
        });

        // Create the actual fetch promise
        const fetchPromise = (async () => {
            const data = JSON.parse(
                (
                    (await (await request(NETWORK_ROUTES[server] as AKDomain)).json()) as {
                        content: string;
                        configs: Record<string, Record<string, Record<string, string>>>;
                        funcVer: string;
                    }
                ).content,
            );
            return data;
        })();

        // Race the fetch against the timeout
        const data = await Promise.race([fetchPromise, timeoutPromise]);
        Object.assign(DOMAINS[server], data.configs[data["funcVer"]]["network"]);

        await emitter.emit(Events.CONFIG_NETWORK_LOADED, server);
    } catch (error) {
        console.error(colors.yellow(`Error loading network config for server ${server}:`), error);
    }
};

export const resetNetworkconfig = () => {
    for (const server in NETWORK_ROUTES) {
        Object.assign(DOMAINS, {
            [server as AKServer]: {},
        });
    }
};
