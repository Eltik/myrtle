import { init } from "./lib/impl/database";
import { preprocess } from "./lib/impl/planner/impl/before";

import { join } from "node:path";
import { getPlan } from "./lib/impl/planner/impl/plan";

(async () => {
    await init();
    console.log("Initialized database.");
    
    const stages = Bun.file(join(import.meta.dir, "./lib/impl/gamedata/gamedata/excel/stage_table.json"));
    const data = await stages.json();

    await preprocess(Object.keys(data.stages).slice(0, 10).map((item) => {
        return {
            stageId: item
        }
    }))

    const requirements = {
        "ap_supply_lt_010": 10, // Replace "item1" with the actual item name and 10 with the required quantity
        "2001": 5,
    };

    const ownedItems = {};

    const result = await getPlan(requirements, ownedItems);
    console.log(Object.keys(result)[0])
    console.log(Object.keys(result)[0] + ": " + Object.values(result)[0].amount)
    console.log(Object.keys(result)[0] + ":", Object.values(result)[0].stage.drops)
})()
    .then(() => console.log("Started server."))
    .catch((err) => console.error(err));
