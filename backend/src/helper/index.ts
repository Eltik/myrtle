import { v4 as uuidv4 } from "uuid";
import random from "random";

export const createRandomDeviceIds = (): [string, string, string] => {
    const deviceid2 = "86" + Array.from({ length: 13 }, () => random.int(0, 9)).join("");
    return [uuidv4(), deviceid2, uuidv4()];
};
