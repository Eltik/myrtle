import { DEVICE_IDS } from "../../..";
import { v4 as uuidv4 } from "uuid";

export const loadDeviceIds = () => {
    DEVICE_IDS.splice(0, DEVICE_IDS.length);
    DEVICE_IDS.push(...createRandomDeviceIds());
};

const generateRandomDigits = (length: number): string => {
    return Array.from({ length }, () => Math.floor(Math.random() * 10)).join("");
};

const createRandomDeviceIds = (): [string, string, string] => {
    const deviceid2 = "86" + generateRandomDigits(13);
    // TODO: these are not entirely correct but it doesn't seem to matter
    return [uuidv4().replace(/-/g, ""), deviceid2, uuidv4().replace(/-/g, "")];
};
