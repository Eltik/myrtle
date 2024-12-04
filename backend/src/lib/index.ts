import { env } from "../env";
import { init as initAuthentication } from "./impl/authentication";
import { init as initLocal } from "./impl/local";
import { init as initDPSCalculator } from "./impl/dps-calculator";

export const init = async () => {
    await initLocal();

    await initDPSCalculator();

    if (env.LOAD_AK_CONFIG) {
        await initAuthentication();
    }
};
