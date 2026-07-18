import { useSelector } from "@tanstack/react-store";
import { useEffect } from "react";
import { hydrateTheme, subscribeSystemTheme, themeActions, themeStore } from "#/lib/theme/store";

export function useTheme() {
    const mode = useSelector(themeStore, (s) => s.mode);
    const resolved = useSelector(themeStore, (s) => s.resolved);
    const accent = useSelector(themeStore, (s) => s.accent);
    const dynamicArtwork = useSelector(themeStore, (s) => s.dynamicArtwork);
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
        hydrated,
        setMode: themeActions.setMode,
        setPresetHue: themeActions.setPresetHue,
        setCustomHex: themeActions.setCustomHex,
        resetAccent: themeActions.resetAccent,
        setDynamicArtwork: themeActions.setDynamicArtwork,
        isDefaultAccent: accent === null,
    };
}
