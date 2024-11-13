import dotenv from "dotenv";
dotenv.config();

import { listener } from "./events/impl/listener";
import { init } from "./lib";
import app from "./app";

(async () => {
    await listener();

    await init();
    await app.start();
})();
