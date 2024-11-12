import emitter, { Events } from "../../../../../events";
import { loadDeviceIds } from "./impl/deviceIds";
import { loadNetworkConfig, resetNetworkconfig } from "./impl/networkConfig";
import { loadVersionConfig, resetVersionConfig } from "./impl/versionConfig";

export default async () => {
    resetNetworkconfig();
    resetVersionConfig();

    await loadDeviceIds();

    await loadNetworkConfig("all");
    await emitter.emit(Events.CONFIG_NETWORK_INITIATED);

    await loadVersionConfig("all");
    await emitter.emit(Events.CONFIG_VERSION_INITIATED);
};
