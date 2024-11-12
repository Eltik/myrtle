import { listener } from "./events/impl/listener";
import { init } from "./lib";

(async () => {
    await listener();

    await init();
})();
