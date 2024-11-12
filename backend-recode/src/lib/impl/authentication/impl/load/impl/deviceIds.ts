import { v4 as uuidv4 } from "uuid";
import random from "random";

import { DEVICE_IDS } from "../../..";

export const loadDeviceIds = () => {
    DEVICE_IDS.splice(0, DEVICE_IDS.length);
    DEVICE_IDS.push(...createRandomDeviceIds());
};

const createRandomDeviceIds = (): [string, string, string] => {
    const deviceid2 = "86" + Array.from({ length: 13 }, () => random.int(0, 9)).join("");
    return [uuidv4(), deviceid2, uuidv4()];
};
