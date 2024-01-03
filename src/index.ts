import { clearDomains, clearVersions, loadNetworkConfig, loadVersionConfig } from "./lib/impl/auth";
import { start } from "./server";

(async () => {
    clearDomains();
    clearVersions();
    
    await loadNetworkConfig("all");
    await loadVersionConfig("all");
    await start();
})();
