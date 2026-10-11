import { useSelector } from "@tanstack/react-store";
import { useEffect } from "react";
import { hydrateTheme, subscribeSystemTheme, themeActions, themeStore } from "#/lib/theme/store";

export function useTheme() {
    const mode = useSelector(themeStore, (s) => s.mode);
    const resolved = useSelector(themeStore, (s) => s.resolved);
    const accent = useSelector(themeStore, (s) => s.accent);
    const dynamicArtwork = useSelector(themeStore, (s) => s.dynamicArtwork);
    const latinNames = useSelector(themeStore, (s) => s.latinNames);
    const spoilers = useSelector(themeStore, (s) => s.spoilers);
    const maintenanceCountdown = useSelector(themeStore, (s) => s.maintenanceCountdown);
    const hydrated = useSelector(themeStore, (s) => s.hydrated);

    useEffect(() => {
        if (!themeStore.state.hydrated) hydrateTheme();
        return subscribeSystemTheme();
    }, []);

    return {
        mode,
        resolved,
        accent,
        dynamicArtwork,
        latinNames,
        spoilers,
        maintenanceCountdown,
        hydrated,
        setMode: themeActions.setMode,
        setPresetHue: themeActions.setPresetHue,
        setCustomHex: themeActions.setCustomHex,
        resetAccent: themeActions.resetAccent,
        setDynamicArtwork: themeActions.setDynamicArtwork,
        setLatinNames: themeActions.setLatinNames,
        setSpoilers: themeActions.setSpoilers,
        setMaintenanceCountdown: themeActions.setMaintenanceCountdown,
        isDefaultAccent: accent === null,
    };
}
