import { init } from "./lib/impl/database";
import { preprocess } from "./lib/impl/planner/impl/before";

import { join } from "node:path";
import { calculateBestStage, getPlan } from "./lib/impl/planner/impl/plan";

(async () => {
    await init();
    console.log("Initialized database.");
    
    const stages = Bun.file(join(import.meta.dir, "./lib/impl/gamedata/gamedata/excel/stage_table.json"));
    const data = await stages.json();

    await preprocess(Object.keys(data.stages).map((item) => {
        return {
            stageId: item
        }
    }))

    const requirements = {
        "30012": 150
    };

    const ownedItems = {};

    const result = await getPlan(requirements, ownedItems);
    console.log(result)

    //const bestStages = await calculateBestStage(requirements);
    //console.log(bestStages);
})()
    .then(() => console.log("Started server."))
    .catch((err) => console.error(err));
