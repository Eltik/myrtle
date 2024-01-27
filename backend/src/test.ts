import { clearDomains, clearVersions, loadNetworkConfig, loadVersionConfig } from "./lib/impl/authentication/auth";
import { init } from "./lib/impl/database";
import { leaderboard } from "./lib/impl/database/impl/leaderboard";
import { start } from "./server";

(async () => {
    await init();
    console.log("Initialized database.");
    clearDomains();
    clearVersions();

    console.log(await leaderboard("en", "trust"))
})();
