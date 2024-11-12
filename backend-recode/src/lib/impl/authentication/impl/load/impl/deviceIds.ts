import { DEVICE_IDS } from "../../..";

export const loadDeviceIds = () => {
    DEVICE_IDS.splice(0, DEVICE_IDS.length);
    DEVICE_IDS.push(...createRandomDeviceIds());
};

const createRandomDeviceIds = (): [string, string, string] => {
    const generateUUID = (): string => {
        return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (char) => {
            const random = Math.random() * 16 | 0;
            const value = char === "x" ? random : (random & 0x3) | 0x8;
            return value.toString(16);
        });
    };

    const generateRandomDigits = (length: number): string => {
        return Array.from({ length }, () => Math.floor(Math.random() * 10)).join("");
    };

    const deviceid2 = "86" + generateRandomDigits(13);
    return [generateUUID(), deviceid2, generateUUID()];
};
