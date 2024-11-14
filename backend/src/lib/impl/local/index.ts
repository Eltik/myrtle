import { init as initHandler } from "./impl/handler";

export const init = async () => {
    await initHandler();
};
