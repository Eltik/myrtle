import { Stage, StageData } from "../../../../types/types";
import { getStage } from "../../gamedata";
import { join } from "node:path";

export const stages: Stage[] = [];

export const preprocess = async(materials: { stageId: string }[]) => {
    const file = Bun.file(join(import.meta.dir, "./stages.json"));
    if (await file.exists()) {
        const data = await file.json();
        stages.push(...data);
        return;
    }

    const tempStages: StageData[] = [];

    const stageIds: string[] = [];
    for (const material of materials) {
        if (!stageIds.includes(material.stageId)) {
            stageIds.push(material.stageId);
        }
    }

    const promises = stageIds.map(async (stage) => {
        if (!stages.find((item) => item.stageId === stage)) {
            const stageData = await getStage(stage);
            return stageData;
        }
    });

    const newStages = await Promise.all(promises);
    const filteredStages = newStages.filter((stageData) => stageData) as StageData[];

    tempStages.push(...filteredStages);

    console.log("Loaded temporary stages. Fetching data from Penguin Stats...")

    // https://penguin-stats.io/PenguinStats/api/v2/result/matrix?show_closed_zone=true
    const data = await (await fetch("https://penguin-stats.io/PenguinStats/api/v2/result/matrix?show_closed_zone=true")).json() as { matrix: {
        stageId: string;
        itemId: string;
        times: number;
        quantity: number;
        stdDev: number;
        start: number;
        end: number | null;
    }[]};
    // Times is amount of times submitted, and quantity is the amount of times that item has dropped within the amount of times.

    for (const stage of data.matrix) {
        const tempStage = tempStages.find((item) => item.stageId === stage.stageId);
        if (!tempStage) continue;

        if (stages.find((item) => item.stageId === stage.stageId)) {
            stages.find((item) => item.stageId === stage.stageId)?.drops.push({
                itemId: stage.itemId,
                quantity: stage.quantity / stage.times,
                probability: stage.quantity / stage.times * 100,
                time: stage.start && stage.end ? stage.end - stage.start : null
            });
        } else {
            stages.push({
                ...tempStage,
                drops: [{
                    itemId: stage.itemId,
                    quantity: stage.quantity / stage.times,
                    probability: stage.quantity / stage.times * 100,
                    time: stage.start && stage.end ? stage.end - stage.start : null
                }],
            });
        }
    }

    console.log("Loaded all stages. Writing to file...");

    await Bun.write(join(import.meta.dir, "./stages.json"), JSON.stringify(stages));
};

/*
def get_json(s):
    req = urllib.request.Request(penguin_url + s, None, headers)
    with urllib.request.urlopen(req, timeout=5) as response:
        return json.loads(response.read().decode())
*/