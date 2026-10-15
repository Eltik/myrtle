import { afterEach, describe, expect, it } from "vitest";
import { hydrateTheme, themeActions, themeStore } from "./store";

const KEYS = { dynamicArtwork: "myrtle-dynamic-art", latinNames: "myrtle-latin-names", spoilers: "myrtle-spoilers", maintenanceCountdown: "myrtle-maintenance-countdown" } as const;

function hydrated() {
    hydrateTheme();
    const { dynamicArtwork, latinNames, spoilers, maintenanceCountdown } = themeStore.state;
    return { dynamicArtwork, latinNames, spoilers, maintenanceCountdown };
}

afterEach(() => window.localStorage.clear());

describe("theme store toggles", () => {
    it("reads the defaults when nothing is stored: spoilers off, the rest on", () => {
        expect(hydrated()).toEqual({ dynamicArtwork: true, latinNames: true, spoilers: false, maintenanceCountdown: true });
    });

    it("reads only the exact string `true` as on", () => {
        for (const key of Object.values(KEYS)) window.localStorage.setItem(key, "TRUE");
        expect(hydrated()).toEqual({ dynamicArtwork: false, latinNames: false, spoilers: false, maintenanceCountdown: false });
        for (const key of Object.values(KEYS)) window.localStorage.setItem(key, "true");
        expect(hydrated()).toEqual({ dynamicArtwork: true, latinNames: true, spoilers: true, maintenanceCountdown: true });
    });

    it("stores each toggle under its own key and updates the store", () => {
        themeActions.setDynamicArtwork(false);
        themeActions.setLatinNames(false);
        themeActions.setSpoilers(true);
        themeActions.setMaintenanceCountdown(false);
        expect({ ...window.localStorage }).toEqual({ [KEYS.dynamicArtwork]: "false", [KEYS.latinNames]: "false", [KEYS.spoilers]: "true", [KEYS.maintenanceCountdown]: "false" });
        const { dynamicArtwork, latinNames, spoilers, maintenanceCountdown } = themeStore.state;
        expect({ dynamicArtwork, latinNames, spoilers, maintenanceCountdown }).toEqual({ dynamicArtwork: false, latinNames: false, spoilers: true, maintenanceCountdown: false });
        expect(hydrated()).toEqual({ dynamicArtwork: false, latinNames: false, spoilers: true, maintenanceCountdown: false });
    });
});
