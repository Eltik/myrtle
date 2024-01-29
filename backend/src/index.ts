import { clearDomains, clearVersions, loadNetworkConfig, loadVersionConfig } from "./lib/impl/authentication/auth";
import { init } from "./lib/impl/database";
import { start } from "./server";

(async () => {
    await init();
    console.log("Initialized database.");
    clearDomains();
    clearVersions();

    await loadNetworkConfig("all");
    await loadVersionConfig("all");
    await start();
})()
    .then(() => console.log("Started server."))
    .catch((err) => console.error(err));
