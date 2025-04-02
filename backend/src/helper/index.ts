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
