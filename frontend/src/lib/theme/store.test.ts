import { afterEach, describe, expect, it } from "vitest";
import { DATA_SAVER_COOKIE, hydrateTheme, themeActions, themeStore } from "./store";

const KEYS = { dynamicArtwork: "myrtle-dynamic-art", latinNames: "myrtle-latin-names", spoilers: "myrtle-spoilers", maintenanceCountdown: "myrtle-maintenance-countdown", dataSaver: "myrtle-data-saver" } as const;

function hydrated() {
    hydrateTheme();
    const { dynamicArtwork, latinNames, spoilers, maintenanceCountdown, dataSaver } = themeStore.state;
    return { dynamicArtwork, latinNames, spoilers, maintenanceCountdown, dataSaver };
}

afterEach(() => window.localStorage.clear());

describe("theme store toggles", () => {
    it("reads the defaults when nothing is stored: spoilers and data saver off, the rest on", () => {
        expect(hydrated()).toEqual({ dynamicArtwork: true, latinNames: true, spoilers: false, maintenanceCountdown: true, dataSaver: false });
    });

    it("reads only the exact string `true` as on", () => {
        for (const key of Object.values(KEYS)) window.localStorage.setItem(key, "TRUE");
        expect(hydrated()).toEqual({ dynamicArtwork: false, latinNames: false, spoilers: false, maintenanceCountdown: false, dataSaver: false });
        for (const key of Object.values(KEYS)) window.localStorage.setItem(key, "true");
        expect(hydrated()).toEqual({ dynamicArtwork: true, latinNames: true, spoilers: true, maintenanceCountdown: true, dataSaver: true });
    });

    it("stores each toggle under its own key and updates the store", () => {
        themeActions.setDynamicArtwork(false);
        themeActions.setLatinNames(false);
        themeActions.setSpoilers(true);
        themeActions.setMaintenanceCountdown(false);
        themeActions.setDataSaver(true);
        expect({ ...window.localStorage }).toEqual({ [KEYS.dynamicArtwork]: "false", [KEYS.latinNames]: "false", [KEYS.spoilers]: "true", [KEYS.maintenanceCountdown]: "false", [KEYS.dataSaver]: "true" });
        const { dynamicArtwork, latinNames, spoilers, maintenanceCountdown, dataSaver } = themeStore.state;
        expect({ dynamicArtwork, latinNames, spoilers, maintenanceCountdown, dataSaver }).toEqual({ dynamicArtwork: false, latinNames: false, spoilers: true, maintenanceCountdown: false, dataSaver: true });
        expect(hydrated()).toEqual({ dynamicArtwork: false, latinNames: false, spoilers: true, maintenanceCountdown: false, dataSaver: true });
    });

    it("mirrors data saver into a cookie the server render reads, and forgets it when off", async () => {
        themeActions.setDataSaver(true);
        await Promise.resolve();
        expect(document.cookie).toContain(`${DATA_SAVER_COOKIE}=1`);
        themeActions.setDataSaver(false);
        await Promise.resolve();
        expect(document.cookie).not.toContain(DATA_SAVER_COOKIE);
    });
});
