import { GAME_DATA_REPOSITORY } from "../local/impl/handler";
import { RESOURCE_REPOSITORY } from "../local/impl/gamedata";
import colors from "colors";
import { checkRepoUpdate } from "./impl/checkRepoUpdate";
import { cleanAndRedownloadGameData } from "./impl/cleanGameData";
import { reloadConfigs } from "./impl/reloadConfigs";
import { setIntervalImmediately } from "../../../helper";

// Constants for intervals
const REPO_CHECK_INTERVAL = 30 * 60 * 1000; // 30 minutes
const CONFIG_RELOAD_INTERVAL = 60 * 60 * 1000; // 1 hour

export const init = async () => {
    // Initial setup
    console.log(colors.blue("Initializing cron jobs..."));

    // Repository check interval
    setIntervalImmediately(async () => {
        console.log(colors.gray("Checking for repository updates..."));

        const [gameDataOwner, gameDataRepo] = GAME_DATA_REPOSITORY.split("/");
        const [resourceOwner, resourceRepo] = RESOURCE_REPOSITORY.split("/");

        const gameDataStatus = await checkRepoUpdate(gameDataOwner, gameDataRepo);
        const resourceStatus = await checkRepoUpdate(resourceOwner, resourceRepo);

        // Log status for each repository
        console.log(colors.gray(`Game Data Repository (${gameDataOwner}/${gameDataRepo}):`), colors.yellow(`Last updated: ${gameDataStatus.lastUpdate.toLocaleString()}`), colors.gray(`Commit: ${gameDataStatus.commit}`));
        console.log(colors.gray(`Resource Repository (${resourceOwner}/${resourceRepo}):`), colors.yellow(`Last updated: ${resourceStatus.lastUpdate.toLocaleString()}`), colors.gray(`Commit: ${resourceStatus.commit}`));

        if (gameDataStatus.updated || resourceStatus.updated) {
            console.log(colors.yellow("Repository updates detected, cleaning and redownloading data..."));
            await cleanAndRedownloadGameData();
        }
    }, REPO_CHECK_INTERVAL);

    // Config reload interval
    setInterval(async () => {
        console.log(colors.gray("Running scheduled config reload..."));
        await reloadConfigs();
    }, CONFIG_RELOAD_INTERVAL);

    console.log(colors.green("Cron jobs initialized successfully"));
};
