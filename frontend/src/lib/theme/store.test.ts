import { afterEach, describe, expect, it } from "vitest";
import { DATA_SAVER_COOKIE, hydrateTheme, themeActions, themeStore } from "./store";

const KEYS = { dynamicArtwork: "myrtle-dynamic-art", latinNames: "myrtle-latin-names", spoilers: "myrtle-spoilers", maintenanceCountdown: "myrtle-maintenance-countdown", dataSaver: "myrtle-data-saver", cnUpdates: "myrtle-cn-updates" } as const;

function hydrated() {
    hydrateTheme();
    const { dynamicArtwork, latinNames, spoilers, maintenanceCountdown, dataSaver, cnUpdates } = themeStore.state;
    return { dynamicArtwork, latinNames, spoilers, maintenanceCountdown, dataSaver, cnUpdates };
}

afterEach(() => window.localStorage.clear());

describe("theme store toggles", () => {
    it("reads the defaults when nothing is stored: spoilers and data saver off, the rest on", () => {
        expect(hydrated()).toEqual({ dynamicArtwork: true, latinNames: true, spoilers: false, maintenanceCountdown: true, dataSaver: false, cnUpdates: true });
    });

    it("reads only the exact string `true` as on", () => {
        for (const key of Object.values(KEYS)) window.localStorage.setItem(key, "TRUE");
        expect(hydrated()).toEqual({ dynamicArtwork: false, latinNames: false, spoilers: false, maintenanceCountdown: false, dataSaver: false, cnUpdates: false });
        for (const key of Object.values(KEYS)) window.localStorage.setItem(key, "true");
        expect(hydrated()).toEqual({ dynamicArtwork: true, latinNames: true, spoilers: true, maintenanceCountdown: true, dataSaver: true, cnUpdates: true });
    });

    it("stores each toggle under its own key and updates the store", () => {
        themeActions.setDynamicArtwork(false);
        themeActions.setLatinNames(false);
        themeActions.setSpoilers(true);
        themeActions.setMaintenanceCountdown(false);
        themeActions.setDataSaver(true);
        themeActions.setCnUpdates(false);
        expect({ ...window.localStorage }).toEqual({ [KEYS.dynamicArtwork]: "false", [KEYS.latinNames]: "false", [KEYS.spoilers]: "true", [KEYS.maintenanceCountdown]: "false", [KEYS.dataSaver]: "true", [KEYS.cnUpdates]: "false" });
        const { dynamicArtwork, latinNames, spoilers, maintenanceCountdown, dataSaver, cnUpdates } = themeStore.state;
        expect({ dynamicArtwork, latinNames, spoilers, maintenanceCountdown, dataSaver, cnUpdates }).toEqual({ dynamicArtwork: false, latinNames: false, spoilers: true, maintenanceCountdown: false, dataSaver: true, cnUpdates: false });
        expect(hydrated()).toEqual({ dynamicArtwork: false, latinNames: false, spoilers: true, maintenanceCountdown: false, dataSaver: true, cnUpdates: false });
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
