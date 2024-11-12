import { init as initAuthentication } from "./impl/authentication";
import { init as initLocal } from "./impl/local";

export const init = async () => {
    await initLocal();
    await initAuthentication();
};
