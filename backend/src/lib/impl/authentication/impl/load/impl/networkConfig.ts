import { DOMAINS, NETWORK_ROUTES } from "../../..";
import emitter, { Events } from "../../../../../../events";
import type { AKDomain, AKServer } from "../../../../../../types/impl/lib/impl/authentication";
import request from "../../request";

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
    Object.assign(DOMAINS[server], data.configs[data["funcVer"]]["network"]);

    await emitter.emit(Events.CONFIG_NETWORK_LOADED, server);
};

export const resetNetworkconfig = () => {
    for (const server in NETWORK_ROUTES) {
        Object.assign(DOMAINS, {
            [server as AKServer]: {},
        });
    }
};
