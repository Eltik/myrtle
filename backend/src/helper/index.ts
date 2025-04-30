export function setIntervalImmediately(func: () => Promise<void>, interval: number) {
    func();
    return setInterval(async () => {
        try {
            await func();
        } catch (e) {
            console.error("Error occurred while trying to execute the interval function: ", e);
        }
    }, interval);
}

// Helper function to filter object keys

export const filterObject = (obj: any, keys: string[]): Partial<any> => {
    if (!obj) return {};
    const filtered: Partial<any> = {};
    for (const key of keys) {
        if (Object.prototype.hasOwnProperty.call(obj, key)) {
            filtered[key] = obj[key];
        }
    }
    return filtered;
};
