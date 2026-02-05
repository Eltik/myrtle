"use client";

import { useTheme } from "next-themes";
import { createContext, type ReactNode, useCallback, useContext, useEffect, useState } from "react";
import { DEFAULT_PRIMARY_HUE, generatePrimaryColors, getStoredAccentHue, setStoredAccentHue } from "~/lib/color-utils";

interface AccentColorContextType {
    /** Current hue value (0-360), or null for default */
    hue: number;
    /** Whether using the default color */
    isDefault: boolean;
    /** Set a new hue value */
    setHue: (hue: number) => void;
    /** Reset to default color */
    resetToDefault: () => void;
}

const AccentColorContext = createContext<AccentColorContextType | undefined>(undefined);

interface AccentColorProviderProps {
    children: ReactNode;
}

export function AccentColorProvider({ children }: AccentColorProviderProps) {
    const { resolvedTheme } = useTheme();
    const [hue, setHueState] = useState<number>(DEFAULT_PRIMARY_HUE);
    const [isDefault, setIsDefault] = useState(true);
    const [mounted, setMounted] = useState(false);

    useEffect(() => {
        setMounted(true);
        const stored = getStoredAccentHue();
        if (stored !== null) {
            setHueState(stored);
            setIsDefault(false);
        }
    }, []);

    useEffect(() => {
        if (!mounted) return;

        const colors = generatePrimaryColors(hue);
        const isDark = resolvedTheme === "dark";
        const themeColors = isDark ? colors.dark : colors.light;
        const navColors = colors.navPillHover;
        const glowColors = colors.glowEffects;

        const root = document.documentElement;

        root.style.setProperty("--primary", themeColors.primary);
        root.style.setProperty("--primary-foreground", themeColors.primaryForeground);
        root.style.setProperty("--ring", themeColors.ring);
        root.style.setProperty("--chart-1", themeColors.chart1);
        root.style.setProperty("--sidebar-primary", themeColors.sidebarPrimary);
        root.style.setProperty("--sidebar-primary-foreground", themeColors.sidebarPrimaryForeground);
        root.style.setProperty("--sidebar-ring", themeColors.sidebarRing);
        root.style.setProperty("--glow-primary", themeColors.glowPrimary);
        root.style.setProperty("--glow-primary-intense", themeColors.glowPrimaryIntense);

        root.style.setProperty("--nav-pill-hover-start", isDark ? navColors.dark : navColors.light);
        root.style.setProperty("--nav-pill-hover-end", isDark ? navColors.darkEnd : navColors.lightEnd);

        root.style.setProperty("--glow-header-border", glowColors.headerBorder);
        root.style.setProperty("--glow-header-border-secondary", glowColors.headerBorderSecondary);
        root.style.setProperty("--glow-header", glowColors.headerGlow);
        root.style.setProperty("--glow-header-secondary", glowColors.headerGlowSecondary);
        root.style.setProperty("--glow-nav", glowColors.navGlow);
        root.style.setProperty("--glow-nav-line-center", glowColors.navLineCenter);
        root.style.setProperty("--glow-nav-line-shadow", glowColors.navLineShadow);
        root.style.setProperty("--glow-nav-line-shadow-soft", glowColors.navLineShadowSoft);
        root.style.setProperty("--glow-text-strong", glowColors.textGlowStrong);
        root.style.setProperty("--glow-text-medium", glowColors.textGlowMedium);
        root.style.setProperty("--glow-text-soft", glowColors.textGlowSoft);
        root.style.setProperty("--glow-text-icon", glowColors.textGlowIcon);
    }, [hue, resolvedTheme, mounted]);

    const setHue = useCallback((newHue: number) => {
        const normalizedHue = ((newHue % 360) + 360) % 360;
        setHueState(normalizedHue);
        setIsDefault(false);
        setStoredAccentHue(normalizedHue);
    }, []);

    const resetToDefault = useCallback(() => {
        setHueState(DEFAULT_PRIMARY_HUE);
        setIsDefault(true);
        setStoredAccentHue(null);
    }, []);

    return <AccentColorContext.Provider value={{ hue, isDefault, setHue, resetToDefault }}>{children}</AccentColorContext.Provider>;
}

export function useAccentColor() {
    const context = useContext(AccentColorContext);
    if (context === undefined) {
        throw new Error("useAccentColor must be used within an AccentColorProvider");
    }
    return context;
}

/**
 * Hook that safely returns accent color context or defaults.
 * Use this in components that may render before the provider is mounted.
 */
export function useAccentColorSafe() {
    const context = useContext(AccentColorContext);
    return (
        context ?? {
            hue: DEFAULT_PRIMARY_HUE,
            isDefault: true,
            setHue: () => {},
            resetToDefault: () => {},
        }
    );
}
