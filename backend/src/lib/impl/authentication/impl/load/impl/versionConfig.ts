import { NETWORK_ROUTES, VERSIONS } from "../../..";
import emitter, { Events } from "../../../../../../events";
import type { AKServer } from "../../../../../../types/impl/lib/impl/authentication";
import request from "../../request";

export const loadVersionConfig = async (server: AKServer | "all") => {
    if (server === "all") {
        for (const server of ["en", "jp", "kr", "cn", "bili", "tw"]) {
            await loadVersionConfig(server as AKServer);
        }
        return;
    }

    const data = await (await request("hv", null, undefined, server)).json();
    Object.assign(VERSIONS[server], data);

    await emitter.emit(Events.CONFIG_VERSION_LOADED, server);
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
