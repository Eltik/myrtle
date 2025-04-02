import { loadVersionConfig } from "../../authentication/impl/load/impl/versionConfig";
import { loadNetworkConfig } from "../../authentication/impl/load/impl/networkConfig";
import colors from "colors";

export const reloadConfigs = async () => {
    try {
        console.log(colors.yellow("Reloading network and version configs..."));
        await loadNetworkConfig("all");
        await loadVersionConfig("all");
        console.log(colors.green("Successfully reloaded network and version configs"));
    } catch (error) {
        console.error(colors.red("Error reloading configs:"), error);
    }
};
