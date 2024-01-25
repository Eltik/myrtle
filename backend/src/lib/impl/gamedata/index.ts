import { join } from "node:path";

/**
 * @description Fetches static data from https://github.com/yuanyan3060/ArknightsGameResource/tree/main/gamedata/excel
 */

const fetchGameData = async (name: string) => {
    const file = Bun.file(join(import.meta.dir, `./gamedata/excel/${name}.json`));
    if (await file.exists()) {
        return await file.json();
    } else {
        // https://github.com/Kengxxiao/ArknightsGameData_YoStar/tree/main/en_US/gamedata
        // https://raw.githubusercontent.com/Kengxxiao/ArknightsGameData_YoStar/main/en_US/gamedata/excel/character_table.json
        const data = await (await fetch(`https://raw.githubusercontent.com/Kengxxiao/ArknightsGameData_YoStar/main/en_US/gamedata/excel/${name}.json`)).json();
        await Bun.write(join(import.meta.dir, `./gamedata/excel/${name}.json`), JSON.stringify(data));
        return data;
    }
};

export const getOperator = async (id: string) => {
    return (await fetchGameData("character_table"))[id];
};

export const getItem = async (id: string) => {
    return (await fetchGameData("item_table")).items[id];
};

export const getMedal = async (id: string) => {
    const data = await fetchGameData("medal_table");
    return data.medalList.find((m: { medalId: string }) => m.medalId === id);
};

export const getSkill = async (id: string) => {
    return (await fetchGameData("skill_table"))[id];
};

export const getModule = async (id: string) => {
    return (await fetchGameData("uniequip_table")).equipDict[id];
};

export const calculateTrust = async (trust: number) => {
    const frames = (await fetchGameData("favor_table")).favorFrames;
    const keyFrames = frames.map((frame: { data: { favorPoint: number } }) => frame.data.favorPoint);
    return keyFrames.findIndex((frame: number) => frame >= trust);
};
